use crate::api::{ApiClient, NodeInfo};
use crate::config::{load_config, save_config, Config};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "rh")]
#[command(about = "RunningHub AI CLI Client")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    Connect {
        #[arg(short, long)]
        api_key: String,
        #[arg(short, long)]
        webapp_id: String,
    },
    Nodes {
        #[arg(short, long)]
        webapp_id: Option<String>,
        #[arg(short, long)]
        api_key: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    Run {
        #[arg(short, long)]
        webapp_id: Option<String>,
        #[arg(short, long)]
        api_key: Option<String>,
        #[arg(short, long)]
        nodes: Option<String>,
        #[arg(short, long, default_value = "./output")]
        output: String,
    },
    Batch {
        #[arg(short, long)]
        batch: String,
        #[arg(short, long)]
        webapp_id: Option<String>,
        #[arg(short, long)]
        api_key: Option<String>,
        #[arg(short, long, default_value = "3")]
        concurrency: usize,
        #[arg(short, long, default_value = "./output")]
        output: String,
    },
    Status {
        #[arg(short, long)]
        task_id: String,
        #[arg(short, long)]
        api_key: Option<String>,
    },
    Apps {
        #[arg(short, long)]
        search: Option<String>,
        #[arg(short, long, default_value = "1")]
        page: i32,
        #[arg(long, default_value = "20")]
        size: i32,
    },
    Account {
        #[arg(short, long)]
        api_key: Option<String>,
    },
}

pub async fn run_cli(command: CliCommand) -> anyhow::Result<()> {
    let client = ApiClient::new();

    match command {
        CliCommand::Connect { api_key, webapp_id } => {
            save_config(&Config {
                api_key: Some(api_key.clone()),
                webapp_id: Some(webapp_id.clone()),
            })?;
            println!(
                "{}",
                "Configuration saved to ~/.runninghub/config.json"
                    .green()
            );
        }
        CliCommand::Nodes {
            webapp_id,
            api_key,
            output,
        } => {
            let config = load_config();
            let key = api_key
                .or(config.api_key)
                .ok_or_else(|| anyhow::anyhow!("API key required"))?;
            let wid = webapp_id
                .or(config.webapp_id)
                .ok_or_else(|| anyhow::anyhow!("WebApp ID required"))?;
            let result = client.get_node_list(&key, &wid).await?;
            println!(
                "{} {}",
                "App:".bold(),
                result
                    .app_info
                    .as_ref()
                    .map(|a| a.webapp_name.clone())
                    .unwrap_or_default()
            );
            println!("{} {}", "Nodes:".bold(), result.nodes.len());
            for node in &result.nodes {
                println!(
                    "  [{}] {}.{} ({}) = {}",
                    node.node_id.cyan(),
                    node.node_name,
                    node.field_name,
                    node.field_type.yellow(),
                    node.field_value
                );
            }
            if let Some(path) = output {
                fs::write(&path, serde_json::to_string_pretty(&result.nodes)?)?;
                println!("{} {}", "Saved to".green(), path);
            }
        }
        CliCommand::Run {
            webapp_id,
            api_key,
            nodes,
            output,
        } => {
            let config = load_config();
            let key = api_key
                .or(config.api_key)
                .ok_or_else(|| anyhow::anyhow!("API key required"))?;
            let wid = webapp_id
                .or(config.webapp_id)
                .ok_or_else(|| anyhow::anyhow!("WebApp ID required"))?;

            let mut node_list = if let Some(path) = nodes {
                let content = fs::read_to_string(&path)?;
                serde_json::from_str(&content)?
            } else {
                let result = client.get_node_list(&key, &wid).await?;
                println!(
                    "{} {}",
                    "Using app:".blue(),
                    result
                        .app_info
                        .as_ref()
                        .map(|a| a.webapp_name.clone())
                        .unwrap_or_default()
                );
                result.nodes
            };

            node_list = resolve_node_files(&client, &key, node_list).await?;

            println!("{}", "Submitting task...".blue());
            let result = client.submit_task(&key, &wid, &node_list).await?;
            println!("{} {}", "Task ID:".green(), result.task_id);

            let outputs = client.poll_task(&key, &result.task_id, 3).await?;
            println!(
                "{} {}",
                "Task completed!".green(),
                format!("{} output(s)", outputs.len())
            );

            fs::create_dir_all(&output)?;
            for (i, out) in outputs.iter().enumerate() {
                let url = crate::api::build_file_url(&out.file_url);
                let ext = out.file_type.as_deref().unwrap_or("png");
                let path = format!("{}/task_1_output_{}.{}", output, i + 1, ext);
                download_file(&url, &path).await?;
                println!("{} {}", "Saved:".green(), path);
            }
        }
        CliCommand::Batch {
            batch,
            webapp_id,
            api_key,
            concurrency,
            output,
        } => {
            let config = load_config();
            let key = api_key
                .or(config.api_key)
                .ok_or_else(|| anyhow::anyhow!("API key required"))?;
            let wid = webapp_id
                .or(config.webapp_id)
                .ok_or_else(|| anyhow::anyhow!("WebApp ID required"))?;

            let content = fs::read_to_string(&batch)?;
            let batch_list: Vec<Vec<NodeInfo>> = serde_json::from_str(&content)?;
            println!(
                "{} {}",
                "Loaded".blue(),
                format!("{} tasks, concurrency: {}", batch_list.len(), concurrency)
            );

            fs::create_dir_all(&output)?;
            let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
            let mut handles = vec![];

            for (index, nodes) in batch_list.into_iter().enumerate() {
                let client = ApiClient::new();
                let key = key.clone();
                let wid = wid.clone();
                let output = output.clone();
                let permit = semaphore.clone().acquire_owned().await?;
                handles.push(tokio::spawn(async move {
                    let _permit = permit;
                    println!("{} {} {}", "[Task".blue(), index + 1, "] Starting...");
                    match async {
                        let resolved = resolve_node_files(&client, &key, nodes).await?;
                        let result = client.submit_task(&key, &wid, &resolved).await?;
                        println!(
                            "{} {} {} {}",
                            "[Task".blue(),
                            index + 1,
                            "] Submitted:".blue(),
                            result.task_id
                        );
                        let outputs = client.poll_task(&key, &result.task_id, 3).await?;
                        println!(
                            "{} {} {} {}",
                            "[Task".green(),
                            index + 1,
                            "] Completed:".green(),
                            format!("{} output(s)", outputs.len())
                        );
                        for (i, out) in outputs.iter().enumerate() {
                            let url = crate::api::build_file_url(&out.file_url);
                            let ext = out.file_type.as_deref().unwrap_or("png");
                            let path = format!(
                                "{}/task_{}_output_{}.{}",
                                output,
                                index + 1,
                                i + 1,
                                ext
                            );
                            download_file(&url, &path).await?;
                            println!(
                                "{} {} {} {}",
                                "[Task".green(),
                                index + 1,
                                "] Saved:".green(),
                                path
                            );
                        }
                        Ok::<_, anyhow::Error>(())
                    }
                    .await
                    {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!(
                                "{} {} {} {}",
                                "[Task".red(),
                                index + 1,
                                "] Failed:".red(),
                                e
                            );
                            false
                        }
                    }
                }));
            }

            let mut completed = 0;
            let mut failed = 0;
            for h in handles {
                if h.await? {
                    completed += 1;
                } else {
                    failed += 1;
                }
            }
            println!(
                "\n{} {}",
                "Batch complete:".bold(),
                format!("{} succeeded, {} failed", completed, failed)
            );
        }
        CliCommand::Status { task_id, api_key } => {
            let config = load_config();
            let key = api_key
                .or(config.api_key)
                .ok_or_else(|| anyhow::anyhow!("API key required"))?;
            let result = client.query_task_outputs(&key, &task_id).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        CliCommand::Apps { search, page, size } => {
            let result = client
                .get_official_app_list(page, size, search.as_deref())
                .await?;
            println!(
                "{} {} {}",
                "Total:".bold(),
                result.total,
                format!("Page: {}", page)
            );
            for app in result.records {
                println!("\n{} {}", format!("[{}]", app.id).cyan(), app.name.bold());
                println!("  {}", app.intro);
            }
        }
        CliCommand::Account { api_key } => {
            let config = load_config();
            let key = api_key
                .or(config.api_key)
                .ok_or_else(|| anyhow::anyhow!("API key required"))?;
            let info = client.get_account_info(&key).await?;
            println!("{} {}", "Remaining Coins:".bold(), info.remain_coins);
            println!(
                "{} {}",
                "Current Tasks:".bold(),
                info.current_task_counts
            );
            if let Some(money) = info.remain_money {
                println!(
                    "{} {} {}",
                    "Remaining Money:".bold(),
                    money,
                    info.currency.as_deref().unwrap_or("")
                );
            }
            println!("{} {}", "API Type:".bold(), info.api_type);
        }
    }
    Ok(())
}

async fn resolve_node_files(
    client: &ApiClient,
    api_key: &str,
    nodes: Vec<NodeInfo>,
) -> anyhow::Result<Vec<NodeInfo>> {
    let mut result = vec![];
    for mut node in nodes {
        if ["IMAGE", "AUDIO", "VIDEO"].contains(&node.field_type.as_str()) {
            let value = &node.field_value;
            if !value.starts_with("http") && Path::new(value).exists() {
                let data = client.upload_file(api_key, value).await?;
                node.field_value = data.file_name;
            }
        }
        result.push(node);
    }
    Ok(result)
}

async fn download_file(url: &str, path: &str) -> anyhow::Result<()> {
    let resp = reqwest::get(url).await?;
    let bytes = resp.bytes().await?;
    fs::write(path, bytes)?;
    Ok(())
}

use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

const API_HOST: &str = "https://www.runninghub.cn";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub node_id: String,
    pub node_name: String,
    pub field_name: String,
    pub field_value: String,
    pub field_type: String,
    pub description: Option<String>,
    pub field_data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UploadData {
    pub file_name: String,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTaskData {
    pub task_id: String,
    pub prompt_tips: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskOutput {
    pub file_url: String,
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebAppInfo {
    pub webapp_name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeListResult {
    pub nodes: Vec<NodeInfo>,
    pub app_info: Option<WebAppInfo>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub remain_coins: String,
    pub current_task_counts: String,
    pub remain_money: Option<String>,
    pub currency: Option<String>,
    pub api_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppListItem {
    pub id: String,
    pub name: String,
    pub intro: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppListResult {
    pub records: Vec<AppListItem>,
    pub total: i32,
}

pub fn build_file_url(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with("data:") {
        return value.to_string();
    }
    format!("{}/task/openapi/view/{}", API_HOST, value)
}

pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn get_node_list(
        &self,
        api_key: &str,
        webapp_id: &str,
    ) -> anyhow::Result<GetNodeListResult> {
        let url = format!(
            "{}/api/webapp/apiCallDemo?apiKey={}&webappId={}",
            API_HOST, api_key, webapp_id
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;
        let json: ApiResponse<serde_json::Value> = resp.json().await?;
        if json.code != 0 {
            anyhow::bail!("{}", json.msg);
        }
        let data = json.data.unwrap_or_default();
        let nodes: Vec<NodeInfo> = data["nodeInfoList"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|n| NodeInfo {
                node_id: n["nodeId"].as_str().unwrap_or("").to_string(),
                node_name: n["nodeName"].as_str().unwrap_or("").to_string(),
                field_name: n["fieldName"].as_str().unwrap_or("").to_string(),
                field_value: n["fieldValue"].as_str().unwrap_or("").to_string(),
                field_type: n["fieldType"].as_str().unwrap_or("STRING").to_string(),
                description: n["description"].as_str().map(|s| s.to_string()),
                field_data: n["fieldData"]
                    .as_str()
                    .or(n["field_data"].as_str())
                    .or(n["options"].as_str())
                    .map(|s| s.to_string()),
            })
            .collect();
        let app_info = if data["webappName"].is_string() {
            Some(WebAppInfo {
                webapp_name: data["webappName"].as_str().unwrap_or("").to_string(),
                description: data["description"].as_str().unwrap_or("").to_string(),
            })
        } else {
            None
        };
        Ok(GetNodeListResult { nodes, app_info })
    }

    pub async fn upload_file(&self, api_key: &str, file_path: &str) -> anyhow::Result<UploadData> {
        let url = format!("{}/task/openapi/upload", API_HOST);
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let file_content = tokio::fs::read(file_path).await?;
        let part = multipart::Part::bytes(file_content).file_name(file_name.to_string());
        let form = multipart::Form::new()
            .text("apiKey", api_key.to_string())
            .text("fileType", "input")
            .part("file", part);
        let resp = self.client.post(&url).multipart(form).send().await?;
        let json: ApiResponse<UploadData> = resp.json().await?;
        if json.code != 0 {
            anyhow::bail!("{}", json.msg);
        }
        Ok(json.data.unwrap())
    }

    pub async fn submit_task(
        &self,
        api_key: &str,
        webapp_id: &str,
        nodes: &[NodeInfo],
    ) -> anyhow::Result<SubmitTaskData> {
        let url = format!("{}/task/openapi/ai-app/run", API_HOST);
        let payload = serde_json::json!({
            "webappId": webapp_id,
            "apiKey": api_key,
            "nodeInfoList": nodes
        });
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        let json: ApiResponse<SubmitTaskData> = resp.json().await?;
        if json.code != 0 {
            anyhow::bail!("{}", json.msg);
        }
        Ok(json.data.unwrap())
    }

    pub async fn query_task_outputs(
        &self,
        api_key: &str,
        task_id: &str,
    ) -> anyhow::Result<ApiResponse<serde_json::Value>> {
        let url = format!("{}/task/openapi/outputs", API_HOST);
        let payload = serde_json::json!({ "apiKey": api_key, "taskId": task_id });
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_account_info(&self, api_key: &str) -> anyhow::Result<AccountInfo> {
        let url = format!("{}/uc/openapi/accountStatus", API_HOST);
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Host", "www.runninghub.cn")
            .json(&serde_json::json!({ "apikey": api_key }))
            .send()
            .await?;
        let json: ApiResponse<AccountInfo> = resp.json().await?;
        if json.code != 0 {
            anyhow::bail!("{}", json.msg);
        }
        Ok(json.data.unwrap())
    }

    pub async fn get_official_app_list(
        &self,
        page: i32,
        size: i32,
        search: Option<&str>,
    ) -> anyhow::Result<AppListResult> {
        let url = format!("{}/api/webapp/list", API_HOST);
        let mut body = serde_json::json!({
            "current": page,
            "size": size,
            "carefullyChosen": true,
            "sort": "RECOMMEND"
        });
        if let Some(s) = search {
            body["search"] = serde_json::json!(s);
        }
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        let json: ApiResponse<serde_json::Value> = resp.json().await?;
        if json.code != 0 {
            anyhow::bail!("{}", json.msg);
        }
        let data = json.data.unwrap_or_default();
        let records: Vec<AppListItem> =
            serde_json::from_value(data["records"].clone()).unwrap_or_default();
        let total = data["total"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        Ok(AppListResult { records, total })
    }

    pub async fn poll_task(
        &self,
        api_key: &str,
        task_id: &str,
        interval_secs: u64,
    ) -> anyhow::Result<Vec<TaskOutput>> {
        loop {
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            let result = self.query_task_outputs(api_key, task_id).await?;
            if result.code != 0 {
                anyhow::bail!("Query failed: {}", result.msg);
            }
            let data = result.data.unwrap_or_default();
            if let Ok(outputs) = serde_json::from_value::<Vec<TaskOutput>>(data.clone()) {
                if !outputs.is_empty() {
                    return Ok(outputs);
                }
            }
            if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
                if status == "FAILED" {
                    let reason = data
                        .get("failedReason")
                        .and_then(|r| r.get("exception_message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    anyhow::bail!("Task failed: {}", reason);
                }
            }
        }
    }
}

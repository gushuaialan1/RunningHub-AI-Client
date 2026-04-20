use clap::Parser;

mod api;
mod cli;
mod config;

use cli::CliArgs;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    if let Err(e) = cli::run_cli(args.command).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

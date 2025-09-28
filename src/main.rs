mod commands;
mod examples;

use anyhow::Result;

use clap::Parser;
use colored::*;

use commands::{
    Commands, SearchAction, SyncAction, add_tool, create_github_repo, delete_tool, init_config,
    install_tool, list_github_repos, list_tools, pull_config_from_github, push_config_to_github,
    remove_tool, reset_config, run_tool, search_all_tools, search_local_tools, search_remote_tools,
    setup_github_sync, show_sync_status, update_tool,
};
use examples::show_examples;

#[derive(Parser)]
#[command(name = "tkit")]
#[command(about = "A customizable tool manager")]
#[command(version = "0.1.1")]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install { tool } => install_tool(&tool).await,
        Commands::Remove { tool } => remove_tool(&tool).await,
        Commands::Update { tool } => update_tool(&tool).await,
        Commands::List => list_tools(),
        Commands::Add { tool } => add_tool(&tool).await,
        Commands::Delete { tool } => delete_tool(&tool).await,
        Commands::Run { tool } => run_tool(&tool).await,
        Commands::Examples => show_examples(),
        Commands::Search { action } => match action {
            SearchAction::Local { query } => search_local_tools(&query).await,
            SearchAction::Remote { query } => search_remote_tools(&query).await,
            SearchAction::All { query } => search_all_tools(&query).await,
        },
        Commands::Init => init_config().await,
        Commands::Reset => reset_config(),
        Commands::Sync { action } => match action {
            SyncAction::Setup { repo, token } => setup_github_sync(repo, token).await,
            SyncAction::CreateRepo { name, private } => create_github_repo(&name, private).await,
            SyncAction::ListRepos => list_github_repos().await,
            SyncAction::Push => push_config_to_github().await,
            SyncAction::Pull => pull_config_from_github().await,
            SyncAction::Status => show_sync_status().await,
        },
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}

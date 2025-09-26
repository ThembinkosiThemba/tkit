// > manual description for installing tools
// > after update commad, it should automatically save when creating a new tool, instead of waiting for me to press enter
// > search command to search any package from remote registries, including crates.io, npm, apt, snap, web socs
// > run command to run a tool
// > delete a tool from config
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use clap::{Parser, Subcommand};
use colored::*;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "tkit")]
#[command(about = "A customizable tool manager")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a tool
    Install { tool: String },
    /// Remove a tool
    Remove { tool: String },
    /// Update a tool
    Update { tool: String },
    /// List available tools
    List,
    /// Add a new tool configuration
    Add { tool: String },
    /// Delete a tool configuration
    Delete { tool: String },
    /// Run a tool
    Run { tool: String },
    /// Initialize the tkit configuration
    Init,
    /// Sync configuration with GitHub
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
}

#[derive(Subcommand)]
enum SyncAction {
    /// Setup GitHub integration
    Setup {
        /// GitHub repository (username/repo-name)
        repo: String,
        /// GitHub personal access token
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Push local config to GitHub
    Push,
    /// Pull config from GitHub
    Pull,
    /// Show sync status
    Status,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ToolConfig {
    name: String,
    description: Option<String>,
    install_commands: Vec<String>,
    remove_commands: Vec<String>,
    update_commands: Vec<String>,
    #[serde(default)]
    run_commands: Vec<String>,
    #[serde(default)]
    installed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    tools: HashMap<String, ToolConfig>,
    #[serde(default)]
    sync: SyncConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct SyncConfig {
    repo: Option<String>,
    token: Option<String>,
    last_sync: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubFile {
    name: String,
    path: String,
    sha: String,
    size: u64,
    url: String,
    html_url: String,
    git_url: String,
    download_url: Option<String>,
    #[serde(rename = "type")]
    file_type: String,
    content: Option<String>,
    encoding: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubCreateFile {
    message: String,
    content: String,
    sha: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubResponse {
    content: GitHubFile,
}

impl Config {
    fn new() -> Self {
        Self {
            tools: HashMap::new(),
            sync: SyncConfig::default(),
        }
    }

    fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        if !config_path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;
    Ok(config_dir.join("tkit").join("config.yaml"))
}


async fn execute_commands(commands: &[String], tool_name: &str, action: &str) -> Result<()> {
    if commands.is_empty() {
        println!(
            "{}",
            format!("No {} commands defined for {}", action, tool_name).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("{}ing {}...", action.to_title_case(), tool_name)
            .blue()
            .bold()
    );

    for (i, cmd) in commands.iter().enumerate() {
        println!("{}", format!("  Step {}: {}", i + 1, cmd).cyan());

        let mut parts = cmd.split_whitespace();
        let program = parts.next().ok_or_else(|| anyhow!("Empty command"))?;
        let args: Vec<&str> = parts.collect();

        let output = Command::new(program).args(&args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Command failed: {}\nError: {}", cmd, stderr));
        }

        // Print stdout if there's any
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("    {}", stdout.trim());
        }
    }

    println!(
        "{}",
        format!("✓ {} completed successfully!", action.to_title_case())
            .green()
            .bold()
    );
    Ok(())
}

trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        let mut chars = self.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

async fn install_tool(tool_name: &str) -> Result<()> {
    let mut config = Config::load()?;

    let tool = config.tools.get_mut(tool_name).ok_or_else(|| {
        anyhow!(
            "Tool '{}' not found. Use 'tkit add {}' to add it first.",
            tool_name,
            tool_name
        )
    })?;

    if tool.installed {
        println!(
            "{}",
            format!("Tool '{}' is already installed.", tool_name).yellow()
        );
        return Ok(());
    }

    execute_commands(&tool.install_commands, tool_name, "install").await?;

    tool.installed = true;
    config.save()?;
    Ok(())
}

async fn remove_tool(tool_name: &str) -> Result<()> {
    let mut config = Config::load()?;

    let tool = config
        .tools
        .get_mut(tool_name)
        .ok_or_else(|| anyhow!("Tool '{}' not found.", tool_name))?;

    if !tool.installed {
        println!(
            "{}",
            format!("Tool '{}' is not installed.", tool_name).yellow()
        );
        return Ok(());
    }

    execute_commands(&tool.remove_commands, tool_name, "remove").await?;

    tool.installed = false;
    config.save()?;
    Ok(())
}

async fn update_tool(tool_name: &str) -> Result<()> {
    let config = Config::load()?;

    let tool = config
        .tools
        .get(tool_name)
        .ok_or_else(|| anyhow!("Tool '{}' not found.", tool_name))?;

    if !tool.installed {
        println!(
            "{}",
            format!("Tool '{}' is not installed. Install it first.", tool_name).yellow()
        );
        return Ok(());
    }

    execute_commands(&tool.update_commands, tool_name, "update").await?;
    Ok(())
}

fn list_tools() -> Result<()> {
    let config = Config::load()?;

    if config.tools.is_empty() {
        println!(
            "{}",
            "No tools configured. Use 'tkit add <tool>' to add some!".yellow()
        );
        return Ok(());
    }

    println!("{}", "Available tools:".blue().bold());
    for (name, tool) in &config.tools {
        let status = if tool.installed {
            "✓".green()
        } else {
            "✗".red()
        };
        let desc = tool.description.as_deref().unwrap_or("No description");
        println!("  {} {} - {}", status, name.bold(), desc);
    }
    Ok(())
}

async fn validate_github_access(repo: &str, token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}", repo);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("tkit/0.1.0"));

    let response = client.get(&url).headers(headers).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to access repository '{}'. Status: {}. Check your token and repository name.",
            repo,
            response.status()
        ));
    }

    Ok(())
}

async fn setup_github_sync(repo: String, token: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    // Get token from user if not provided
    let token = if let Some(t) = token {
        t
    } else {
        use std::io::{self, Write};
        print!("Enter your GitHub Personal Access Token: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Validate token and repo
    validate_github_access(&repo, &token).await?;

    config.sync.repo = Some(repo.clone());
    config.sync.token = Some(token);
    config.save()?;

    println!(
        "{}",
        format!("✓ GitHub sync configured for repository: {}", repo)
            .green()
            .bold()
    );
    println!("  Use 'tkit sync push' to upload your config");
    println!("  Use 'tkit sync pull' to download config from GitHub");

    Ok(())
}

async fn push_config_to_github() -> Result<()> {
    let config = Config::load()?;

    let repo = config.sync.repo.as_ref().ok_or_else(|| {
        anyhow!("GitHub sync not configured. Run 'tkit sync setup <repo>' first.")
    })?;
    let token =
        config.sync.token.as_ref().ok_or_else(|| {
            anyhow!("GitHub token not found. Run 'tkit sync setup <repo>' first.")
        })?;

    // Create a copy of config without the token for pushing to GitHub
    let mut safe_config = config.clone();
    safe_config.sync.token = None;
    let config_content = serde_yaml::to_string(&safe_config)?;
    let encoded_content = general_purpose::STANDARD.encode(config_content);

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/contents/tkit-config.yaml",
        repo
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("tkit/0.1.0"));

    // Check if file exists to get SHA
    let existing_response = client.get(&url).headers(headers.clone()).send().await;
    let sha = if let Ok(response) = existing_response {
        if response.status().is_success() {
            let file: GitHubFile = response.json().await?;
            Some(file.sha)
        } else {
            None
        }
    } else {
        None
    };

    let payload = GitHubCreateFile {
        message: format!(
            "Update tkit config - {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ),
        content: encoded_content,
        sha,
    };

    let response = client
        .put(&url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!(
            "{}",
            "✓ Configuration pushed to GitHub successfully!"
                .green()
                .bold()
        );

        // Update last sync time
        let mut updated_config = config;
        updated_config.sync.last_sync = Some(chrono::Utc::now().to_rfc3339());
        updated_config.save()?;
    } else {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to push to GitHub: {}", error_text));
    }

    Ok(())
}

async fn show_sync_status() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "GitHub Sync Status:".blue().bold());

    if let Some(repo) = &config.sync.repo {
        println!("  Repository: {}", repo.green());
        println!(
            "  Token: {}",
            if config.sync.token.is_some() {
                "✓ Configured".green()
            } else {
                "✗ Not set".red()
            }
        );

        if let Some(last_sync) = &config.sync.last_sync {
            println!("  Last sync: {}", last_sync);
        } else {
            println!("  Last sync: {}", "Never".yellow());
        }
    } else {
        println!("  Status: {}", "Not configured".yellow());
        println!("  Run 'tkit sync setup <username/repo>' to get started");
    }

    Ok(())
}

async fn pull_config_from_github() -> Result<()> {
    let config = Config::load()?;

    let repo = config.sync.repo.as_ref().ok_or_else(|| {
        anyhow!("GitHub sync not configured. Run 'tkit sync setup <repo>' first.")
    })?;
    let token =
        config.sync.token.as_ref().ok_or_else(|| {
            anyhow!("GitHub token not found. Run 'tkit sync setup <repo>' first.")
        })?;

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/contents/tkit-config.yaml",
        repo
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("tkit/0.1.0"));

    let response = client.get(&url).headers(headers).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch config from GitHub. Make sure the file exists and you have access."
        ));
    }

    let file: GitHubFile = response.json().await?;

    let content = file
        .content
        .ok_or_else(|| anyhow!("No content in GitHub file"))?;
    let decoded_content = general_purpose::STANDARD.decode(content.replace('\n', ""))?;
    let config_str = String::from_utf8(decoded_content)?;

    let remote_config: Config = serde_yaml::from_str(&config_str)?;

    // Backup current config
    let backup_path = get_config_path()?.with_extension("yaml.backup");
    if let Ok(current_content) = fs::read_to_string(get_config_path()?) {
        fs::write(&backup_path, current_content)?;
        println!(
            "{}",
            format!("✓ Current config backed up to: {}", backup_path.display()).yellow()
        );
    }

    // Merge configurations (preserve local sync settings)
    let mut merged_config = remote_config;
    merged_config.sync = config.sync; // Keep local sync settings
    merged_config.sync.last_sync = Some(chrono::Utc::now().to_rfc3339());

    merged_config.save()?;

    println!(
        "{}",
        "✓ Configuration pulled from GitHub successfully!"
            .green()
            .bold()
    );
    println!("  {} tools loaded", merged_config.tools.len());

    Ok(())
}

fn add_tool(tool_name: &str) -> Result<()> {
    use std::io::{self, Write};
    
    let mut config = Config::load()?;

    if config.tools.contains_key(tool_name) {
        println!(
            "{}",
            format!("Tool '{}' already exists.", tool_name).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Adding tool '{}'...", tool_name).blue().bold()
    );

    // Get description first (required)
    print!("Description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    std::io::stdin().read_line(&mut description)?;
    let description = description.trim().to_string();
    
    if description.is_empty() {
        return Err(anyhow!("Description is required"));
    }

    println!("Enter commands for each action (empty line to finish):");

    let install_commands = read_commands("Install")?;
    let remove_commands = read_commands("Remove")?;
    let update_commands = read_commands("Update")?;
    let run_commands = read_commands("Run")?;

    let tool_config = ToolConfig {
        name: tool_name.to_string(),
        description: Some(description),
        install_commands,
        remove_commands,
        update_commands,
        run_commands,
        installed: false,
    };

    config.tools.insert(tool_name.to_string(), tool_config);
    config.save()?;

    println!(
        "{}",
        format!("✓ Tool '{}' added successfully!", tool_name)
            .green()
            .bold()
    );
    Ok(())
}

fn delete_tool(tool_name: &str) -> Result<()> {
    let mut config = Config::load()?;

    if !config.tools.contains_key(tool_name) {
        println!(
            "{}",
            format!("Tool '{}' not found.", tool_name).yellow()
        );
        return Ok(());
    }

    config.tools.remove(tool_name);
    config.save()?;

    println!(
        "{}",
        format!("✓ Tool '{}' deleted successfully!", tool_name)
            .green()
            .bold()
    );
    Ok(())
}

async fn run_tool(tool_name: &str) -> Result<()> {
    let config = Config::load()?;

    let tool = config
        .tools
        .get(tool_name)
        .ok_or_else(|| anyhow!("Tool '{}' not found.", tool_name))?;

    if tool.run_commands.is_empty() {
        println!(
            "{}",
            format!("No run commands defined for '{}'.", tool_name).yellow()
        );
        return Ok(());
    }

    execute_commands(&tool.run_commands, tool_name, "run").await?;
    Ok(())
}

fn read_commands(action: &str) -> Result<Vec<String>> {
    use std::io::{self, Write};

    println!("{}", format!("{} commands:", action).cyan().bold());
    let mut commands = Vec::new();
    let mut line_num = 1;

    loop {
        print!("  {}: ", line_num);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        commands.push(input.to_string());
        line_num += 1;
    }

    Ok(commands)
}

fn init_config() -> Result<()> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        println!("{}", "Configuration already exists.".yellow());
        return Ok(());
    }

    let mut config = Config::new();

    // Add some example tools
    config.tools.insert(
        "node".to_string(),
        ToolConfig {
            name: "node".to_string(),
            description: Some("Node.js runtime".to_string()),
            install_commands: vec![
                "curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -".to_string(),
                "sudo apt-get install -y nodejs".to_string(),
            ],
            remove_commands: vec!["sudo apt-get remove -y nodejs".to_string()],
            update_commands: vec![
                "sudo apt-get update".to_string(),
                "sudo apt-get upgrade -y nodejs".to_string(),
            ],
            run_commands: vec![
                "node --version".to_string(),
                "npm --version".to_string(),
            ],
            installed: false,
        },
    );

    config.tools.insert(
        "docker".to_string(),
        ToolConfig {
            name: "docker".to_string(),
            description: Some("Docker container platform".to_string()),
            install_commands: vec![
                "curl -fsSL https://get.docker.com -o get-docker.sh".to_string(),
                "sudo sh get-docker.sh".to_string(),
                "sudo usermod -aG docker $USER".to_string(),
            ],
            remove_commands: vec![
                "sudo apt-get remove -y docker docker-engine docker.io containerd runc".to_string(),
            ],
            update_commands: vec![
                "sudo apt-get update".to_string(),
                "sudo apt-get upgrade -y docker-ce".to_string(),
            ],
            run_commands: vec![
                "docker --version".to_string(),
                "docker ps".to_string(),
            ],
            installed: false,
        },
    );

    config.save()?;

    println!(
        "{}",
        "✓ Configuration initialized with example tools!"
            .green()
            .bold()
    );
    println!("  Run 'tkit list' to see available tools");
    println!("  Run 'tkit add <tool>' to add new tools");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install { tool } => install_tool(&tool).await,
        Commands::Remove { tool } => remove_tool(&tool).await,
        Commands::Update { tool } => update_tool(&tool).await,
        Commands::List => list_tools(),
        Commands::Add { tool } => add_tool(&tool),
        Commands::Delete { tool } => delete_tool(&tool),
        Commands::Run { tool } => run_tool(&tool).await,
        Commands::Init => init_config(),
        Commands::Sync { action } => match action {
            SyncAction::Setup { repo, token } => setup_github_sync(repo, token).await,
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

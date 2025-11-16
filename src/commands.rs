use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use clap::Subcommand;
use colored::*;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{fs, process::Command};
use tkit::{Config, ToolConfig, get_config_path};

#[derive(Subcommand)]
pub enum Commands {
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
    /// Show examples of tool configurations
    Examples,
    /// Initialize the tkit configuration
    Init,
    /// Reset configuration (clear all tools and settings)
    Reset,
    /// Sync configuration with GitHub
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Setup GitHub integration
    Setup {
        /// GitHub repository (username/repo-name)
        repo: String,
        /// GitHub personal access token
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Create a new GitHub repository
    CreateRepo {
        /// Repository name
        name: String,
        /// Make repository private
        #[arg(short, long)]
        private: bool,
    },
    /// Update GitHub personal access token
    UpdateToken {
        /// New GitHub personal access token
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
struct GitHubRepo {
    id: u64,
    name: String,
    full_name: String,
    description: Option<String>,
    private: bool,
    html_url: String,
    clone_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateRepoRequest {
    name: String,
    description: Option<String>,
    private: bool,
    auto_init: bool,
}

pub async fn execute_commands(commands: &[String], tool_name: &str, action: &str) -> Result<()> {
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

// install_tool is used to install a particular configured tool
pub async fn install_tool(tool_name: &str) -> Result<()> {
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

    auto_sync_if_enabled(&config).await?;

    Ok(())
}

pub async fn remove_tool(tool_name: &str) -> Result<()> {
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

    // Auto-sync if enabled
    auto_sync_if_enabled(&config).await?;

    Ok(())
}

pub async fn update_tool(tool_name: &str) -> Result<()> {
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

pub fn list_tools() -> Result<()> {
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

pub async fn validate_github_access(repo: &str, token: &str) -> Result<()> {
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

pub async fn setup_github_sync(repo: String, token: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

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

pub async fn update_github_token(token: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    // Check if sync is already configured
    let repo = config.sync.repo.as_ref().ok_or_else(|| {
        anyhow!("GitHub sync not configured. Run 'tkit sync setup <repo>' first.")
    })?;

    let token = if let Some(t) = token {
        t
    } else {
        use std::io::{self, Write};
        print!("Enter your new GitHub Personal Access Token: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Validate the new token
    validate_github_access(repo, &token).await?;

    // Update the token
    config.sync.token = Some(token);
    config.save()?;

    println!(
        "{}",
        "✓ GitHub token updated successfully!"
            .green()
            .bold()
    );

    Ok(())
}

pub async fn push_config_to_github() -> Result<()> {
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

pub async fn show_sync_status() -> Result<()> {
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

        println!(
            "  Auto-sync: {}",
            if config.sync.auto_sync {
                "✓ Enabled".green()
            } else {
                "✗ Disabled".red()
            }
        );
    } else {
        println!("  Status: {}", "Not configured".yellow());
        println!("  Run 'tkit sync setup <username/repo>' to get started");
    }

    Ok(())
}

pub async fn pull_config_from_github() -> Result<()> {
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

pub async fn add_tool(tool_name: &str) -> Result<()> {
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

    // Auto-sync if enabled
    auto_sync_if_enabled(&config).await?;

    println!(
        "{}",
        format!("✓ Tool '{}' added successfully!", tool_name)
            .green()
            .bold()
    );
    Ok(())
}

pub async fn delete_tool(tool_name: &str) -> Result<()> {
    let mut config = Config::load()?;

    if !config.tools.contains_key(tool_name) {
        println!("{}", format!("Tool '{}' not found.", tool_name).yellow());
        return Ok(());
    }

    config.tools.remove(tool_name);
    config.save()?;

    // Auto-sync if enabled
    auto_sync_if_enabled(&config).await?;

    println!(
        "{}",
        format!("✓ Tool '{}' deleted successfully!", tool_name)
            .green()
            .bold()
    );
    Ok(())
}

pub async fn run_tool(tool_name: &str) -> Result<()> {
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

pub fn read_commands(action: &str) -> Result<Vec<String>> {
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

pub async fn init_config() -> Result<()> {
    use std::io::{self, Write};

    let config_path = get_config_path()?;

    if config_path.exists() {
        println!("{}", "Configuration already exists.".yellow());
        print!("Do you want to reset and start fresh? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            return Ok(());
        }
        reset_config()?;
        println!();
    }

    println!("{}", "Welcome to TKIT Setup Wizard!".blue().bold());
    println!("Let's get you set up with your personalized tool manager.");
    println!();

    let mut config = Config::new();

    // Step 1: Basic setup
    println!("{}", "Step 1: Basic Configuration".cyan().bold());
    println!("First, let's add some essential tools to get you started.");
    println!();

    // Ask about example tools
    let examples = vec![
        (
            "git",
            "Version control system",
            vec!["sudo apt-get update", "sudo apt-get install -y git"],
            vec!["git --version"],
        ),
        (
            "docker",
            "Container platform",
            vec![
                "curl -fsSL https://get.docker.com -o get-docker.sh",
                "sudo sh get-docker.sh",
            ],
            vec!["docker --version"],
        ),
        (
            "node",
            "Node.js runtime",
            vec![
                "curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -",
                "sudo apt-get install -y nodejs",
            ],
            vec!["node --version", "npm --version"],
        ),
        (
            "python",
            "Python programming language",
            vec![
                "sudo apt-get update",
                "sudo apt-get install -y python3 python3-pip",
            ],
            vec!["python3 --version"],
        ),
    ];

    for (name, desc, install_cmds, run_cmds) in &examples {
        print!("Add {} ({})?  (Y/n): ", name.bold(), desc);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            config.tools.insert(
                name.to_string(),
                ToolConfig {
                    name: name.to_string(),
                    description: Some(desc.to_string()),
                    install_commands: install_cmds.iter().map(|s| s.to_string()).collect(),
                    remove_commands: vec![format!("sudo apt-get remove -y {}", name)],
                    update_commands: vec![
                        "sudo apt-get update".to_string(),
                        format!("sudo apt-get upgrade -y {}", name),
                    ],
                    run_commands: run_cmds.iter().map(|s| s.to_string()).collect(),
                    installed: false,
                },
            );
            println!("  ✓ Added {}", name.green());
        }
    }

    println!();

    // Step 2: GitHub Integration
    println!("{}", "Step 2: GitHub Integration (Optional)".cyan().bold());
    println!("TKIT can sync your configuration to GitHub for backup and sharing across machines.");
    println!();

    print!("Set up GitHub sync? (y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        println!();
        println!("GitHub setup options:");
        println!("1. Create a new repository automatically");
        println!("2. Use an existing repository");
        println!("3. Skip for now");

        print!("Choose option (1-3): ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => {
                // Create new repo
                print!("Enter GitHub Personal Access Token: ");
                io::stdout().flush()?;

                let mut token = String::new();
                io::stdin().read_line(&mut token)?;
                let token = token.trim();

                if !token.is_empty() {
                    config.sync.token = Some(token.to_string());

                    print!("Repository name (default: tkit-config): ");
                    io::stdout().flush()?;

                    let mut repo_name = String::new();
                    io::stdin().read_line(&mut repo_name)?;
                    let repo_name = if repo_name.trim().is_empty() {
                        "tkit-config"
                    } else {
                        repo_name.trim()
                    };

                    print!("Make repository private? (Y/n): ");
                    io::stdout().flush()?;

                    let mut private_input = String::new();
                    io::stdin().read_line(&mut private_input)?;
                    let private = private_input.trim().to_lowercase() != "n";

                    // Temporarily save config with token
                    config.save()?;

                    match create_github_repo(repo_name, private).await {
                        Ok(()) => {
                            println!("  ✓ GitHub repository created and configured!");

                            // Reload config to get the updated repo info
                            config = Config::load()?;
                        }
                        Err(e) => {
                            println!("  ⚠️  Failed to create repository: {}", e);
                            config.sync.token = None; // Clear token on failure
                        }
                    }
                }
            }
            "2" => {
                // Use existing repo
                print!("Enter repository (username/repo-name): ");
                io::stdout().flush()?;

                let mut repo = String::new();
                io::stdin().read_line(&mut repo)?;
                let repo = repo.trim();

                if !repo.is_empty() {
                    print!("Enter GitHub Personal Access Token: ");
                    io::stdout().flush()?;

                    let mut token = String::new();
                    io::stdin().read_line(&mut token)?;
                    let token = token.trim();

                    if !token.is_empty() {
                        config.sync.repo = Some(repo.to_string());
                        config.sync.token = Some(token.to_string());

                        // Validate access
                        match validate_github_access(repo, token).await {
                            Ok(()) => println!("  ✓ GitHub sync configured!"),
                            Err(e) => {
                                println!("  ⚠️  Failed to validate GitHub access: {}", e);
                                config.sync.repo = None;
                                config.sync.token = None;
                            }
                        }
                    }
                }
            }
            _ => println!("  Skipping GitHub setup."),
        }

        // Auto-sync option
        if config.sync.repo.is_some() && config.sync.token.is_some() {
            println!();
            print!("Enable automatic sync on configuration changes? (Y/n): ");
            io::stdout().flush()?;

            let mut auto_sync_input = String::new();
            io::stdin().read_line(&mut auto_sync_input)?;
            let auto_sync = auto_sync_input.trim().to_lowercase() != "n";

            config.sync.auto_sync = auto_sync;

            if auto_sync {
                println!("  ✓ Auto-sync enabled - your changes will be automatically backed up!");
            } else {
                println!("  ✓ Manual sync mode - use 'tkit sync push' to backup your config");
            }
        }
    }

    println!();

    // Step 3: Add a custom tool
    println!(
        "{}",
        "Step 3: Add Your First Custom Tool (Optional)"
            .cyan()
            .bold()
    );
    print!("Would you like to add a custom tool now? (y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        println!();
        print!("Tool name: ");
        io::stdout().flush()?;

        let mut tool_name = String::new();
        io::stdin().read_line(&mut tool_name)?;
        let tool_name = tool_name.trim();

        if !tool_name.is_empty() {
            if let Err(e) = add_tool(tool_name).await {
                println!("Failed to add tool: {}", e);
            }
            // Reload config after adding tool
            config = Config::load()?;
        }
    }

    // Final save
    config.save()?;

    // Auto-sync if enabled
    auto_sync_if_enabled(&config).await?;

    println!();
    println!("{}", "🎉 Setup Complete!".green().bold());
    println!("Your TKIT configuration is ready to use.");
    println!();
    println!("{}", "Next steps:".yellow().bold());
    println!("  • Run 'tkit list' to see your configured tools");
    println!("  • Run 'tkit examples' to see more tool ideas");
    println!("  • Run 'tkit add <tool>' to add more custom tools");

    if config.sync.repo.is_some() {
        println!(
            "  • Your config is synced to GitHub: {}",
            config.sync.repo.as_ref().unwrap().cyan()
        );
    }

    Ok(())
}

pub fn reset_config() -> Result<()> {
    use std::io::{self, Write};

    println!("{}", "⚠️  Reset Configuration".red().bold());
    println!("This will permanently delete:");
    println!("  • All configured tools");
    println!("  • GitHub sync settings");
    println!("  • All configuration data");
    println!();

    print!("Are you sure you want to continue? Type 'yes' to confirm: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input != "yes" {
        println!("{}", "Reset cancelled.".yellow());
        return Ok(());
    }

    // Remove config file
    let config_path = get_config_path()?;
    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
        println!("{}", "✓ Configuration file deleted".green());
    }

    // Remove config directory if empty
    if let Some(config_dir) = config_path.parent() {
        if config_dir.exists() && config_dir.read_dir()?.next().is_none() {
            std::fs::remove_dir(config_dir)?;
            println!("{}", "✓ Configuration directory removed".green());
        }
    }

    println!();
    println!("{}", "✓ Reset completed successfully!".green().bold());
    println!("Run 'tkit init' to set up a fresh configuration.");

    Ok(())
}

async fn auto_sync_if_enabled(config: &Config) -> Result<()> {
    if config.should_auto_sync() {
        println!("{}", "🔄 Auto-syncing to GitHub...".blue().dimmed());
        if let Err(e) = push_config_to_github_silent().await {
            println!(
                "{}",
                format!("⚠️  Auto-sync failed: {}", e).yellow().dimmed()
            );
        } else {
            println!("{}", "✓ Auto-sync completed".green().dimmed());
        }
    }
    Ok(())
}

pub async fn push_config_to_github_silent() -> Result<()> {
    let config = Config::load()?;

    let repo = config
        .sync
        .repo
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub sync not configured"))?;
    let token = config
        .sync
        .token
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub token not found"))?;

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
            "Auto-sync tkit config - {}",
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
        // Update last sync time
        let mut updated_config = config;
        updated_config.sync.last_sync = Some(chrono::Utc::now().to_rfc3339());
        updated_config.save()?;
        Ok(())
    } else {
        let error_text = response.text().await?;
        Err(anyhow!("Failed to push to GitHub: {}", error_text))
    }
}

pub async fn create_github_repo(name: &str, private: bool) -> Result<()> {
    let config = Config::load()?;

    let token =
        config.sync.token.as_ref().ok_or_else(|| {
            anyhow!("GitHub token not found. Run 'tkit sync setup <repo>' first.")
        })?;

    let client = reqwest::Client::new();
    let url = "https://api.github.com/user/repos";

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("tkit/0.1.0"));

    let request_body = CreateRepoRequest {
        name: name.to_string(),
        description: Some(format!("TKIT configuration repository for {}", name)),
        private,
        auto_init: true,
    };

    let response = client
        .post(url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let repo: GitHubRepo = response.json().await?;
        println!(
            "{}",
            format!("✓ Repository '{}' created successfully!", repo.full_name)
                .green()
                .bold()
        );
        println!("  URL: {}", repo.html_url);
        println!("  Clone URL: {}", repo.clone_url);

        // Update config with new repo
        let mut updated_config = config;
        updated_config.sync.repo = Some(repo.full_name.clone());
        updated_config.save()?;

        println!("  Automatically configured for sync with this repository.");
    } else {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to create repository: {}", error_text));
    }

    Ok(())
}


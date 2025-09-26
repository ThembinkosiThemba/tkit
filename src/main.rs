// > manual description for installing tools
// < after update commad, it should automatically save when creating a new tool, instead of waiting for me to press enter
// 
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use colored::*;
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
    /// Initialize the tkit configuration
    Init,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ToolConfig {
    name: String,
    description: Option<String>,
    install_commands: Vec<String>,
    remove_commands: Vec<String>,
    update_commands: Vec<String>,
    #[serde(default)]
    installed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    tools: HashMap<String, ToolConfig>,
}

impl Config {
    fn new() -> Self {
        Self {
            tools: HashMap::new(),
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

fn add_tool(tool_name: &str) -> Result<()> {
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
    println!("Enter commands for each action (empty line to finish):");

    let install_commands = read_commands("Install")?;
    let remove_commands = read_commands("Remove")?;
    let update_commands = read_commands("Update")?;

    print!("Description (optional): ");
    let mut description = String::new();
    std::io::stdin().read_line(&mut description)?;
    let description = description.trim();
    let description = if description.is_empty() {
        None
    } else {
        Some(description.to_string())
    };

    let tool_config = ToolConfig {
        name: tool_name.to_string(),
        description,
        install_commands,
        remove_commands,
        update_commands,
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
        Commands::Init => init_config(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}

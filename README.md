# TKIT - Customizable Tool Manager

A Rust CLI tool that works like a package manager with customizable install scripts. Similar to `make`, it allows you to define complex installation, removal, and update procedures for any tool through simple YAML configuration.

## Installation

```bash
cargo install tkit
```

Or build from source:

```bash
git clone https://github.com/ThembinkosiThemba/tkit
cd tkit
cargo build --release
sudo cp target/release/tkit /usr/local/bin/
```

## Quick Start

### Interactive Setup

TKIT includes a comprehensive setup wizard that guides you through the initial configuration:

```bash
tkit init
```

The setup wizard will help you:
1. **Add Essential Tools** - Choose from curated tools (git, docker, node, python)
2. **Configure GitHub Sync** - Automatically create repositories or use existing ones
3. **Set Auto-Sync** - Choose between manual or automatic synchronization
4. **Add Custom Tools** - Create your first custom tool configuration

### Manual Setup

If you prefer manual setup:

```bash
# Initialize with example tools
tkit init

# List available tools
tkit list

# Install a tool
tkit install node

# Add a custom tool
tkit add mytool
```

### Reset Configuration

Start fresh by clearing all configuration:

```bash
tkit reset
```

**Warning**: This permanently deletes all tools, settings, and sync configuration.

## Commands

- `tkit install <tool>` - Install a tool using its defined install commands
- `tkit remove <tool>` - Remove a tool using its defined remove commands
- `tkit update <tool>` - Update a tool using its defined update commands
- `tkit run <tool>` - Run a tool using its defined run commands
- `tkit list` - List all available tools and their status
- `tkit add <tool>` - Add a new tool configuration interactively
- `tkit delete <tool>` - Delete a tool configuration
- `tkit examples` - Show examples of tool configurations
- `tkit search local <query>` - Search installed tools locally with fuzzy matching
- `tkit search remote <query>` - Search remote package registries (apt, snap, cargo)
- `tkit search all <query>` - Search both local and remote tools- `tkit init` - Interactive setup wizard to initialize configuration
- `tkit reset` - Reset configuration (clear all tools and settings)
- `tkit sync setup <repo>` - Setup GitHub integration for syncing configs
- `tkit sync create-repo <name>` - Create a new GitHub repository
- `tkit sync list-repos` - List your GitHub repositories
- `tkit sync push` - Push local config to GitHub
- `tkit sync pull` - Pull config from GitHub
- `tkit sync status` - Show sync status

## Configuration

Tools are configured in `~/.config/tkit/config.yaml`. Each tool can have:

- **install_commands**: List of commands to install the tool
- **remove_commands**: List of commands to remove the tool
- **update_commands**: List of commands to update the tool
- **run_commands**: List of commands to run the tool
- **description**: Description of the tool

### Example Configuration

```yaml
tools:
  node:
    name: node
    description: Node.js runtime
    install_commands:
      - curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
      - sudo apt-get install -y nodejs
    remove_commands:
      - sudo apt-get remove -y nodejs
    update_commands:
      - sudo apt-get update
      - sudo apt-get upgrade -y nodejs
    run_commands:
      - node --version
      - npm --version
    installed: false

  rust:
    name: rust
    description: Rust programming language
    install_commands:
      - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      - source ~/.cargo/env
    remove_commands:
      - rustup self uninstall -y
    update_commands:
      - rustup update
    run_commands:
      - rustc --version
      - cargo --version
    installed: false
```

## GitHub Sync

TKIT supports syncing your tool configurations with GitHub for backup and sharing across machines.

### Setup GitHub Integration

1. **Option A: Create repository automatically**
   ```bash
   # Create and configure a new repository
   tkit sync create-repo my-tkit-configs --private
   ```

2. **Option B: Use existing repository**
   ```bash
   # List your repositories
   tkit sync list-repos
   
   # Configure sync with existing repo
   tkit sync setup username/existing-repo --token ghp_xxxxx
   ```

3. **Option C: Manual setup**
   - Create a GitHub repository manually
   - Generate a personal access token with `repo` permissions
   - Configure sync:

```bash
# Setup with token as argument
tkit sync setup username/my-tkit-configs --token ghp_xxxxx

# Or setup interactively (token input hidden)
tkit sync setup username/my-tkit-configs
```

### Sync Commands

```bash
# Push your config to GitHub
tkit sync push

# Pull config from GitHub
tkit sync pull

# Check sync status (includes auto-sync status)
tkit sync status
```

### Auto-Sync Feature

TKIT can automatically sync your configuration to GitHub whenever you make changes:

- **Enabled**: Changes are automatically pushed to GitHub after adding, deleting, installing, or removing tools
- **Disabled**: Manual sync using `tkit sync push`

Auto-sync is configured during the initial setup wizard or can be enabled by editing your configuration file.

### Example Workflow

Setting up a new machine:
```bash
# 1. Initialize tkit
tkit init

# 2. Configure GitHub sync  
tkit sync setup username/my-configs --token ghp_xxxxx

# 3. Pull your existing configuration
tkit sync pull

# 4. Install tools
tkit install docker
tkit install node
```

## Search Functionality

TKIT includes powerful search capabilities with fuzzy matching to help you find tools quickly.

### Search Commands

```bash
# Search for tools installed on your system
tkit search local python
tkit search local code

# Search remote package registries
tkit search remote docker
tkit search remote node

# Search everything (local + remote)
tkit search all git
```

### Search Features

- **Fuzzy Matching**: Find tools even with typos (e.g., "pytho" will match "python")
- **Multiple Package Managers**: Searches apt, snap, and cargo registries
- **System Detection**: Shows installed binaries with version info and install path
- **Package Manager Detection**: Identifies how tools were installed
- **Configured Tool Search**: Searches your TKIT tool configurations

### Search Example Output

```bash
$ tkit search local python

Searching for 'python' in local system...

Configured Tools:
  ✓ python - Python programming language

System Binaries:
  ✓ python3 - /usr/bin/python3 (Python 3.12.3)
  ✗ python - not installed
```

## Examples Command

Get inspired with curated tool configurations:

```bash
# Show all examples
tkit examples

# Categories include:
# - Development Tools (VS Code, Git, Docker)
# - Programming Languages (Python, Rust, Go, Node.js)
# - Utilities (curl commands, system info)
# - Web Development (nginx, databases)
# - DevOps Tools (kubectl, terraform)
```

## Use Cases

- **Development Environment Setup**: Install language runtimes, databases, tools
- **Server Provisioning**: Automate installation of services and dependencies
- **Personal Tool Management**: Keep track of installed tools and their versions
- **Team Onboarding**: Share consistent installation procedures
- **Cross-Platform Scripts**: Define platform-specific installation commands
- **Configuration Backup**: Sync configs with GitHub for backup and sharing

## Examples

### Adding a Complex Tool

```bash
tkit add kubernetes
# Follow prompts to add install/remove/update/run commands
```

### Adding Custom Tools

You can add any custom tool or command:

```bash
# Add a custom tool for VS Code
tkit add vscode
# Description: Visual Studio Code editor
# Install commands: 
# 1: wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
# 2: sudo install -o root -g root -m 644 packages.microsoft.gpg /etc/apt/trusted.gpg.d/
# 3: sudo apt-get update && sudo apt-get install code
# Remove commands:
# 1: sudo apt-get remove code
# Update commands:
# 1: sudo apt-get update && sudo apt-get upgrade code
# Run commands:
# 1: code

# Add a utility tool
tkit add curl-example  
# Description: Curl example website
# Run commands:
# 1: curl -s https://httpbin.org/json
```

### Running Tools

```bash
# Run a tool's defined commands
tkit run vscode        # Opens VS Code
tkit run curl-example  # Executes curl command
tkit run docker        # Shows Docker version and running containers
```

### Multi-Step Installation Example

When adding a tool like Docker, you might define:

Install commands:
1. `curl -fsSL https://get.docker.com -o get-docker.sh`
2. `sudo sh get-docker.sh`
3. `sudo usermod -aG docker $USER`
4. `rm get-docker.sh`

Remove commands:
1. `sudo apt-get remove -y docker docker-engine docker.io containerd runc`
2. `sudo rm -rf /var/lib/docker`

Update commands:
1. `sudo apt-get update`
2. `sudo apt-get upgrade -y docker-ce`

## Features

- ✅ Interactive tool addition
- ✅ Command execution tracking
- ✅ Installation status tracking
- ✅ Colored output for better UX
- ✅ Error handling with descriptive messages
- ✅ Cross-platform configuration storage
- ✅ YAML-based configuration
- ✅ Example tools included

## Publishing to crates.io

1. Update version in `Cargo.toml`
2. Build and test: `cargo build --release && cargo test`
3. Publish: `cargo publish`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

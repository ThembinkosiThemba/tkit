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

1. Initialize configuration:
```bash
tkit init
```

2. List available tools:
```bash
tkit list
```

3. Install a tool:
```bash
tkit install node
```

4. Add a custom tool:
```bash
tkit add mytool
```

## Commands

- `tkit install <tool>` - Install a tool using its defined install commands
- `tkit remove <tool>` - Remove a tool using its defined remove commands
- `tkit update <tool>` - Update a tool using its defined update commands
- `tkit run <tool>` - Run a tool using its defined run commands
- `tkit list` - List all available tools and their status
- `tkit add <tool>` - Add a new tool configuration interactively
- `tkit delete <tool>` - Delete a tool configuration
- `tkit init` - Initialize configuration with example tools
- `tkit sync setup <repo>` - Setup GitHub integration for syncing configs
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

1. Create a GitHub repository for your configs
2. Generate a personal access token with `repo` permissions
3. Configure sync:

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

# Check sync status
tkit sync status
```

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

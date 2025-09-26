use anyhow::Result;
use colored::*;

pub fn show_examples() -> Result<()> {
    println!("{}", "Tool Configuration Examples:".blue().bold());
    println!();

    println!("{}", "Development Tools:".cyan().bold());

    println!("  {}", "Visual Studio Code:".green());
    println!("    tkit add vscode");
    println!("    Description: Visual Studio Code editor");
    println!(
        "    Install: wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg"
    );
    println!("    Run: code");
    println!();

    println!("  {}", "Git:".green());
    println!("    tkit add git");
    println!("    Description: Version control system");
    println!("    Install: sudo apt-get update && sudo apt-get install -y git");
    println!("    Run: git --version");
    println!();

    println!("  {}", "Docker:".green());
    println!("    tkit add docker");
    println!("    Description: Container platform");
    println!(
        "    Install: curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh"
    );
    println!("    Run: docker --version");
    println!();

    println!("{}", "Programming Languages:".cyan().bold());

    println!("  {}", "Python:".green());
    println!("    tkit add python");
    println!("    Description: Python programming language");
    println!("    Install: sudo apt-get update && sudo apt-get install -y python3 python3-pip");
    println!("    Run: python3 --version");
    println!();

    println!("  {}", "Rust:".green());
    println!("    tkit add rust");
    println!("    Description: Rust programming language");
    println!(
        "    Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    );
    println!("    Run: rustc --version");
    println!();

    println!("  {}", "Go:".green());
    println!("    tkit add golang");
    println!("    Description: Go programming language");
    println!(
        "    Install: wget https://golang.org/dl/go1.21.0.linux-amd64.tar.gz && sudo tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz"
    );
    println!("    Run: go version");
    println!();

    println!("{}", "Utilities:".cyan().bold());

    println!("  {}", "Curl Example:".green());
    println!("    tkit add curl-test");
    println!("    Description: Test HTTP requests with curl");
    println!("    Run: curl -s https://httpbin.org/json");
    println!();

    println!("  {}", "System Info:".green());
    println!("    tkit add sysinfo");
    println!("    Description: Show system information");
    println!("    Run: uname -a && df -h && free -h");
    println!();

    println!("{}", "Web Development:".cyan().bold());

    println!("  {}", "Node.js:".green());
    println!("    tkit add nodejs");
    println!("    Description: Node.js runtime");
    println!(
        "    Install: curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash - && sudo apt-get install -y nodejs"
    );
    println!("    Run: node --version && npm --version");
    println!();

    println!("  {}", "Nginx:".green());
    println!("    tkit add nginx");
    println!("    Description: Web server");
    println!("    Install: sudo apt-get update && sudo apt-get install -y nginx");
    println!("    Run: nginx -v");
    println!();

    println!("{}", "DevOps Tools:".cyan().bold());

    println!("  {}", "Kubernetes kubectl:".green());
    println!("    tkit add kubectl");
    println!("    Description: Kubernetes command-line tool");
    println!(
        "    Install: curl -LO https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl && sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl"
    );
    println!("    Run: kubectl version --client");
    println!();

    println!("  {}", "Terraform:".green());
    println!("    tkit add terraform");
    println!("    Description: Infrastructure as Code tool");
    println!(
        "    Install: wget https://releases.hashicorp.com/terraform/1.5.0/terraform_1.5.0_linux_amd64.zip && unzip terraform_1.5.0_linux_amd64.zip && sudo mv terraform /usr/local/bin/"
    );
    println!("    Run: terraform version");
    println!();

    println!("{}", "Usage:".yellow().bold());
    println!("  Copy any example above and run the commands to add tools to your configuration.");
    println!("  You can modify the install, remove, update, and run commands as needed.");
    println!("  Run 'tkit list' to see your configured tools.");

    Ok(())
}

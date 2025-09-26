use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolConfig {
    pub name: String,
    pub description: Option<String>,
    pub install_commands: Vec<String>,
    pub remove_commands: Vec<String>,
    pub update_commands: Vec<String>,
    #[serde(default)]
    pub run_commands: Vec<String>,
    #[serde(default)]
    pub installed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub tools: HashMap<String, ToolConfig>,
    #[serde(default)]
    pub sync: SyncConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigWithSync {
    pub tools: HashMap<String, ToolConfig>,
    pub sync: SyncConfig,
}

impl From<Config> for ConfigWithSync {
    fn from(config: Config) -> Self {
        Self {
            tools: config.tools,
            sync: config.sync,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SyncConfig {
    pub repo: Option<String>,
    pub token: Option<String>,
    pub last_sync: Option<String>,
    #[serde(default)]
    pub auto_sync: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            sync: SyncConfig::default(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        if !config_path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        self.save_to_path(&config_path)
    }

    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_tool(&mut self, name: &str, tool_config: ToolConfig) -> Result<()> {
        if self.tools.contains_key(name) {
            return Err(anyhow!("Tool '{}' already exists.", name));
        }
        self.tools.insert(name.to_string(), tool_config);
        Ok(())
    }

    pub fn remove_tool(&mut self, name: &str) -> Result<bool> {
        Ok(self.tools.remove(name).is_some())
    }

    pub fn get_tool(&self, name: &str) -> Option<&ToolConfig> {
        self.tools.get(name)
    }

    pub fn get_tool_mut(&mut self, name: &str) -> Option<&mut ToolConfig> {
        self.tools.get_mut(name)
    }

    pub fn list_tools(&self) -> Vec<(&String, &ToolConfig)> {
        self.tools.iter().collect()
    }

    pub fn should_auto_sync(&self) -> bool {
        self.sync.auto_sync && self.sync.repo.is_some() && self.sync.token.is_some()
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;
    Ok(config_dir.join("tkit").join("config.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.tools.is_empty());
        assert!(config.sync.repo.is_none());
    }

    #[test]
    fn test_add_tool() {
        let mut config = Config::new();
        let tool_config = ToolConfig {
            name: "test".to_string(),
            description: Some("Test tool".to_string()),
            install_commands: vec!["echo install".to_string()],
            remove_commands: vec!["echo remove".to_string()],
            update_commands: vec!["echo update".to_string()],
            run_commands: vec!["echo run".to_string()],
            installed: false,
        };

        assert!(config.add_tool("test", tool_config).is_ok());
        assert!(config.tools.contains_key("test"));
    }

    #[test]
    fn test_add_duplicate_tool() {
        let mut config = Config::new();
        let tool_config = ToolConfig {
            name: "test".to_string(),
            description: Some("Test tool".to_string()),
            install_commands: vec![],
            remove_commands: vec![],
            update_commands: vec![],
            run_commands: vec![],
            installed: false,
        };

        config.add_tool("test", tool_config.clone()).unwrap();
        assert!(config.add_tool("test", tool_config).is_err());
    }

    #[test]
    fn test_remove_tool() {
        let mut config = Config::new();
        let tool_config = ToolConfig {
            name: "test".to_string(),
            description: Some("Test tool".to_string()),
            install_commands: vec![],
            remove_commands: vec![],
            update_commands: vec![],
            run_commands: vec![],
            installed: false,
        };

        config.add_tool("test", tool_config).unwrap();
        assert!(config.remove_tool("test").unwrap());
        assert!(!config.tools.contains_key("test"));
    }

    #[test]
    fn test_remove_nonexistent_tool() {
        let mut config = Config::new();
        assert!(!config.remove_tool("nonexistent").unwrap());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut config = Config::new();
        let tool_config = ToolConfig {
            name: "test".to_string(),
            description: Some("Test tool".to_string()),
            install_commands: vec!["install cmd".to_string()],
            remove_commands: vec!["remove cmd".to_string()],
            update_commands: vec!["update cmd".to_string()],
            run_commands: vec!["run cmd".to_string()],
            installed: true,
        };

        config.add_tool("test", tool_config).unwrap();
        config.save_to_path(&config_path).unwrap();

        let loaded_config = Config::load_from_path(&config_path).unwrap();
        assert_eq!(loaded_config.tools.len(), 1);
        assert!(loaded_config.tools.contains_key("test"));
        
        let tool = loaded_config.get_tool("test").unwrap();
        assert_eq!(tool.name, "test");
        assert_eq!(tool.description, Some("Test tool".to_string()));
        assert_eq!(tool.install_commands, vec!["install cmd"]);
        assert!(tool.installed);
    }

    #[test]
    fn test_load_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.yaml");
        
        let config = Config::load_from_path(&config_path).unwrap();
        assert!(config.tools.is_empty());
    }
}
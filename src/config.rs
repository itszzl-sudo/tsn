use anyhow::Result;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: ProjectInfo,
    #[serde(default)]
    pub compile: CompileConfig,
    #[serde(default)]
    pub extensions: ExtensionsConfig,
    #[serde(default)]
    pub link: LinkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileConfig {
    #[serde(default = "default_input")]
    pub input: String,
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default)]
    pub target: Option<String>,
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self {
            input: default_input(),
            output: default_output(),
            target: None,
        }
    }
}

fn default_input() -> String {
    "src/main.ts".to_string()
}

fn default_output() -> String {
    "a.exe".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionsConfig {
    #[serde(default = "default_true")]
    pub auto_discover: bool,
    #[serde(default)]
    pub items: Vec<ExtensionItem>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for ExtensionsConfig {
    fn default() -> Self {
        Self {
            auto_discover: true,
            items: Vec::new(),
            exclude: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionItem {
    pub path: Option<String>,
    pub crate_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkConfig {
    #[serde(default = "default_linker")]
    pub linker: String,
    #[serde(default)]
    pub flags: Vec<String>,
}

fn default_linker() -> String {
    "auto".to_string()
}

impl ProjectConfig {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn load_from_dir(dir: &std::path::Path) -> Result<Self> {
        let config_path = dir.join("ts-native.toml");
        Self::load(&config_path)
    }
    
    pub fn has_explicit_extensions(&self) -> bool {
        !self.extensions.items.is_empty()
    }
    
    pub fn should_auto_discover(&self) -> bool {
        self.extensions.auto_discover && !self.has_explicit_extensions()
    }
    
    pub fn get_excluded_extensions(&self) -> &[String] {
        &self.extensions.exclude
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();
        assert_eq!(config.compile.input, "src/main.ts");
        assert!(config.extensions.auto_discover);
        assert!(!config.has_explicit_extensions());
    }
    
    #[test]
    fn test_parse_config() {
        let config_content = r#"
[project]
name = "my-app"
version = "0.1.0"

[compile]
input = "src/app.ts"
output = "dist/app.exe"

[extensions]
auto_discover = true
exclude = ["ts-native-fs"]
"#;
        let config: ProjectConfig = toml::from_str(config_content).unwrap();
        assert_eq!(config.project.name, "my-app");
        assert_eq!(config.compile.input, "src/app.ts");
        assert!(config.should_auto_discover());
        assert!(config.get_excluded_extensions().contains(&"ts-native-fs".to_string()));
    }
}

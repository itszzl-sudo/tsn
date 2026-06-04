#[allow(dead_code)]
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub package: ExtensionPackage,
    pub functions: HashMap<String, ExternalFunction>,
    #[serde(default)]
    pub link: LinkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalFunction {
    pub args: Vec<ArgType>,
    pub ret: RetType,
    pub impl_name: String,
    #[serde(default)]
    pub is_property: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArgType {
    Number,
    String,
    Boolean,
    Any,
    Array,
    Object,
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RetType {
    Number,
    String,
    Boolean,
    Any,
    Array,
    Object,
    Void,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkConfig {
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub lib: Option<String>,
    #[serde(default)]
    pub libs: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub ffi: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoToml {
    #[serde(default)]
    pub dependencies: HashMap<String, toml::Value>,
    #[serde(default)]
    pub package: CargoPackage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CargoPackage {
    #[serde(default)]
    pub metadata: CargoMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CargoMetadata {
    #[serde(default, rename = "ts-native")]
    pub ts_native: Option<TsNativeMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsNativeMetadata {
    #[serde(default)]
    pub plugin: bool,
    #[serde(default)]
    pub manifest: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExtensionRegistry {
    pub extensions: Vec<Extension>,
    pub function_map: HashMap<String, String>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            function_map: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, extension: Extension) {
        for (ts_name, func) in &extension.functions {
            self.function_map.insert(ts_name.clone(), func.impl_name.clone());
        }
        self.extensions.push(extension);
    }
    
    pub fn get_c_name(&self, ts_name: &str) -> Option<&String> {
        self.function_map.get(ts_name)
    }
    
    pub fn is_external_function(&self, ts_name: &str) -> bool {
        self.function_map.contains_key(ts_name)
    }
}

pub fn parse_manifest(path: &Path) -> Result<Extension> {
    let content = std::fs::read_to_string(path)?;
    let manifest: Extension = toml::from_str(&content)?;
    Ok(manifest)
}

pub fn parse_cargo_toml(path: &Path) -> Result<CargoToml> {
    let content = std::fs::read_to_string(path)?;
    let cargo: CargoToml = toml::from_str(&content)?;
    Ok(cargo)
}

pub fn find_plugin_crates(cargo_toml: &CargoToml) -> Vec<String> {
    cargo_toml
        .dependencies
        .keys()
        .filter(|name| {
            let dep_path = find_crate_path(name);
            if let Ok(path) = dep_path {
                let cargo_path = path.join("Cargo.toml");
                if cargo_path.exists() {
                    if let Ok(dep_cargo) = parse_cargo_toml(&cargo_path) {
                        if let Some(ref ts_meta) = dep_cargo.package.metadata.ts_native {
                            if ts_meta.plugin {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        })
        .cloned()
        .collect()
}

pub fn load_extension_from_path(path: &Path) -> Result<Extension> {
    let cargo_path = path.join("Cargo.toml");
    
    if cargo_path.exists() {
        let cargo = parse_cargo_toml(&cargo_path)?;
        
        if let Some(ref ts_meta) = cargo.package.metadata.ts_native {
            if let Some(ref manifest_rel) = ts_meta.manifest {
                let manifest_path = path.join(manifest_rel);
                if manifest_path.exists() {
                    return parse_manifest(&manifest_path);
                }
            }
        }
    }
    
    let manifest_path = path.join("ts-native.toml");
    if manifest_path.exists() {
        return parse_manifest(&manifest_path);
    }
    
    bail!("No manifest found in {:?} (tried Cargo.toml metadata and ts-native.toml)", path)
}

pub fn load_extension_from_crate(crate_name: &str) -> Result<Extension> {
    let crate_path = find_crate_path(crate_name)?;
    load_extension_from_path(&crate_path)
}

fn find_crate_path(crate_name: &str) -> Result<std::path::PathBuf> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let local_path = std::path::PathBuf::from(&manifest_dir)
            .join(crate_name);
        if local_path.join("Cargo.toml").exists() || local_path.join("ts-native.toml").exists() {
            return Ok(local_path);
        }
    }
    
    let current_dir = std::env::current_dir()?;
    let local_path = current_dir.join(crate_name);
    if local_path.join("Cargo.toml").exists() || local_path.join("ts-native.toml").exists() {
        return Ok(local_path);
    }
    
    let parent_dir = current_dir.parent().unwrap_or(&current_dir);
    let sibling_path = parent_dir.join(crate_name);
    if sibling_path.join("Cargo.toml").exists() || sibling_path.join("ts-native.toml").exists() {
        return Ok(sibling_path);
    }
    
    bail!("Crate {} not found. Tried:\n  - ./{}/\n  - ../{}/", crate_name, crate_name, crate_name)
}

pub fn discover_extensions_from_cargo(cargo_toml_path: &Path) -> Result<ExtensionRegistry> {
    let mut registry = ExtensionRegistry::new();
    
    if !cargo_toml_path.exists() {
        return Ok(registry);
    }
    
    let cargo_toml = parse_cargo_toml(cargo_toml_path)?;
    let plugin_crates = find_plugin_crates(&cargo_toml);
    
    for crate_name in plugin_crates {
        match load_extension_from_crate(&crate_name) {
            Ok(ext) => {
                println!("  加载扩展包: {} ({})", ext.package.name, crate_name);
                registry.register(ext);
            }
            Err(e) => {
                eprintln!("  ⚠️  加载扩展包 {} 失败: {}", crate_name, e);
            }
        }
    }
    
    Ok(registry)
}

pub fn discover_extensions_from_tsnp(tsnp_dir: &Path) -> Result<ExtensionRegistry> {
    discover_extensions_from_tsnp_filtered(tsnp_dir, None)
}

/// Scan `tsnp_dir` for plugin subdirectories.  
/// When `filter` is `Some(list)`, only subdirectories whose name appears in
/// the list are loaded.  When `None`, **all** subdirectories are loaded
/// (backward-compatible behaviour).
pub fn discover_extensions_from_tsnp_filtered(
    tsnp_dir: &Path,
    filter: Option<&[String]>,
) -> Result<ExtensionRegistry> {
    let mut registry = ExtensionRegistry::new();

    if !tsnp_dir.exists() {
        return Ok(registry);
    }

    let entries = std::fs::read_dir(tsnp_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read tsnp directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| anyhow::anyhow!("Failed to read entry: {}", e))?;
        let plugin_dir = entry.path();

        if plugin_dir.is_dir() {
            // Apply filter if present
            if let Some(allowed) = filter {
                let dir_name = plugin_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !allowed.iter().any(|a| a == &dir_name) {
                    continue;
                }
            }

            let manifest_path = plugin_dir.join("ts-native.toml");
            if manifest_path.exists() {
                match parse_manifest(&manifest_path) {
                    Ok(ext) => {
                        println!("  \u{1f50c} \u{52a0}\u{8f7d}\u{63d2}\u{4ef6}: {} from {:?}", ext.package.name, plugin_dir);
                        registry.register(ext);
                    }
                    Err(e) => {
                        eprintln!("  \u{26a0}\u{fe0f}  \u{52a0}\u{8f7d}\u{63d2}\u{4ef6} {:?} \u{5931}\u{8d25}: {}", plugin_dir, e);
                    }
                }
            }
        }
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_manifest() {
        let manifest_content = r#"
[package]
name = "stdlib"
version = "0.1.0"

[functions]
"Math.sin" = { args = ["number"], ret = "number", impl_name = "js_math_sin" }

[link]
runtime = "runtime.c"
"#;
        let ext: Extension = toml::from_str(manifest_content).unwrap();
        assert_eq!(ext.package.name, "stdlib");
        assert!(ext.functions.contains_key("Math.sin"));
    }
}

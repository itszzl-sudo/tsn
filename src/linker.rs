use anyhow::Result;
use std::process::Command;

pub struct Linker {
    output_name: String,
}

impl Linker {
    pub fn new(output_name: &str) -> Self {
        Self {
            output_name: output_name.to_string(),
        }
    }
    
    pub fn link(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        #[cfg(target_os = "windows")]
        {
            self.link_windows(object_files)
        }
        
        #[cfg(target_os = "linux")]
        {
            self.link_linux(object_files)
        }
        
        #[cfg(target_os = "macos")]
        {
            self.link_macos(object_files)
        }
    }
    
    #[cfg(target_os = "windows")]
    fn link_windows(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        let jade_link = Self::find_jade_linker();
        
        let mut link_args = vec![
            "/NOLOGO".to_string(),
            "/ENTRY:mainCRTStartup".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
            format!("/OUT:{}", self.output_name),
        ];
        
        for obj in object_files {
            link_args.push((*obj).to_string());
        }
        
        let crt_start_o = Self::find_crt_start();
        if let Some(crt) = &crt_start_o {
            link_args.push(crt.clone());
        }
        
        let ext_libs = Self::find_extension_libs();
        if !ext_libs.is_empty() {
            eprintln!("[link] Found {} extension libs", ext_libs.len());
        }
        for lib in &ext_libs {
            link_args.push(lib.clone());
        }
        
        let linker = jade_link.clone().unwrap_or_else(|| "link.exe".to_string());
        
        let mut cmd = Command::new(&linker);
        cmd.args(&link_args);
        
        if let Some(linker_path) = &jade_link {
            if let Some(parent) = std::path::Path::new(linker_path).parent() {
                let path_env = std::env::var("PATH").unwrap_or_default();
                let new_path = format!("{};{}", parent.to_string_lossy(), path_env);
                cmd.env("PATH", new_path);
            }
        }
        
        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute linker '{}': {}", linker, e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("Link failed:\n  Linker: {}\n  Args: {:?}\n  Stderr: {}\n  Stdout: {}", 
                linker, link_args, stderr, stdout);
        }
        
        std::fs::read(&self.output_name)
            .map_err(|e| anyhow::anyhow!("Failed to read output: {}", e))
    }
    
    fn normalize_path(path: std::path::PathBuf) -> Option<String> {
        let canonical = path.canonicalize().ok()?;
        let path_str = canonical.to_string_lossy().to_string();
        if path_str.starts_with("\\\\?\\") {
            Some(path_str[4..].to_string())
        } else {
            Some(path_str)
        }
    }
    
    fn find_jade_linker() -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        let candidates = vec![
            cwd.join("javascript-web-to-rust-native/packages/jade/toolchain/win32-x64-msvc/link.exe"),
            cwd.join("../javascript-web-to-rust-native/packages/jade/toolchain/win32-x64-msvc/link.exe"),
            cwd.join("../../javascript-web-to-rust-native/packages/jade/toolchain/win32-x64-msvc/link.exe"),
        ];
        
        if let Some(path) = std::env::var("TSN_LINKER").ok() {
            let p = std::path::PathBuf::from(&path);
            if p.exists() { return Some(path); }
        }
        
        for path in candidates {
            if path.exists() {
                return Self::normalize_path(path);
            }
        }
        None
    }
    
    fn find_crt_start() -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        
        if let Some(path) = std::env::var("TSN_CRT_START").ok() {
            let p = std::path::PathBuf::from(&path);
            if p.exists() { return Some(path); }
        }
        
        let runtime_dir = cwd.join("ts-native-runtime/target/release/build");
        if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
            for entry in entries.flatten() {
                let start_path = entry.path().join("out/ea708c7824d36062-crt_start.o");
                if start_path.exists() {
                    return Self::normalize_path(start_path);
                }
            }
        }
        
        let candidates = vec![
            cwd.join("ts-native-runtime/target/release/build").join("crt_start.o"),
            cwd.join("../ts-native-runtime/target/release/build").join("crt_start.o"),
        ];
        
        for path in candidates {
            if path.exists() {
                return Self::normalize_path(path);
            }
        }

        None
    }
    
    fn find_extension_libs() -> Vec<String> {
        let mut libs = Vec::new();
        
        let cwd = if let Ok(c) = std::env::current_dir() {
            c
        } else {
            return libs;
        };

        let tsnp_dir = cwd.join("tsnp");
        if tsnp_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&tsnp_dir) {
                for entry in entries.flatten() {
                    let toml_path = entry.path().join("ts-native.toml");
                    if toml_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&toml_path) {
                            if let Ok(ext) = content.parse::<toml::Value>() {
                                if let Some(link) = ext.get("link") {
                                    if let Some(lib) = link.get("lib").and_then(|v| v.as_str()) {
                                        for search_dir in [&cwd, cwd.parent().unwrap_or(&cwd)] {
                                            let lib_path = search_dir.join(lib);
                                            if lib_path.exists() {
                                                eprintln!("[link] Found lib from tsnp: {:?}", lib_path);
                                                if let Some(normalized) = Self::normalize_path(lib_path) {
                                                    libs.push(normalized);
                                                }
                                                break;
                                            }
                                        }
                                    }
                                    if let Some(libs_arr) = link.get("libs").and_then(|v| v.as_array()) {
                                        for lib_val in libs_arr {
                                            if let Some(lib) = lib_val.as_str() {
                                                for search_dir in [&cwd, cwd.parent().unwrap_or(&cwd)] {
                                                    let lib_path = search_dir.join(lib);
                                                    if lib_path.exists() {
                                                        eprintln!("[link] Found lib from tsnp: {:?}", lib_path);
                                                        if let Some(normalized) = Self::normalize_path(lib_path) {
                                                            libs.push(normalized);
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let candidates = vec![
            cwd.join("ts-native-stdlib/target/release/ts_native_stdlib.lib"),
            cwd.join("../ts-native-stdlib/target/release/ts_native_stdlib.lib"),
            cwd.join("ts-native-extension-dom/target/release/ts_native_extension_dom.lib"),
            cwd.join("../ts-native-extension-dom/target/release/ts_native_extension_dom.lib"),
        ];
        
        for lib_path in candidates {
            if lib_path.exists() {
                eprintln!("[link] Found lib: {:?}", lib_path);
                if let Some(normalized) = Self::normalize_path(lib_path) {
                    libs.push(normalized);
                }
            }
        }
        
        libs
    }
    
    #[cfg(target_os = "linux")]
    fn link_linux(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        let mut ld_args = vec![
            "-o".to_string(),
            self.output_name.clone(),
        ];
        
        for obj in object_files {
            ld_args.push((*obj).to_string());
        }
        
        ld_args.push("-lc".to_string());
        
        let output = Command::new("ld")
            .args(&ld_args)
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("Link failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        std::fs::read(&self.output_name)
            .map_err(|e| anyhow::anyhow!("Failed to read output: {}", e))
    }
    
    #[cfg(target_os = "macos")]
    fn link_macos(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        let mut ld_args = vec![
            "-o".to_string(),
            self.output_name.clone(),
        ];
        
        for obj in object_files {
            ld_args.push((*obj).to_string());
        }
        
        ld_args.push("-lSystem".to_string());
        
        let output = Command::new("ld")
            .args(&ld_args)
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("Link failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        std::fs::read(&self.output_name)
            .map_err(|e| anyhow::anyhow!("Failed to read output: {}", e))
    }
    
    pub fn link_with_gcc(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        let mut gcc_args = vec![
            "-o".to_string(),
            self.output_name.clone(),
        ];
        
        for obj in object_files {
            gcc_args.push((*obj).to_string());
        }
        
        let output = Command::new("gcc")
            .args(&gcc_args)
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("GCC link failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        std::fs::read(&self.output_name)
            .map_err(|e| anyhow::anyhow!("Failed to read output: {}", e))
    }
    
    pub fn link_with_clang(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        let clang_path = "C:\\Program Files\\Microsoft Visual Studio\\18\\Community\\VC\\Tools\\Llvm\\x64\\bin\\clang.exe";
        
        let mut clang_args = vec![
            "-Wl,/ENTRY:_start".to_string(),
            "-o".to_string(),
            self.output_name.clone(),
        ];
        
        for obj in object_files {
            clang_args.push((*obj).to_string());
        }
        
        clang_args.push("-lkernel32".to_string());
        
        let output = Command::new(clang_path)
            .args(&clang_args)
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("Clang link failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        std::fs::read(&self.output_name)
            .map_err(|e| anyhow::anyhow!("Failed to read output: {}", e))
    }
}


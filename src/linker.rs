use anyhow::Result;
use std::process::Command;
use crate::extension::LinkConfig;

pub struct Linker {
    output_name: String,
    link_config: LinkConfig,
}

impl Linker {
    pub fn new(output_name: &str, link_config: LinkConfig) -> Self {
        Self {
            output_name: output_name.to_string(),
            link_config,
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
        let linker = std::env::var("TSN_LINKER").unwrap_or_else(|_| "link.exe".to_string());
        
        // 检查是否有start_nocrt.o，如果有则使用_start作为入口点
        let has_start = object_files.iter().any(|f| f.contains("start_nocrt.o"));
        let entry = if has_start { "_start".to_string() } else { "mainCRTStartup".to_string() };
        
        let mut link_args = vec![
            "/NOLOGO".to_string(),
            format!("/ENTRY:{}", entry),
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
        
        for lib in &self.link_config.libs {
            link_args.push(format!("{}.lib", lib));
        }
        
        // Windows nocrt 模式必需的系统库
        // kernel32: ExitProcess, GetStdHandle, WriteFile, GetProcessHeap, HeapAlloc, HeapReAlloc
        // legacy_stdio_definitions: 提供 C 标准库函数的 DLL 导出桩
        let default_libs = ["kernel32", "shell32", "legacy_stdio_definitions"];
        for lib in &default_libs {
            link_args.push(format!("{}.lib", lib));
        }
        
        for flag in &self.link_config.flags {
            link_args.push(flag.clone());
        }

        
        let mut cmd = Command::new(&linker);
        cmd.args(&link_args);
        
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
    
    fn find_crt_start() -> Option<String> {
        std::env::var("TSN_CRT_START").ok()
    }
    
    #[cfg(target_os = "linux")]
    fn link_linux(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        // Use system cc/clang as linker driver — it handles crt objects,
        // dynamic linker, and library paths correctly across architectures
        let cc = std::env::var("TSN_CC")
            .or_else(|_| std::env::var("CC"))
            .unwrap_or_else(|_| "cc".to_string());
        
        let mut args = vec![
            "-o".to_string(),
            self.output_name.clone(),
            "-no-pie".to_string(),
        ];
        
        for obj in object_files {
            args.push((*obj).to_string());
        }
        
        for lib in &self.link_config.libs {
            args.push(format!("-l{}", lib));
        }
        
        for flag in &self.link_config.flags {
            args.push(flag.clone());
        }
        
        let output = Command::new(&cc)
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("Link failed (cc): {}\n  Stderr: {}", 
                String::from_utf8_lossy(&output.stdout), 
                String::from_utf8_lossy(&output.stderr));
        }
        
        std::fs::read(&self.output_name)
            .map_err(|e| anyhow::anyhow!("Failed to read output: {}", e))
    }
    
    #[cfg(target_os = "macos")]
    fn link_macos(&self, object_files: &[&str]) -> Result<Vec<u8>> {
        // Use system cc as linker driver — it finds the correct SDK path automatically
        let cc = std::env::var("TSN_CC")
            .or_else(|_| std::env::var("CC"))
            .unwrap_or_else(|_| "cc".to_string());
        
        let mut args = vec![
            "-o".to_string(),
            self.output_name.clone(),
        ];
        
        for obj in object_files {
            args.push((*obj).to_string());
        }
        
        for lib in &self.link_config.libs {
            args.push(format!("-l{}", lib));
        }
        
        for flag in &self.link_config.flags {
            args.push(flag.clone());
        }
        
        let output = Command::new(&cc)
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("Link failed (cc): {}\n  Stderr: {}", 
                String::from_utf8_lossy(&output.stdout), 
                String::from_utf8_lossy(&output.stderr));
        }
        
        std::fs::read(&self.output_name)
            .map_err(|e| anyhow::anyhow!("Failed to read output: {}", e))
    }
}

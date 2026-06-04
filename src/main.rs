use anyhow::Result;
use codegen::{CodeGen, HirExpr, BinOp};

mod codegen;
mod runtime;
mod swc_transform;
mod linker;
mod pe_builder;
mod extension;
mod config;
mod syntax_check;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--test" {
        return test_compile();
    }
    
    if args.len() < 2 {
        eprintln!("ts-native has moved to tsn. Install with: cargo install tsn");
        eprintln!("Usage: tsn <input.ts> [-o output]");
        eprintln!("       tsn --test");
        eprintln!("       tsn compile [--config ts-native.toml]");
        std::process::exit(1);
    }
    
    // Parse arguments: ts-native <input.ts> [-o <output>]
    let mut input_file = String::new();
    let mut output_file = String::new();
    let mut skip_check = false;
    let mut gen_syntax_md = false;
    {
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-o" | "--output" => {
                    i += 1;
                    if i < args.len() {
                        output_file = args[i].clone();
                    }
                }
                "--skip-check" => { skip_check = true; }
                "--gen-syntax-md" => { gen_syntax_md = true; }
                arg if !arg.starts_with('-') && input_file.is_empty() => {
                    input_file = arg.to_string();
                }
                _ => {}
            }
            i += 1;
        }
    }

    if gen_syntax_md {
        print!("{}", syntax_check::generate_markdown());
        return Ok(());
    }

    if input_file.is_empty() {
        eprintln!("Error: no input file specified");
        std::process::exit(1);
    }

    if output_file.is_empty() {
        let input_path_buf = std::path::PathBuf::from(&input_file);
        let stem = input_path_buf
            .file_stem()
            .unwrap_or(std::ffi::OsStr::new("a"))
            .to_string_lossy()
            .to_string();
        let dir = input_path_buf
            .parent()
            .unwrap_or(std::path::Path::new("."));
        output_file = dir.join(format!("{}.exe", stem)).to_string_lossy().to_string();
    }

    let mut registry = extension::ExtensionRegistry::new();

    let input_path = std::path::PathBuf::from(&input_file);
    let tsnp_dir = input_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("tsnp");

    if tsnp_dir.exists() {
        match config::FileDepsConfig::load_for_input(&input_path)? {
            Some(deps) => {
                println!("读取依赖声明: {}.toml", input_path.file_name().unwrap_or_default().to_string_lossy());
                println!("  tsnp 依赖: {:?}", deps.dependencies.tsnp);
                registry = extension::discover_extensions_from_tsnp_filtered(
                    &tsnp_dir,
                    Some(&deps.dependencies.tsnp),
                )?;
            }
            None => {
                println!("扫描 tsnp/ 目录（未找到 {}.toml，加载全部）...", input_path.file_name().unwrap_or_default().to_string_lossy());
                registry = extension::discover_extensions_from_tsnp(&tsnp_dir)?;
            }
        }
        println!("加载了 {} 个扩展包", registry.extensions.len());
    }
    
    let source = std::fs::read_to_string(&input_file)?;
    
    if !skip_check {
        match syntax_check::check_syntax(&source) {
            Ok(unsupported) if !unsupported.is_empty() => {
                eprintln!("❌ 检测到不支持的语法:");
                for item in &unsupported {
                    eprintln!("   第 {} 行: {} (ts-native 暂不支持)", item.line, item.name);
                }
                eprintln!("\n提示: 使用 --skip-check 跳过检查（可能导致编译错误）");
                std::process::exit(1);
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("⚠️  语法检查失败: {}", e);
                eprintln!("继续编译（可能不支持部分语法）...");
            }
        }
    }
    
    println!("\n解析 TypeScript: {}", input_file);
    let hir = swc_transform::parse(&source)?;
    
    println!("生成 native 代码...");
    let mut codegen = CodeGen::new();
    let binary = codegen.compile(&hir)?;
    
    let obj_file = if output_file.ends_with(".exe") {
        output_file.replace(".exe", ".o")
    } else if output_file.ends_with(".o") {
        output_file.clone()
    } else {
        format!("{}.o", output_file)
    };
    
    std::fs::write(&obj_file, &binary)?;
    println!("✅ 目标文件: {} ({} bytes)", obj_file, binary.len());
    
    if output_file.ends_with(".exe") || output_file.ends_with(".o") {
        let exe_file = if output_file.ends_with(".exe") {
            output_file.clone()
        } else {
            output_file.replace(".o", ".exe")
        };
        println!("\n尝试链接为可执行文件...");
        
        let link_result = try_link(&obj_file, &exe_file, &registry);
        match link_result {
            Ok(size) => println!("✅ 生成: {} ({} bytes)", exe_file, size),
            Err(e) => println!("⚠️  链接失败（需要安装链接器）: {}", e),
        }
    }
    
    Ok(())
}

fn try_link(obj_file: &str, exe_file: &str, registry: &extension::ExtensionRegistry) -> Result<usize> {
    let mut link_config = extension::LinkConfig::default();
    
    for ext in &registry.extensions {
        for lib in &ext.link.libs {
            if !link_config.libs.contains(lib) {
                link_config.libs.push(lib.clone());
            }
        }
        for flag in &ext.link.flags {
            if !link_config.flags.contains(flag) {
                link_config.flags.push(flag.clone());
            }
        }
    }
    
    let linker_instance = linker::Linker::new(exe_file, link_config);
    
    let has_start = std::path::Path::new("start_nocrt.o").exists();
    let has_runtime = std::path::Path::new("runtime_nocrt.o").exists();
    
    let mut obj_files: Vec<&str> = vec![obj_file];
    if has_start {
        obj_files.push("start_nocrt.o");
    }
    if has_runtime {
        obj_files.push("runtime_nocrt.o");
    }
    
    match linker_instance.link(&obj_files) {
        Ok(result) => return Ok(result.len()),
        Err(e) => anyhow::bail!("MSVC link failed: {:#}", e),
    }
}

fn test_compile() -> Result<()> {
    println!("=== 测试 Cranelift 代码生成 ===\n");
    
    let hir = vec![
        HirExpr::Function {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: vec![
                HirExpr::Return(Some(Box::new(
                    HirExpr::Binary {
                        op: BinOp::Add,
                        left: Box::new(HirExpr::Identifier("a".to_string())),
                        right: Box::new(HirExpr::Identifier("b".to_string())),
                    }
                ))),
            ],
        },
        HirExpr::Function {
            name: "main".to_string(),
            params: vec![],
            body: vec![
                HirExpr::Var {
                    name: "x".to_string(),
                    init: Some(Box::new(HirExpr::Number(10.0))),
                    is_mut: false,
                },
                HirExpr::Var {
                    name: "sum".to_string(),
                    init: Some(Box::new(HirExpr::Number(0.0))),
                    is_mut: true,
                },
                HirExpr::While {
                    cond: Box::new(HirExpr::Binary {
                        op: BinOp::Gt,
                        left: Box::new(HirExpr::Identifier("x".to_string())),
                        right: Box::new(HirExpr::Number(0.0)),
                    }),
                    body: vec![
                        HirExpr::Assign {
                            target: Box::new(HirExpr::Identifier("sum".to_string())),
                            value: Box::new(HirExpr::Binary {
                                op: BinOp::Add,
                                left: Box::new(HirExpr::Identifier("sum".to_string())),
                                right: Box::new(HirExpr::Identifier("x".to_string())),
                            }),
                        },
                        HirExpr::Assign {
                            target: Box::new(HirExpr::Identifier("x".to_string())),
                            value: Box::new(HirExpr::Binary {
                                op: BinOp::Sub,
                                left: Box::new(HirExpr::Identifier("x".to_string())),
                                right: Box::new(HirExpr::Number(1.0)),
                            }),
                        },
                    ],
                },
                HirExpr::Return(Some(Box::new(HirExpr::Identifier("sum".to_string())))),
            ],
        },
    ];
    
    println!("HIR:");
    for expr in &hir {
        println!("  {:?}", expr);
    }
    
    let mut codegen = CodeGen::new();
    let binary = codegen.compile(&hir)?;
    
    std::fs::write("test.o", &binary)?;
    println!("\n✅ 生成 test.o ({} bytes)", binary.len());
    println!("\n函数:");
    println!("  add(a, b) = a + b");
    println!("  main() = sum(1..10) = 55");
    
    Ok(())
}

fn parse_simple(source: &str) -> Result<Vec<HirExpr>> {
    let mut exprs = Vec::new();
    
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("function") {
            let line = line.strip_prefix("function").unwrap().trim();
            if let Some(paren_pos) = line.find('(') {
                let name = line[..paren_pos].trim().to_string();
                let rest = &line[paren_pos+1..];
                if let Some(end_paren) = rest.find(')') {
                    let params_str = &rest[..end_paren];
                    let params: Vec<String> = if params_str.is_empty() {
                        vec![]
                    } else {
                        params_str.split(',').map(|s| s.trim().to_string()).collect()
                    };
                    
                    exprs.push(HirExpr::Function {
                        name,
                        params,
                        body: vec![],
                    });
                }
            }
        }
    }
    
    if exprs.is_empty() {
        exprs.push(HirExpr::Function {
            name: "main".to_string(),
            params: vec![],
            body: vec![HirExpr::Return(Some(Box::new(HirExpr::Number(0.0))))],
        });
    }
    
    Ok(exprs)
}

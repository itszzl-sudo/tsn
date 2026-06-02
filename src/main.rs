use anyhow::Result;
use codegen::CodeGen;
use hir::{HirExpr, BinOp};

mod hir;
mod builtins;
mod codegen;
#[allow(dead_code)]
mod runtime;
mod ts_parser;
mod linker;
#[allow(dead_code)]
mod pe_builder;
mod extension;
#[allow(dead_code)]
mod config;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--test" {
        return test_compile();
    }
    
    if args.len() < 2 {
        eprintln!("Usage: tsn <input.ts> [-o output]");
        eprintln!("       tsn --test");
        eprintln!("       tsn compile [--config ts-native.toml]");
        std::process::exit(1);
    }
    
    let mut registry = extension::ExtensionRegistry::new();
    
    let tsnp_dir = std::path::PathBuf::from("tsnp");
    if tsnp_dir.exists() {
        println!("扫描 tsnp/ 目录...");
        registry = extension::discover_extensions_from_tsnp(&tsnp_dir)?;
        println!("加载了 {} 个扩展包", registry.extensions.len());
    }
    
    let mut input_file: Option<&str> = None;
    let mut output_file: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 >= args.len() {
                    anyhow::bail!("-o requires an argument");
                }
                i += 1;
                output_file = Some(args[i].clone());
            }
            s if !s.starts_with('-') => {
                input_file = Some(s);
            }
            _ => {
                eprintln!("Unknown flag: {}", args[i]);
            }
        }
        i += 1;
    }
    
    let input_file = input_file.ok_or_else(|| anyhow::anyhow!("No input file specified"))?;
    let source = std::fs::read_to_string(input_file)?;
    
    let default_output = std::path::Path::new(input_file)
        .with_extension("o")
        .to_string_lossy()
        .to_string();
    let output_file = output_file.unwrap_or(default_output);
    
    println!("\n解析 TypeScript: {}", input_file);
    let hir = ts_parser::parse(&source)?;
    
    println!("生成 native 代码...");
    let mut codegen = CodeGen::new().with_registry(registry.clone());
    let binary = codegen.compile(&hir)?;
    
    std::fs::write(&output_file, &binary)?;
    
    println!("✅ 输出: {} ({} bytes)", output_file, binary.len());
    
    if output_file.ends_with(".o") {
        let exe_file = output_file.replace(".o", ".exe");
        println!("\n尝试链接为可执行文件...");
        
        let link_result = try_link(&output_file, &exe_file, &registry);
        match link_result {
            Ok(size) => println!("✅ 生成: {} ({} bytes)", exe_file, size),
            Err(e) => println!("⚠️  链接失败（需要安装链接器）: {}", e),
        }
    }
    
    Ok(())
}

fn try_link(obj_file: &str, exe_file: &str, _registry: &extension::ExtensionRegistry) -> Result<usize> {
    let linker_instance = linker::Linker::new(exe_file);
    
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
        Err(e) => eprintln!("MSVC link failed: {}", e),
    }
    
    match linker_instance.link_with_clang(&obj_files) {
        Ok(result) => return Ok(result.len()),
        Err(e) => eprintln!("Clang link failed: {}", e),
    }
    
    match linker_instance.link_with_gcc(&obj_files) {
        Ok(result) => return Ok(result.len()),
        Err(e) => eprintln!("GCC link failed: {}", e),
    }
    
    anyhow::bail!("No suitable linker found (tried: MSVC link, clang, gcc)")
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

#[allow(dead_code)]
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

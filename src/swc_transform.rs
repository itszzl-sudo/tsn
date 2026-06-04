use anyhow::{anyhow, Result};
use crate::codegen::{BinOp, HirExpr, UnaryOp as HirUnaryOp};
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use std::sync::Arc;

pub fn parse(source: &str) -> Result<Vec<HirExpr>> {
    let cm: Arc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom("input.ts".into()).into(),
        source.to_string(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            decorators: true,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        EsVersion::Es2020,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().map_err(|e| anyhow!("swc 解析失败: {:?}", e))?;

    transform_module(&module)
}

fn transform_module(module: &Module) -> Result<Vec<HirExpr>> {
    let mut functions: Vec<HirExpr> = Vec::new();
    let mut top_stmts: Vec<HirExpr> = Vec::new();

    for item in &module.body {
        match item {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::ExportDecl(ExportDecl { decl: d, .. }) => {
                    match d {
                        Decl::Fn(f) => {
                            if let Some(hir) = transform_fn_decl(f)? {
                                functions.push(hir);
                            }
                        }
                        Decl::Var(v) => {
                            for decl in &v.decls {
                                if let Some(hir) = transform_var_declarator(decl, v.kind) {
                                    top_stmts.push(hir);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ModuleDecl::ExportDefaultDecl(ExportDefaultDecl { decl: d, .. }) => {
                    match d {
                        DefaultDecl::Fn(f) => {
                            if let Some(hir) = transform_fn_expr(&f.function, Some("default"))? {
                                functions.push(hir);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            ModuleItem::Stmt(stmt) => {
                if let Some(hir) = transform_stmt(stmt)? {
                    match &hir {
                        HirExpr::Function { .. } => functions.push(hir),
                        _ => top_stmts.push(hir),
                    }
                }
            }
        }
    }

    let has_main = functions.iter().any(|f| {
        if let HirExpr::Function { name, .. } = f {
            name == "main"
        } else {
            false
        }
    });

    if !top_stmts.is_empty() || !has_main {
        top_stmts.retain(|s| !is_bare_main_call(s));

        if has_main {
            if let Some(HirExpr::Function { body, .. }) =
                functions.iter_mut().find(|f| matches!(f, HirExpr::Function { name, .. } if name == "main"))
            {
                body.extend(top_stmts);
            }
        } else {
            functions.push(HirExpr::Function {
                name: "main".to_string(),
                params: vec![],
                body: top_stmts,
            });
        }
    }

    if !functions.iter().any(|f| matches!(f, HirExpr::Function { name, .. } if name == "main")) {
        functions.push(HirExpr::Function {
            name: "main".to_string(),
            params: vec![],
            body: vec![],
        });
    }

    Ok(functions)
}

fn is_bare_main_call(hir: &HirExpr) -> bool {
    matches!(hir, HirExpr::ExprStmt(inner) if matches!(inner.as_ref(), HirExpr::Call { callee, args, .. } if args.is_empty() && matches!(callee.as_ref(), HirExpr::Identifier(n) if n == "main")))
}

fn transform_stmt(stmt: &Stmt) -> Result<Option<HirExpr>> {
    match stmt {
        Stmt::Decl(decl) => transform_decl(decl),
        Stmt::Expr(ExprStmt { expr, .. }) => {
            let hir = transform_expr(expr)?;
            Ok(Some(HirExpr::ExprStmt(Box::new(hir))))
        }
        Stmt::Block(BlockStmt { stmts, .. }) => {
            let mut body = Vec::new();
            for s in stmts {
                if let Some(hir) = transform_stmt(s)? {
                    body.push(hir);
                }
            }
            Ok(Some(HirExpr::Block(body)))
        }
        Stmt::If(IfStmt { test, cons, alt, .. }) => {
            let cond = transform_expr(test)?;
            let then_body = transform_stmt_as_vec(cons)?;
            let else_body = if let Some(a) = alt {
                Some(transform_stmt_as_vec(a)?)
            } else {
                None
            };
            Ok(Some(HirExpr::If { cond: Box::new(cond), then_body, else_body }))
        }
        Stmt::While(WhileStmt { test, body, .. }) => {
            let cond = transform_expr(test)?;
            let body_vec = transform_stmt_as_vec(body)?;
            Ok(Some(HirExpr::While { cond: Box::new(cond), body: body_vec }))
        }
        Stmt::DoWhile(DoWhileStmt { test, body, .. }) => {
            let cond = transform_expr(test)?;
            let body_vec = transform_stmt_as_vec(body)?;
            let body_block = HirExpr::Block(body_vec.clone());
            let while_loop = HirExpr::While { cond: Box::new(cond), body: body_vec };
            Ok(Some(HirExpr::Block(vec![body_block, while_loop])))
        }
        Stmt::For(ForStmt { init, test, update, body, .. }) => {
            let init_hir = if let Some(i) = init {
                Some(Box::new(transform_for_init(i)?))
            } else {
                None
            };
            let cond_hir = if let Some(c) = test {
                Some(Box::new(transform_expr(c)?))
            } else {
                None
            };
            let update_hir = if let Some(u) = update {
                Some(Box::new(transform_expr_as_stmt(u)?))
            } else {
                None
            };
            let body_vec = transform_stmt_as_vec(body)?;
            Ok(Some(HirExpr::For { init: init_hir, cond: cond_hir, update: update_hir, body: body_vec }))
        }
        Stmt::ForIn(ForInStmt { left, right, body, .. }) => {
            let var_name = match left {
                ForHead::VarDecl(v) => v.decls.first().and_then(|d| pat_ident_name(&d.name)).unwrap_or_default(),
                ForHead::Pat(pat) => pat_ident_name(pat).unwrap_or_default(),
                ForHead::UsingDecl(_) => "_using".to_string(),
            };
            let obj = transform_expr(right)?;
            let body_vec = transform_stmt_as_vec(body)?;
            let keys_var = format!("_keys_{}", var_name);
            let keys_len_var = format!("_keys_len_{}", var_name);
            let idx_var = format!("_i_{}", var_name);
            let for_in_body = vec![
                HirExpr::Var { name: var_name.clone(), init: Some(Box::new(
                    HirExpr::Index { object: Box::new(HirExpr::Identifier(keys_var.clone())), index: Box::new(HirExpr::Identifier(idx_var.clone())) }
                )), is_mut: false },
            ];
            Ok(Some(HirExpr::Block(vec![
                HirExpr::Var { name: keys_var.clone(), init: Some(Box::new(
                    HirExpr::Call { callee: Box::new(HirExpr::Identifier("js_object_keys".to_string())), args: vec![obj] }
                )), is_mut: false },
                HirExpr::Var { name: keys_len_var.clone(), init: Some(Box::new(
                    HirExpr::Property { object: Box::new(HirExpr::Identifier(keys_var.clone())), name: "length".to_string() }
                )), is_mut: false },
                HirExpr::For {
                    init: Some(Box::new(HirExpr::Var { name: idx_var.clone(), init: Some(Box::new(HirExpr::Number(0.0))), is_mut: true })),
                    cond: Some(Box::new(HirExpr::Binary { op: BinOp::Lt, left: Box::new(HirExpr::Identifier(idx_var.clone())), right: Box::new(HirExpr::Identifier(keys_len_var.clone())) })),
                    update: Some(Box::new(HirExpr::Assign { target: Box::new(HirExpr::Identifier(idx_var.clone())), value: Box::new(HirExpr::Binary { op: BinOp::Add, left: Box::new(HirExpr::Identifier(idx_var.clone())), right: Box::new(HirExpr::Number(1.0)) }) })),
                    body: [&for_in_body[..], &body_vec[..]].concat(),
                },
            ])))
        }
        Stmt::ForOf(ForOfStmt { left, right, body, .. }) => {
            let var_name = match left {
                ForHead::VarDecl(v) => v.decls.first().and_then(|d| pat_ident_name(&d.name)).unwrap_or_default(),
                ForHead::Pat(pat) => pat_ident_name(pat).unwrap_or_default(),
                ForHead::UsingDecl(_) => "_using".to_string(),
            };
            let iterable = transform_expr(right)?;
            let body_vec = transform_stmt_as_vec(body)?;
            let len_var = format!("_len_{}", var_name);
            let idx_var = format!("_i_{}", var_name);
            let for_of_body = vec![
                HirExpr::Var { name: var_name.clone(), init: Some(Box::new(
                    HirExpr::Index { object: Box::new(iterable.clone()), index: Box::new(HirExpr::Identifier(idx_var.clone())) }
                )), is_mut: false },
            ];
            Ok(Some(HirExpr::Block(vec![
                HirExpr::Var { name: len_var.clone(), init: Some(Box::new(
                    HirExpr::Property { object: Box::new(iterable.clone()), name: "length".to_string() }
                )), is_mut: false },
                HirExpr::For {
                    init: Some(Box::new(HirExpr::Var { name: idx_var.clone(), init: Some(Box::new(HirExpr::Number(0.0))), is_mut: true })),
                    cond: Some(Box::new(HirExpr::Binary { op: BinOp::Lt, left: Box::new(HirExpr::Identifier(idx_var.clone())), right: Box::new(HirExpr::Identifier(len_var.clone())) })),
                    update: Some(Box::new(HirExpr::Assign { target: Box::new(HirExpr::Identifier(idx_var.clone())), value: Box::new(HirExpr::Binary { op: BinOp::Add, left: Box::new(HirExpr::Identifier(idx_var.clone())), right: Box::new(HirExpr::Number(1.0)) }) })),
                    body: [&for_of_body[..], &body_vec[..]].concat(),
                },
            ])))
        }
        Stmt::Return(ReturnStmt { arg, .. }) => {
            let val = if let Some(a) = arg {
                Some(Box::new(transform_expr(a)?))
            } else {
                None
            };
            Ok(Some(HirExpr::Return(val)))
        }
        Stmt::Break(_) => Ok(Some(HirExpr::Break)),
        Stmt::Continue(_) => Ok(Some(HirExpr::Continue)),
        Stmt::Throw(ThrowStmt { arg, .. }) => {
            Ok(Some(HirExpr::Throw(Some(Box::new(transform_expr(arg)?)))))
        }
        Stmt::Try(try_stmt) => {
            let TryStmt { block, handler, finalizer, .. } = try_stmt.as_ref();
            let try_body = transform_block_stmts(&block.stmts)?;
            let (catch_param, catch_body) = if let Some(CatchClause { param, body: cb, .. }) = handler {
                let cp = param.as_ref().and_then(|p| pat_ident_name(p));
                let cb_body = transform_block_stmts(&cb.stmts)?;
                (cp, Some(cb_body))
            } else {
                (None, None)
            };
            let finally_body = if let Some(f) = finalizer {
                Some(transform_block_stmts(&f.stmts)?)
            } else {
                None
            };
            Ok(Some(HirExpr::TryCatch { try_body, catch_param, catch_body, finally_body }))
        }
        Stmt::Switch(SwitchStmt { discriminant, cases, .. }) => {
            let disc = transform_expr(discriminant)?;
            let mut if_chain: Option<HirExpr> = None;
            for case in cases.iter().rev() {
                let body = case.cons.iter().filter_map(|s| transform_stmt(s).ok().flatten()).collect::<Vec<_>>();
                if let Some(test) = &case.test {
                    let test_expr = transform_expr(test)?;
                    let cond = HirExpr::Binary { op: BinOp::Eq, left: Box::new(disc.clone()), right: Box::new(test_expr) };
                    if_chain = Some(if let Some(prev) = if_chain {
                        HirExpr::If { cond: Box::new(cond), then_body: body, else_body: Some(vec![prev]) }
                    } else {
                        HirExpr::If { cond: Box::new(cond), then_body: body, else_body: None }
                    });
                } else {
                    if_chain = Some(HirExpr::Block(body));
                }
            }
            Ok(if_chain)
        }
        Stmt::Labeled(_) | Stmt::With(_) | Stmt::Empty(_) | Stmt::Debugger(_) => Ok(None),
    }
}

fn transform_decl(decl: &Decl) -> Result<Option<HirExpr>> {
    match decl {
        Decl::Fn(f) => transform_fn_decl(f),
        Decl::Var(v) => {
            let mut stmts = Vec::new();
            for d in &v.decls {
                if let Some(hir) = transform_var_declarator(d, v.kind) {
                    stmts.push(hir);
                }
            }
            if stmts.len() == 1 {
                Ok(Some(stmts.remove(0)))
            } else if stmts.is_empty() {
                Ok(None)
            } else {
                Ok(Some(HirExpr::Block(stmts)))
            }
        }
        Decl::TsEnum(e) => transform_enum(e),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsModule(_) | Decl::Using(_) | Decl::Class(_) => Ok(None),
    }
}

fn transform_fn_decl(f: &FnDecl) -> Result<Option<HirExpr>> {
    if f.declare {
        return Ok(None);
    }
    let name = f.ident.sym.to_string();
    let (params, default_inits) = extract_params(&f.function.params)?;
    let mut body = transform_block_stmts(&f.function.body.as_ref().map(|b| b.stmts.as_slice()).unwrap_or(&[]))?;
    let mut full_body = default_inits;
    full_body.append(&mut body);
    Ok(Some(HirExpr::Function { name, params, body: full_body }))
}

fn transform_fn_expr(f: &Function, default_name: Option<&str>) -> Result<Option<HirExpr>> {
    let name = default_name.unwrap_or("anonymous").to_string();
    let (params, default_inits) = extract_params(&f.params)?;
    let mut body = transform_block_stmts(&f.body.as_ref().map(|b| b.stmts.as_slice()).unwrap_or(&[]))?;
    let mut full_body = default_inits;
    full_body.append(&mut body);
    Ok(Some(HirExpr::Function { name, params, body: full_body }))
}

fn transform_enum(e: &TsEnumDecl) -> Result<Option<HirExpr>> {
    if e.is_const {
        return Ok(None);
    }
    let mut stmts = Vec::new();
    let mut next_val: f64 = 0.0;
    for member in &e.members {
        let name = match &member.id {
            TsEnumMemberId::Ident(id) => id.sym.to_string(),
            TsEnumMemberId::Str(s) => s.value.to_string_lossy().to_string(),
        };
        if let Some(init) = &member.init {
            if let Ok(hir) = transform_expr(init) {
                if let HirExpr::Number(n) = &hir {
                    next_val = *n;
                }
                stmts.push(HirExpr::Var {
                    name: format!("{}_{}", e.id.sym, name),
                    init: Some(Box::new(hir)),
                    is_mut: false,
                });
            }
        } else {
            stmts.push(HirExpr::Var {
                name: format!("{}_{}", e.id.sym, name),
                init: Some(Box::new(HirExpr::Number(next_val))),
                is_mut: false,
            });
        }
        next_val += 1.0;
    }
    if stmts.is_empty() {
        Ok(None)
    } else if stmts.len() == 1 {
        Ok(Some(stmts.remove(0)))
    } else {
        Ok(Some(HirExpr::Block(stmts)))
    }
}

fn extract_params(fn_params: &[Param]) -> Result<(Vec<String>, Vec<HirExpr>)> {
    let mut params = Vec::new();
    let mut default_inits = Vec::new();
    for (i, p) in fn_params.iter().enumerate() {
        match &p.pat {
            Pat::Assign(AssignPat { left, right, .. }) => {
                if let Some(name) = pat_ident_name(left) {
                    params.push(name.clone());
                    let default_val = transform_expr(right)?;
                    let check = HirExpr::Binary {
                        op: BinOp::Eq,
                        left: Box::new(HirExpr::Identifier(name.clone())),
                        right: Box::new(HirExpr::Undefined),
                    };
                    let assign = HirExpr::Assign {
                        target: Box::new(HirExpr::Identifier(name.clone())),
                        value: Box::new(default_val),
                    };
                    default_inits.push(HirExpr::If {
                        cond: Box::new(check),
                        then_body: vec![HirExpr::ExprStmt(Box::new(assign))],
                        else_body: None,
                    });
                } else {
                    params.push("_".to_string());
                }
            }
            Pat::Rest(RestPat { arg, .. }) => {
                if let Some(name) = pat_ident_name(arg) {
                    let rest_var = name.clone();
                    let start_idx = i;
                    default_inits.push(HirExpr::Var {
                        name: rest_var.clone(),
                        init: Some(Box::new(HirExpr::Call {
                            callee: Box::new(HirExpr::Identifier("js_array_from_args".to_string())),
                            args: vec![HirExpr::Number(start_idx as f64)],
                        })),
                        is_mut: false,
                    });
                }
            }
            _ => {
                if let Some(name) = pat_ident_name(&p.pat) {
                    params.push(name);
                } else {
                    params.push("_".to_string());
                }
            }
        }
    }
    Ok((params, default_inits))
}

fn transform_var_declarator(decl: &VarDeclarator, kind: VarDeclKind) -> Option<HirExpr> {
    let is_mut = kind != VarDeclKind::Const;
    match &decl.name {
        Pat::Object(ObjectPat { props, .. }) => {
            let init_expr = decl.init.as_ref().and_then(|e| transform_expr(e).ok());
            let tmp_name = format!("_destr_{}", pat_ident_name(&decl.name).unwrap_or_else(|| "tmp".to_string()));
            let mut stmts = Vec::new();
            stmts.push(HirExpr::Var { name: tmp_name.clone(), init: init_expr.map(Box::new), is_mut: false });
            for prop in props {
                match prop {
                    ObjectPatProp::KeyValue(KeyValuePatProp { key, value, .. }) => {
                        let (prop_name, var_name) = match key {
                            PropName::Ident(id) => (id.sym.to_string(), None),
                            PropName::Str(s) => (s.value.to_string_lossy().to_string(), None),
                            PropName::Num(n) => (n.value.to_string(), None),
                            PropName::Computed(ComputedPropName { expr, .. }) => {
                                if let Expr::Ident(id) = expr.as_ref() {
                                    (id.sym.to_string(), None)
                                } else {
                                    continue;
                                }
                            }
                            PropName::BigInt(b) => ((&*b.value).to_string(), None),
                        };
                        let final_var = match value.as_ref() {
                            Pat::Ident(bi) => bi.id.sym.to_string(),
                            Pat::Assign(AssignPat { left, right, .. }) => {
                                let var = pat_ident_name(left).unwrap_or_else(|| prop_name.clone());
                                let default_val = transform_expr(right).ok()?;
                                let access = HirExpr::Property { object: Box::new(HirExpr::Identifier(tmp_name.clone())), name: prop_name.clone() };
                                let check = HirExpr::Binary { op: BinOp::Eq, left: Box::new(access.clone()), right: Box::new(HirExpr::Undefined) };
                                let init = HirExpr::Ternary { cond: Box::new(check), then_expr: Box::new(default_val), else_expr: Box::new(access) };
                                stmts.push(HirExpr::Var { name: var.clone(), init: Some(Box::new(init)), is_mut });
                                continue;
                            }
                            _ => prop_name.clone(),
                        };
                        if var_name.is_some() {
                            let vn = var_name.unwrap();
                            let access = HirExpr::Property { object: Box::new(HirExpr::Identifier(tmp_name.clone())), name: prop_name };
                            stmts.push(HirExpr::Var { name: vn, init: Some(Box::new(access)), is_mut });
                        } else {
                            let access = HirExpr::Property { object: Box::new(HirExpr::Identifier(tmp_name.clone())), name: prop_name };
                            stmts.push(HirExpr::Var { name: final_var, init: Some(Box::new(access)), is_mut });
                        }
                    }
                    ObjectPatProp::Assign(AssignPatProp { key, value, .. }) => {
                        let var_name = key.id.sym.to_string();
                        let access = HirExpr::Property { object: Box::new(HirExpr::Identifier(tmp_name.clone())), name: var_name.clone() };
                        let init = if let Some(default_val) = value {
                            let default_hir = transform_expr(default_val).ok()?;
                            let check = HirExpr::Binary { op: BinOp::Eq, left: Box::new(access.clone()), right: Box::new(HirExpr::Undefined) };
                            HirExpr::Ternary { cond: Box::new(check), then_expr: Box::new(default_hir), else_expr: Box::new(access) }
                        } else {
                            access
                        };
                        stmts.push(HirExpr::Var { name: var_name, init: Some(Box::new(init)), is_mut });
                    }
                    ObjectPatProp::Rest(RestPat { arg, .. }) => {
                        if let Some(name) = pat_ident_name(arg) {
                            stmts.push(HirExpr::Var { name, init: Some(Box::new(HirExpr::Identifier(tmp_name.clone()))), is_mut });
                        }
                    }

                }
            }
            if stmts.len() == 1 {
                stmts.remove(0).into()
            } else {
                Some(HirExpr::Block(stmts))
            }
        }
        Pat::Array(ArrayPat { elems, .. }) => {
            let init_expr = decl.init.as_ref().and_then(|e| transform_expr(e).ok());
            let tmp_name = format!("_destr_arr_{}", elems.iter().position(|e| e.is_some()).unwrap_or(0));
            let mut stmts = Vec::new();
            stmts.push(HirExpr::Var { name: tmp_name.clone(), init: init_expr.map(Box::new), is_mut: false });
            for (i, elem) in elems.iter().enumerate() {
                if let Some(pat) = elem {
                    match pat {
                        Pat::Ident(bi) => {
                            let var_name = bi.id.sym.to_string();
                            let access = HirExpr::Index { object: Box::new(HirExpr::Identifier(tmp_name.clone())), index: Box::new(HirExpr::Number(i as f64)) };
                            stmts.push(HirExpr::Var { name: var_name, init: Some(Box::new(access)), is_mut });
                        }
                        Pat::Assign(AssignPat { left, right, .. }) => {
                            let var_name = pat_ident_name(left).unwrap_or_else(|| format!("_arr{}", i));
                            let default_val = transform_expr(right).ok()?;
                            let access = HirExpr::Index { object: Box::new(HirExpr::Identifier(tmp_name.clone())), index: Box::new(HirExpr::Number(i as f64)) };
                            let check = HirExpr::Binary { op: BinOp::Eq, left: Box::new(access.clone()), right: Box::new(HirExpr::Undefined) };
                            let init = HirExpr::Ternary { cond: Box::new(check), then_expr: Box::new(default_val), else_expr: Box::new(access) };
                            stmts.push(HirExpr::Var { name: var_name, init: Some(Box::new(init)), is_mut });
                        }
                        Pat::Rest(RestPat { arg, .. }) => {
                            if let Some(name) = pat_ident_name(arg) {
                                let mut sig_args = vec![
                                    HirExpr::Identifier(tmp_name.clone()),
                                    HirExpr::Number(i as f64),
                                ];
                                stmts.push(HirExpr::Var { name, init: Some(Box::new(
                                    HirExpr::Call { callee: Box::new(HirExpr::Identifier("js_array_slice".to_string())), args: sig_args }
                                )), is_mut });
                            }
                        }
                        Pat::Expr(expr) => {
                            if let Ok(hir) = transform_expr(expr) {
                                let access = HirExpr::Index { object: Box::new(HirExpr::Identifier(tmp_name.clone())), index: Box::new(HirExpr::Number(i as f64)) };
                                stmts.push(HirExpr::Assign { target: Box::new(hir), value: Box::new(access) });
                            }
                        }
                        _ => {}
                    }
                }
            }
            if stmts.len() == 1 {
                stmts.remove(0).into()
            } else {
                Some(HirExpr::Block(stmts))
            }
        }
        _ => {
            let name = pat_ident_name(&decl.name).unwrap_or_default();
            let init = decl.init.as_ref().and_then(|e| transform_expr(e).ok());
            Some(HirExpr::Var { name, init: init.map(Box::new), is_mut })
        }
    }
}

fn binding_ident_name(bi: &BindingIdent) -> String {
    bi.id.sym.to_string()
}

fn pat_ident_name(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(bi) => Some(bi.id.sym.to_string()),
        Pat::Array(ArrayPat { elems, .. }) => elems.first().and_then(|e| e.as_ref().and_then(|e| pat_ident_name(e))),
        Pat::Object(ObjectPat { props, .. }) => props.first().and_then(|p| match p {
            ObjectPatProp::KeyValue(KeyValuePatProp { key, .. }) => match key {
                PropName::Ident(id) => Some(id.sym.to_string()),
                PropName::Str(s) => Some(s.value.to_string_lossy().to_string()),
                _ => None,
            },
            ObjectPatProp::Assign(AssignPatProp { key, .. }) => Some(key.id.sym.to_string()),
            _ => None,
        }),
        Pat::Rest(RestPat { arg, .. }) => pat_ident_name(arg),
        Pat::Assign(AssignPat { left, .. }) => pat_ident_name(left),
        Pat::Expr(_) => None,
        Pat::Invalid(_) => None,
    }
}

fn transform_block_stmts(stmts: &[Stmt]) -> Result<Vec<HirExpr>> {
    let mut result = Vec::new();
    for s in stmts {
        if let Some(hir) = transform_stmt(s)? {
            result.push(hir);
        }
    }
    Ok(result)
}

fn transform_stmt_as_vec(stmt: &Stmt) -> Result<Vec<HirExpr>> {
    match stmt {
        Stmt::Block(BlockStmt { stmts, .. }) => transform_block_stmts(stmts),
        _ => {
            if let Some(hir) = transform_stmt(stmt)? {
                Ok(vec![hir])
            } else {
                Ok(vec![])
            }
        }
    }
}

fn transform_for_init(init: &VarDeclOrExpr) -> Result<HirExpr> {
    match init {
        VarDeclOrExpr::VarDecl(v) => {
            let mut stmts = Vec::new();
            for d in &v.decls {
                if let Some(hir) = transform_var_declarator(d, v.kind) {
                    stmts.push(hir);
                }
            }
            if stmts.len() == 1 {
                Ok(stmts.remove(0))
            } else {
                Ok(HirExpr::Block(stmts))
            }
        }
        VarDeclOrExpr::Expr(e) => Ok(HirExpr::ExprStmt(Box::new(transform_expr(e)?))),
    }
}

fn transform_expr_as_stmt(expr: &Expr) -> Result<HirExpr> {
    Ok(HirExpr::ExprStmt(Box::new(transform_expr(expr)?)))
}

fn transform_expr(expr: &Expr) -> Result<HirExpr> {
    match expr {
        Expr::Lit(lit) => transform_lit(lit),
        Expr::Ident(Ident { sym, .. }) => Ok(HirExpr::Identifier(sym.to_string())),
        Expr::Bin(BinExpr { op, left, right, .. }) => {
            if *op == BinaryOp::NullishCoalescing {
                let l = transform_expr(left)?;
                let r = transform_expr(right)?;
                let is_not_null = HirExpr::Binary { op: BinOp::Ne, left: Box::new(l.clone()), right: Box::new(HirExpr::Null) };
                let is_not_undef = HirExpr::Binary { op: BinOp::Ne, left: Box::new(l.clone()), right: Box::new(HirExpr::Undefined) };
                let cond = HirExpr::Binary { op: BinOp::And, left: Box::new(is_not_null), right: Box::new(is_not_undef) };
                return Ok(HirExpr::Ternary { cond: Box::new(cond), then_expr: Box::new(l), else_expr: Box::new(r) });
            }
            if *op == BinaryOp::In {
                let key = transform_expr(left)?;
                let obj = transform_expr(right)?;
                return Ok(HirExpr::Call { callee: Box::new(HirExpr::Identifier("js_in".to_string())), args: vec![key, obj] });
            }
            let l = transform_expr(left)?;
            let r = transform_expr(right)?;
            let bin_op = transform_binop(*op)?;
            Ok(HirExpr::Binary { op: bin_op, left: Box::new(l), right: Box::new(r) })
        }
        Expr::Unary(UnaryExpr { op, arg, .. }) => {
            let val = transform_expr(arg)?;
            match op {
                swc_ecma_ast::UnaryOp::Bang => Ok(HirExpr::Unary { op: HirUnaryOp::Not, operand: Box::new(val) }),
                swc_ecma_ast::UnaryOp::Minus => Ok(HirExpr::Unary { op: HirUnaryOp::Neg, operand: Box::new(val) }),
                swc_ecma_ast::UnaryOp::Plus => Ok(val),
                swc_ecma_ast::UnaryOp::TypeOf => Ok(HirExpr::Typeof(Box::new(val))),
                swc_ecma_ast::UnaryOp::Void => Ok(HirExpr::Undefined),
                _ => Ok(val),
            }
        }
        Expr::Update(UpdateExpr { op, arg, prefix, .. }) => {
            let target = transform_expr(arg)?;
            let delta = HirExpr::Number(1.0);
            match (op, prefix) {
                (UpdateOp::PlusPlus, true) => Ok(HirExpr::Assign {
                    target: Box::new(target.clone()),
                    value: Box::new(HirExpr::Binary { op: BinOp::Add, left: Box::new(target), right: Box::new(delta) }),
                }),
                (UpdateOp::MinusMinus, true) => Ok(HirExpr::Assign {
                    target: Box::new(target.clone()),
                    value: Box::new(HirExpr::Binary { op: BinOp::Sub, left: Box::new(target), right: Box::new(delta) }),
                }),
                _ => Ok(target),
            }
        }
        Expr::Assign(AssignExpr { op, left, right, .. }) => {
            let val = transform_expr(right)?;
            let target = transform_assign_target(left)?;
            match op {
                AssignOp::Assign => Ok(HirExpr::Assign { target: Box::new(target), value: Box::new(val) }),
                AssignOp::AddAssign | AssignOp::SubAssign | AssignOp::MulAssign
                | AssignOp::DivAssign | AssignOp::ModAssign
                | AssignOp::BitAndAssign | AssignOp::BitOrAssign | AssignOp::BitXorAssign
                | AssignOp::LShiftAssign | AssignOp::RShiftAssign => {
                    let bin_op = match op {
                        AssignOp::AddAssign => BinOp::Add,
                        AssignOp::SubAssign => BinOp::Sub,
                        AssignOp::MulAssign => BinOp::Mul,
                        AssignOp::DivAssign => BinOp::Div,
                        AssignOp::ModAssign => BinOp::Mod,
                        AssignOp::BitAndAssign => BinOp::BitAnd,
                        AssignOp::BitOrAssign => BinOp::BitOr,
                        AssignOp::BitXorAssign => BinOp::BitXor,
                        AssignOp::LShiftAssign => BinOp::Shl,
                        AssignOp::RShiftAssign => BinOp::Shr,
                        _ => BinOp::Add,
                    };
                    Ok(HirExpr::Assign {
                        target: Box::new(target.clone()),
                        value: Box::new(HirExpr::Binary { op: bin_op, left: Box::new(target), right: Box::new(val) }),
                    })
                }
                _ => Ok(HirExpr::Assign { target: Box::new(target), value: Box::new(val) }),
            }
        }
        Expr::Call(CallExpr { callee, args, .. }) => {
            let callee_hir = match callee {
                Callee::Expr(e) => transform_expr(e)?,
                Callee::Super(_) | Callee::Import(_) => HirExpr::Undefined,
            };
            let has_spread = args.iter().any(|a| a.spread.is_some());
            if has_spread {
                let mut arg_hirs: Vec<HirExpr> = Vec::new();
                for a in args {
                    if a.spread.is_some() {
                        if let Ok(spread_expr) = transform_expr(&a.expr) {
                            arg_hirs.push(HirExpr::Call {
                                callee: Box::new(HirExpr::Identifier("js_array_spread".to_string())),
                                args: vec![spread_expr],
                            });
                        }
                    } else {
                        if let Ok(hir) = transform_expr(&a.expr) {
                            arg_hirs.push(HirExpr::Array(vec![hir]));
                        }
                    }
                }
                let args_array = if arg_hirs.is_empty() {
                    HirExpr::Array(vec![])
                } else if arg_hirs.len() == 1 {
                    arg_hirs.remove(0)
                } else {
                    let mut result = arg_hirs.remove(0);
                    for elem in arg_hirs {
                        result = HirExpr::Call {
                            callee: Box::new(HirExpr::Identifier("js_array_concat".to_string())),
                            args: vec![result, elem],
                        };
                    }
                    result
                };
                Ok(HirExpr::Call {
                    callee: Box::new(HirExpr::Identifier("js_function_apply".to_string())),
                    args: vec![callee_hir, args_array],
                })
            } else {
                let arg_hirs: Vec<HirExpr> = args.iter()
                    .map(|a| transform_expr(&a.expr))
                    .filter_map(|r| r.ok())
                    .collect();
                Ok(HirExpr::Call { callee: Box::new(callee_hir), args: arg_hirs })
            }
        }
        Expr::Member(MemberExpr { obj, prop, .. }) => {
            let obj_hir = transform_expr(obj)?;
            match prop {
                MemberProp::Ident(IdentName { sym, .. }) => Ok(HirExpr::Property { object: Box::new(obj_hir), name: sym.to_string() }),
                MemberProp::Computed(ComputedPropName { expr, .. }) => {
                    let idx = transform_expr(expr)?;
                    Ok(HirExpr::Index { object: Box::new(obj_hir), index: Box::new(idx) })
                }
                MemberProp::PrivateName(_) => Ok(HirExpr::Undefined),
            }
        }
        Expr::Cond(CondExpr { test, cons, alt, .. }) => {
            Ok(HirExpr::Ternary {
                cond: Box::new(transform_expr(test)?),
                then_expr: Box::new(transform_expr(cons)?),
                else_expr: Box::new(transform_expr(alt)?),
            })
        }
        Expr::Arrow(ArrowExpr { params, body, .. }) => {
            let mut param_names = Vec::new();
            let mut default_inits = Vec::new();
            for (i, p) in params.iter().enumerate() {
                match p {
                    Pat::Assign(AssignPat { left, right, .. }) => {
                        if let Some(name) = pat_ident_name(left) {
                            param_names.push(name.clone());
                            let default_val = transform_expr(right)?;
                            let check = HirExpr::Binary {
                                op: BinOp::Eq,
                                left: Box::new(HirExpr::Identifier(name.clone())),
                                right: Box::new(HirExpr::Undefined),
                            };
                            let assign = HirExpr::Assign {
                                target: Box::new(HirExpr::Identifier(name.clone())),
                                value: Box::new(default_val),
                            };
                            default_inits.push(HirExpr::If {
                                cond: Box::new(check),
                                then_body: vec![HirExpr::ExprStmt(Box::new(assign))],
                                else_body: None,
                            });
                        } else {
                            param_names.push("_".to_string());
                        }
                    }
                    Pat::Rest(RestPat { arg, .. }) => {
                        if let Some(name) = pat_ident_name(arg) {
                            let start_idx = i;
                            default_inits.push(HirExpr::Var {
                                name: name.clone(),
                                init: Some(Box::new(HirExpr::Call {
                                    callee: Box::new(HirExpr::Identifier("js_array_from_args".to_string())),
                                    args: vec![HirExpr::Number(start_idx as f64)],
                                })),
                                is_mut: false,
                            });
                        }
                    }
                    _ => {
                        if let Some(name) = pat_ident_name(p) {
                            param_names.push(name);
                        } else {
                            param_names.push("_".to_string());
                        }
                    }
                }
            }
            let mut body_hirs = match &**body {
                BlockStmtOrExpr::BlockStmt(b) => transform_block_stmts(&b.stmts)?,
                BlockStmtOrExpr::Expr(e) => vec![HirExpr::Return(Some(Box::new(transform_expr(e)?)))],
            };
            let mut full_body = default_inits;
            full_body.append(&mut body_hirs);
            Ok(HirExpr::Function { name: "anonymous".to_string(), params: param_names, body: full_body })
        }
        Expr::Fn(FnExpr { ident, function, .. }) => {
            let name = ident.as_ref().map(|id| id.sym.to_string()).unwrap_or_else(|| "anonymous".to_string());
            let params: Vec<String> = function.params.iter().filter_map(|p| pat_ident_name(&p.pat)).collect();
            let body = transform_block_stmts(&function.body.as_ref().map(|b| b.stmts.as_slice()).unwrap_or(&[]))?;
            Ok(HirExpr::Function { name, params, body })
        }
        Expr::Array(ArrayLit { elems, .. }) => {
            let has_spread = elems.iter().any(|e| e.as_ref().map_or(false, |e| e.spread.is_some()));
            if !has_spread {
                let elements: Vec<HirExpr> = elems.iter()
                    .filter_map(|e| e.as_ref().and_then(|e| transform_expr(&e.expr).ok()))
                    .collect();
                return Ok(HirExpr::Array(elements));
            }
            let mut elements: Vec<HirExpr> = Vec::new();
            for e in elems {
                match e {
                    Some(ExprOrSpread { expr, spread: None, .. }) => {
                        if let Ok(hir) = transform_expr(expr) {
                            elements.push(HirExpr::Array(vec![hir]));
                        }
                    }
                    Some(ExprOrSpread { expr, spread: Some(_), .. }) => {
                        if let Ok(spread_expr) = transform_expr(expr) {
                            elements.push(HirExpr::Call {
                                callee: Box::new(HirExpr::Identifier("js_array_spread".to_string())),
                                args: vec![spread_expr],
                            });
                        }
                    }
                    None => {}
                }
            }
            if elements.is_empty() {
                Ok(HirExpr::Array(vec![]))
            } else if elements.len() == 1 {
                Ok(elements.remove(0))
            } else {
                let mut result = elements.remove(0);
                for elem in elements {
                    result = HirExpr::Call {
                        callee: Box::new(HirExpr::Identifier("js_array_concat".to_string())),
                        args: vec![result, elem],
                    };
                }
                Ok(result)
            }
        }
        Expr::Object(ObjectLit { props, .. }) => {
            let mut properties = Vec::new();
            for p in props {
                match p {
                    PropOrSpread::Prop(prop) => match prop.as_ref() {
                        Prop::KeyValue(KeyValueProp { key, value, .. }) => {
                            let (key_name, key_expr) = match key {
                                PropName::Ident(id) => (id.sym.to_string(), None),
                                PropName::Str(s) => (s.value.to_string_lossy().to_string(), None),
                                PropName::Num(n) => (n.value.to_string(), None),
                                PropName::BigInt(b) => ((&*b.value).to_string(), None),
                                PropName::Computed(ComputedPropName { expr, .. }) => {
                                    if let Ok(hir) = transform_expr(expr) {
                                        (String::new(), Some(hir))
                                    } else {
                                        continue;
                                    }
                                }
                            };
                            if let Ok(val) = transform_expr(value) {
                                if let Some(idx_expr) = key_expr {
                                    properties.push((format!("_computed_{}", properties.len()), HirExpr::Call {
                                        callee: Box::new(HirExpr::Identifier("js_object_set_computed".to_string())),
                                        args: vec![HirExpr::Number(properties.len() as f64), idx_expr, val],
                                    }));
                                } else {
                                    properties.push((key_name, val));
                                }
                            }
                        }
                        Prop::Shorthand(Ident { sym, .. }) => {
                            properties.push((sym.to_string(), HirExpr::Identifier(sym.to_string())));
                        }
                        _ => {}
                    },
                    PropOrSpread::Spread(SpreadElement { expr, .. }) => {
                        if let Ok(spread_hir) = transform_expr(expr) {
                            properties.push((format!("_spread_{}", properties.len()), HirExpr::Call {
                                callee: Box::new(HirExpr::Identifier("js_object_spread".to_string())),
                                args: vec![spread_hir],
                            }));
                        }
                    }
                }
            }
            Ok(HirExpr::Object(properties))
        }
        Expr::Tpl(Tpl { quasis, exprs, .. }) => {
            Ok(transform_template(quasis, exprs)?)
        }
        Expr::Seq(SeqExpr { exprs, .. }) => {
            if let Some(last) = exprs.last() {
                transform_expr(last)
            } else {
                Ok(HirExpr::Undefined)
            }
        }
        Expr::Paren(ParenExpr { expr, .. }) => transform_expr(expr),
        Expr::New(NewExpr { callee, args, .. }) => {
            let callee_hir = transform_expr(callee)?;
            let arg_hirs: Vec<HirExpr> = args.as_ref().map(|a| a.iter()
                .map(|a| transform_expr(&a.expr))
                .filter_map(|r| r.ok())
                .collect()).unwrap_or_default();
            Ok(HirExpr::Call { callee: Box::new(callee_hir), args: arg_hirs })
        }
        Expr::TsAs(TsAsExpr { expr, .. }) => transform_expr(expr),
        Expr::TsNonNull(TsNonNullExpr { expr, .. }) => transform_expr(expr),
        Expr::TsTypeAssertion(TsTypeAssertion { expr, .. }) => transform_expr(expr),
        Expr::TsInstantiation(TsInstantiation { expr, .. }) => transform_expr(expr),
        Expr::TsConstAssertion(TsConstAssertion { expr, .. }) => transform_expr(expr),
        Expr::TsSatisfies(TsSatisfiesExpr { expr, .. }) => transform_expr(expr),
        Expr::MetaProp(MetaPropExpr { kind: MetaPropKind::ImportMeta, .. }) => Ok(HirExpr::Undefined),
        Expr::MetaProp(MetaPropExpr { kind: MetaPropKind::NewTarget, .. }) => Ok(HirExpr::Undefined),
        Expr::Await(AwaitExpr { arg, .. }) => transform_expr(arg),
        Expr::OptChain(OptChainExpr { base, .. }) => {
            match base.as_ref() {
                OptChainBase::Member(MemberExpr { obj, prop, .. }) => {
                    let obj_hir = transform_expr(obj)?;
                    let is_not_null = HirExpr::Binary { op: BinOp::Ne, left: Box::new(obj_hir.clone()), right: Box::new(HirExpr::Null) };
                    let is_not_undef = HirExpr::Binary { op: BinOp::Ne, left: Box::new(obj_hir.clone()), right: Box::new(HirExpr::Undefined) };
                    let cond = HirExpr::Binary { op: BinOp::And, left: Box::new(is_not_null), right: Box::new(is_not_undef) };
                    let access = match prop {
                        MemberProp::Ident(IdentName { sym, .. }) => HirExpr::Property { object: Box::new(obj_hir), name: sym.to_string() },
                        MemberProp::Computed(ComputedPropName { expr, .. }) => HirExpr::Index { object: Box::new(obj_hir), index: Box::new(transform_expr(expr)?) },
                        MemberProp::PrivateName(_) => HirExpr::Undefined,
                    };
                    Ok(HirExpr::Ternary { cond: Box::new(cond), then_expr: Box::new(access), else_expr: Box::new(HirExpr::Undefined) })
                }
                OptChainBase::Call(OptCall { callee, args, .. }) => {
                    let callee_hir = transform_expr(callee)?;
                    let is_not_null = HirExpr::Binary { op: BinOp::Ne, left: Box::new(callee_hir.clone()), right: Box::new(HirExpr::Null) };
                    let is_not_undef = HirExpr::Binary { op: BinOp::Ne, left: Box::new(callee_hir.clone()), right: Box::new(HirExpr::Undefined) };
                    let cond = HirExpr::Binary { op: BinOp::And, left: Box::new(is_not_null), right: Box::new(is_not_undef) };
                    let arg_hirs: Vec<HirExpr> = args.iter()
                        .map(|a| transform_expr(&a.expr))
                        .filter_map(|r| r.ok())
                        .collect();
                    let call = HirExpr::Call { callee: Box::new(callee_hir), args: arg_hirs };
                    Ok(HirExpr::Ternary { cond: Box::new(cond), then_expr: Box::new(call), else_expr: Box::new(HirExpr::Undefined) })
                }
            }
        }
        Expr::TaggedTpl(TaggedTpl { tpl, .. }) => {
            Ok(transform_template(&tpl.quasis, &tpl.exprs)?)
        }
        Expr::This(_) => Ok(HirExpr::Undefined),
        Expr::SuperProp(_) => Ok(HirExpr::Undefined),
        Expr::Class(_) => Ok(HirExpr::Undefined),
        Expr::Yield(_) => Ok(HirExpr::Undefined),
        Expr::Invalid(_) => Ok(HirExpr::Undefined),
        Expr::PrivateName(_) => Ok(HirExpr::Undefined),
        Expr::JSXMember(_) | Expr::JSXNamespacedName(_) | Expr::JSXEmpty(_) | Expr::JSXElement(_) | Expr::JSXFragment(_) => Ok(HirExpr::Undefined),
    }
}

fn transform_template(quasis: &[TplElement], exprs: &[Box<Expr>]) -> Result<HirExpr> {
    let mut parts: Vec<HirExpr> = Vec::new();
    for (i, quasi) in quasis.iter().enumerate() {
        let raw = (&*quasi.raw).to_owned();
        if !raw.is_empty() {
            parts.push(HirExpr::String(raw));
        }
        if i < exprs.len() {
            if let Ok(e) = transform_expr(&exprs[i]) {
                parts.push(e);
            }
        }
    }
    if parts.is_empty() {
        Ok(HirExpr::String(String::new()))
    } else if parts.len() == 1 {
        Ok(parts.remove(0))
    } else {
        let mut result = parts.remove(0);
        for part in parts {
            result = HirExpr::Binary { op: BinOp::Add, left: Box::new(result), right: Box::new(part) };
        }
        Ok(result)
    }
}

fn transform_lit(lit: &Lit) -> Result<HirExpr> {
    match lit {
        Lit::Str(Str { value, .. }) => Ok(HirExpr::String(value.to_string_lossy().to_string())),
        Lit::Bool(Bool { value, .. }) => Ok(HirExpr::Boolean(*value)),
        Lit::Null(_) => Ok(HirExpr::Null),
        Lit::Num(Number { value, .. }) => Ok(HirExpr::Number(*value)),
        Lit::BigInt(BigInt { value, .. }) => {
            let s = value.to_string();
            let n: f64 = s.parse().unwrap_or(0.0);
            Ok(HirExpr::Number(n))
        }
        Lit::Regex(_) | Lit::JSXText(_) => Ok(HirExpr::Undefined),
    }
}

fn transform_binop(op: BinaryOp) -> Result<BinOp> {
    match op {
        BinaryOp::Add => Ok(BinOp::Add),
        BinaryOp::Sub => Ok(BinOp::Sub),
        BinaryOp::Mul => Ok(BinOp::Mul),
        BinaryOp::Div => Ok(BinOp::Div),
        BinaryOp::Mod => Ok(BinOp::Mod),
        BinaryOp::EqEq | BinaryOp::EqEqEq => Ok(BinOp::Eq),
        BinaryOp::NotEq | BinaryOp::NotEqEq => Ok(BinOp::Ne),
        BinaryOp::Lt => Ok(BinOp::Lt),
        BinaryOp::LtEq => Ok(BinOp::Le),
        BinaryOp::Gt => Ok(BinOp::Gt),
        BinaryOp::GtEq => Ok(BinOp::Ge),
        BinaryOp::LogicalAnd => Ok(BinOp::And),
        BinaryOp::LogicalOr => Ok(BinOp::Or),
        BinaryOp::BitAnd => Ok(BinOp::BitAnd),
        BinaryOp::BitOr => Ok(BinOp::BitOr),
        BinaryOp::BitXor => Ok(BinOp::BitXor),
        BinaryOp::LShift => Ok(BinOp::Shl),
        BinaryOp::RShift => Ok(BinOp::Shr),
        _ => Ok(BinOp::Eq),
    }
}

fn transform_assign_target(target: &AssignTarget) -> Result<HirExpr> {
    match target {
        AssignTarget::Simple(s) => match s {
            SimpleAssignTarget::Ident(bi) => Ok(HirExpr::Identifier(bi.id.sym.to_string())),
            SimpleAssignTarget::Member(MemberExpr { obj, prop, .. }) => {
                let obj_hir = transform_expr(obj)?;
                match prop {
                    MemberProp::Ident(IdentName { sym, .. }) => Ok(HirExpr::Property { object: Box::new(obj_hir), name: sym.to_string() }),
                    MemberProp::Computed(ComputedPropName { expr, .. }) => {
                        Ok(HirExpr::Index { object: Box::new(obj_hir), index: Box::new(transform_expr(expr)?) })
                    }
                    MemberProp::PrivateName(_) => Ok(HirExpr::Undefined),
                }
            }
            SimpleAssignTarget::Paren(ParenExpr { expr, .. }) => transform_assign_target_expr(expr),
            SimpleAssignTarget::OptChain(OptChainExpr { base, .. }) => {
                match base.as_ref() {
                    OptChainBase::Member(MemberExpr { obj, prop, .. }) => {
                        let obj_hir = transform_expr(obj)?;
                        match prop {
                            MemberProp::Ident(IdentName { sym, .. }) => Ok(HirExpr::Property { object: Box::new(obj_hir), name: sym.to_string() }),
                            _ => Ok(HirExpr::Undefined),
                        }
                    }
                    _ => Ok(HirExpr::Undefined),
                }
            }
            SimpleAssignTarget::TsAs(TsAsExpr { expr, .. }) => transform_assign_target_expr(expr),
            SimpleAssignTarget::TsNonNull(TsNonNullExpr { expr, .. }) => transform_assign_target_expr(expr),
            SimpleAssignTarget::TsTypeAssertion(TsTypeAssertion { expr, .. }) => transform_assign_target_expr(expr),
            SimpleAssignTarget::TsInstantiation(TsInstantiation { expr, .. }) => transform_assign_target_expr(expr),
            SimpleAssignTarget::TsSatisfies(TsSatisfiesExpr { expr, .. }) => transform_assign_target_expr(expr),
            SimpleAssignTarget::SuperProp(_) | SimpleAssignTarget::Invalid(_) => Ok(HirExpr::Undefined),
        },
        AssignTarget::Pat(pat) => match pat {
            AssignTargetPat::Array(ArrayPat { elems, .. }) => {
                let tmp_name = "_destr_arr_assign".to_string();
                let mut stmts = vec![HirExpr::Var { name: tmp_name.clone(), init: None, is_mut: true }];
                for (i, elem) in elems.iter().enumerate() {
                    if let Some(pat) = elem {
                        if let Pat::Ident(bi) = pat {
                            let var_name = bi.id.sym.to_string();
                            let access = HirExpr::Index { object: Box::new(HirExpr::Identifier(tmp_name.clone())), index: Box::new(HirExpr::Number(i as f64)) };
                            stmts.push(HirExpr::Assign { target: Box::new(HirExpr::Identifier(var_name)), value: Box::new(access) });
                        }
                    }
                }
                Ok(HirExpr::Identifier(tmp_name))
            }
            AssignTargetPat::Object(ObjectPat { props, .. }) => {
                let tmp_name = "_destr_obj_assign".to_string();
                let mut stmts = vec![HirExpr::Var { name: tmp_name.clone(), init: None, is_mut: true }];
                for prop in props {
                    if let ObjectPatProp::KeyValue(KeyValuePatProp { key, value, .. }) = prop {
                        let prop_name = match key {
                            PropName::Ident(id) => id.sym.to_string(),
                            PropName::Str(s) => s.value.to_string_lossy().to_string(),
                            _ => continue,
                        };
                        if let Pat::Ident(bi) = value.as_ref() {
                            let var_name = bi.id.sym.to_string();
                            let access = HirExpr::Property { object: Box::new(HirExpr::Identifier(tmp_name.clone())), name: prop_name };
                            stmts.push(HirExpr::Assign { target: Box::new(HirExpr::Identifier(var_name)), value: Box::new(access) });
                        }
                    }
                }
                Ok(HirExpr::Identifier(tmp_name))
            }
            AssignTargetPat::Invalid(_) => Ok(HirExpr::Undefined),
        },
    }
}

fn transform_assign_target_expr(expr: &Expr) -> Result<HirExpr> {
    transform_expr(expr)
}

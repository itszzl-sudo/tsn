#[allow(dead_code)]
use anyhow::Result;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SyntaxItem {
    pub name: &'static str,
    pub supported: bool,
    pub example: &'static str,
    pub category: &'static str,
}

pub const SYNTAX_LIST: &[SyntaxItem] = &[
    SyntaxItem { name: "function declaration", supported: true, example: "function foo() {}", category: "函数" },
    SyntaxItem { name: "function expression", supported: true, example: "let f = function() {}", category: "函数" },
    SyntaxItem { name: "arrow function", supported: true, example: "(x) => x + 1", category: "函数" },
    SyntaxItem { name: "return", supported: true, example: "return x;", category: "控制流" },
    SyntaxItem { name: "if/else", supported: true, example: "if (x) {} else {}", category: "控制流" },
    SyntaxItem { name: "while", supported: true, example: "while (x) {}", category: "控制流" },
    SyntaxItem { name: "for", supported: true, example: "for (let i=0; i<n; i++) {}", category: "控制流" },
    SyntaxItem { name: "for...in", supported: true, example: "for (let k in obj) {}", category: "控制流" },
    SyntaxItem { name: "for...of", supported: true, example: "for (let x of arr) {}", category: "控制流" },
    SyntaxItem { name: "break/continue", supported: true, example: "break; continue;", category: "控制流" },
    SyntaxItem { name: "try/catch/finally", supported: true, example: "try {} catch(e) {} finally {}", category: "控制流" },
    SyntaxItem { name: "throw", supported: true, example: "throw new Error()", category: "控制流" },
    SyntaxItem { name: "ternary (?:)", supported: true, example: "x ? a : b", category: "表达式" },
    SyntaxItem { name: "let/const/var", supported: true, example: "let x = 1; const y = 2;", category: "声明" },
    SyntaxItem { name: "assignment", supported: true, example: "x = 1; x += 2;", category: "表达式" },
    SyntaxItem { name: "binary operators", supported: true, example: "+, -, *, /, %, ==, <, >, &&, ||", category: "表达式" },
    SyntaxItem { name: "unary operators", supported: true, example: "-x, !x, typeof x", category: "表达式" },
    SyntaxItem { name: "increment/decrement", supported: true, example: "x++; x--;", category: "表达式" },
    SyntaxItem { name: "string literal", supported: true, example: "\"hello\", 'world'", category: "字面量" },
    SyntaxItem { name: "template literal", supported: true, example: "`hello ${name}`", category: "字面量" },
    SyntaxItem { name: "number literal", supported: true, example: "42, 3.14, 0xFF", category: "字面量" },
    SyntaxItem { name: "boolean literal", supported: true, example: "true, false", category: "字面量" },
    SyntaxItem { name: "null/undefined", supported: true, example: "null, undefined", category: "字面量" },
    SyntaxItem { name: "array literal", supported: true, example: "[1, 2, 3]", category: "字面量" },
    SyntaxItem { name: "object literal", supported: true, example: "{ key: value }", category: "字面量" },
    SyntaxItem { name: "property access", supported: true, example: "obj.key, obj[0]", category: "表达式" },
    SyntaxItem { name: "function call", supported: true, example: "foo(1, 2)", category: "表达式" },
    SyntaxItem { name: "method call", supported: true, example: "obj.method()", category: "表达式" },
    SyntaxItem { name: "import declaration", supported: true, example: "import { x } from 'mod'", category: "模块" },
    SyntaxItem { name: "export declaration", supported: true, example: "export function foo() {}", category: "模块" },
    SyntaxItem { name: "declare function", supported: true, example: "declare function foo(): void;", category: "TS扩展" },
    SyntaxItem { name: "interface (declare only)", supported: true, example: "interface Config { x: number }", category: "TS扩展" },
    SyntaxItem { name: "type annotation", supported: true, example: "let x: number = 1;", category: "TS扩展" },
    SyntaxItem { name: "as type assertion", supported: true, example: "x as number", category: "TS扩展" },
    SyntaxItem { name: "typeof operator", supported: true, example: "typeof x", category: "表达式" },
    SyntaxItem { name: "console.log/print", supported: true, example: "console.log(x), print(x)", category: "内置" },
    SyntaxItem { name: "Math methods", supported: true, example: "Math.sin, Math.random, ...", category: "内置" },
    SyntaxItem { name: "JSON.parse/stringify", supported: true, example: "JSON.parse(s), JSON.stringify(o)", category: "内置" },
    SyntaxItem { name: "Date.now", supported: true, example: "Date.now()", category: "内置" },
    SyntaxItem { name: "parseInt/parseFloat", supported: true, example: "parseInt(s)", category: "内置" },
    SyntaxItem { name: "class", supported: false, example: "class Foo {}", category: "面向对象" },
    SyntaxItem { name: "class inheritance", supported: false, example: "class Bar extends Foo {}", category: "面向对象" },
    SyntaxItem { name: "decorator", supported: false, example: "@decorator class Foo {}", category: "面向对象" },
    SyntaxItem { name: "enum", supported: true, example: "enum E { A, B }", category: "声明" },
    SyntaxItem { name: "async/await", supported: false, example: "async function f() { await p }", category: "异步" },
    SyntaxItem { name: "generator", supported: false, example: "function* g() { yield 1 }", category: "函数" },
    SyntaxItem { name: "Promise", supported: false, example: "new Promise((r) => r(1))", category: "异步" },
    SyntaxItem { name: "namespace", supported: false, example: "namespace N { }", category: "TS扩展" },
    SyntaxItem { name: "abstract class", supported: false, example: "abstract class A {}", category: "面向对象" },
    SyntaxItem { name: "with statement", supported: false, example: "with (obj) { }", category: "遗留" },
    SyntaxItem { name: "labeled statement", supported: false, example: "label: for (;;) { break label; }", category: "控制流" },
    SyntaxItem { name: "switch", supported: true, example: "switch(x) { case 1: break; }", category: "控制流" },
    SyntaxItem { name: "do...while", supported: true, example: "do {} while (x)", category: "控制流" },
    SyntaxItem { name: "new expression", supported: true, example: "new Foo()", category: "表达式" },
    SyntaxItem { name: "delete/void/in", supported: true, example: "delete obj.key; void 0; x in obj", category: "表达式" },
    SyntaxItem { name: "spread/rest", supported: true, example: "...args, fn(...a)", category: "表达式" },
    SyntaxItem { name: "destructuring", supported: true, example: "let { a, b } = obj;", category: "声明" },
    SyntaxItem { name: "default param", supported: true, example: "function f(x = 1) {}", category: "函数" },
    SyntaxItem { name: "computed property", supported: true, example: "{ [key]: value }", category: "表达式" },
    SyntaxItem { name: "optional chaining", supported: true, example: "obj?.key?.method()", category: "表达式" },
    SyntaxItem { name: "nullish coalescing", supported: true, example: "x ?? y", category: "表达式" },
    SyntaxItem { name: "regex literal", supported: false, example: "/pattern/flags", category: "字面量" },
    SyntaxItem { name: "satisfies operator", supported: false, example: "x as const satisfies T", category: "TS扩展" },
    SyntaxItem { name: "type alias (body)", supported: false, example: "type T = { a: number }", category: "TS扩展" },
];

pub fn generate_markdown() -> String {
    let mut md = String::new();
    md.push_str("# ts-native 语法支持列表\n\n");
    md.push_str("| 语法 | 状态 | 示例 | 分类 |\n");
    md.push_str("|------|------|------|------|\n");
    for item in SYNTAX_LIST {
        let status = if item.supported { "✅" } else { "❌" };
        md.push_str(&format!("| {} | {} | `{}` | {} |\n", item.name, status, item.example, item.category));
    }
    md
}

#[derive(Debug)]
pub struct UnsupportedSyntax {
    pub name: String,
    pub line: usize,
    pub col: usize,
}

pub fn check_syntax(source: &str) -> Result<Vec<UnsupportedSyntax>> {
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
    let module = parser.parse_module().map_err(|e| {
        anyhow::anyhow!("swc 解析失败: {:?}", e)
    })?;

    let mut unsupported = Vec::new();
    check_module(&module, &cm, &mut unsupported);
    Ok(unsupported)
}

fn check_module(module: &Module, cm: &SourceMap, out: &mut Vec<UnsupportedSyntax>) {
    for item in &module.body {
        check_module_item(item, cm, out);
    }
}

fn check_module_item(item: &ModuleItem, cm: &SourceMap, out: &mut Vec<UnsupportedSyntax>) {
    match item {
        ModuleItem::ModuleDecl(decl) => check_module_decl(decl, cm, out),
        ModuleItem::Stmt(stmt) => check_stmt(stmt, cm, out),
    }
}

fn check_module_decl(decl: &ModuleDecl, cm: &SourceMap, out: &mut Vec<UnsupportedSyntax>) {
    match decl {
        ModuleDecl::Import(_) => {}
        ModuleDecl::ExportDecl(_) => {}
        ModuleDecl::ExportDefaultDecl(e) => {
            match &e.decl {
                DefaultDecl::Class(c) => add("class", c.class.span, cm, out),
                DefaultDecl::Fn(_) => {}
                DefaultDecl::TsInterfaceDecl(_) => {}
            }
        }
        ModuleDecl::ExportDefaultExpr(_) => {}
        ModuleDecl::ExportNamed(_) => {}
        ModuleDecl::ExportAll(_) => {}
        ModuleDecl::TsImportEquals(_) => {}
        ModuleDecl::TsExportAssignment(_) => {}
        ModuleDecl::TsNamespaceExport(_) => {}
    }
}

fn check_stmt(stmt: &Stmt, cm: &SourceMap, out: &mut Vec<UnsupportedSyntax>) {
    match stmt {
        Stmt::Block(b) => { for s in &b.stmts { check_stmt(s, cm, out); } }
        Stmt::Decl(decl) => check_decl(decl, cm, out),
        Stmt::Expr(_) => {}
        Stmt::If(i) => {
            check_stmt(&i.cons, cm, out);
            if let Some(a) = &i.alt { check_stmt(a, cm, out); }
        }
        Stmt::Return(_) => {}
        Stmt::While(w) => { check_stmt(&w.body, cm, out); }
        Stmt::DoWhile(d) => { check_stmt(&d.body, cm, out); }
        Stmt::For(f) => { check_stmt(&f.body, cm, out); }
        Stmt::ForIn(_) => {}
        Stmt::ForOf(_) => {}
        Stmt::Continue(_) => {}
        Stmt::Break(_) => {}
        Stmt::Switch(s) => {
            for c in &s.cases {
                for st in &c.cons { check_stmt(st, cm, out); }
            }
        }
        Stmt::Throw(_) => {}
        Stmt::Try(t) => {
            for s in &t.block.stmts { check_stmt(s, cm, out); }
            if let Some(h) = &t.handler { for s in &h.body.stmts { check_stmt(s, cm, out); } }
            if let Some(f) = &t.finalizer { for s in &f.stmts { check_stmt(s, cm, out); } }
        }
        Stmt::With(w) => { add("with statement", w.span, cm, out); }
        Stmt::Labeled(l) => { add("labeled statement", l.span, cm, out); }
        Stmt::Empty(_) | Stmt::Debugger(_) => {}
    }
}

fn check_decl(decl: &Decl, cm: &SourceMap, out: &mut Vec<UnsupportedSyntax>) {
    match decl {
        Decl::Class(c) => { add("class", c.class.span, cm, out); }
        Decl::Fn(_) => {}
        Decl::Var(_) => {}
        Decl::TsInterface(_) => {}
        Decl::TsEnum(_) => {}
        Decl::TsModule(m) => { add("namespace", m.span, cm, out); }
        Decl::TsTypeAlias(a) => { add("type alias (body)", a.span, cm, out); }
        Decl::Using(_) => { add("using declaration", swc_common::Span::default(), cm, out); }

    }
}

fn add(name: &str, span: swc_common::Span, cm: &SourceMap, out: &mut Vec<UnsupportedSyntax>) {
    let loc = cm.lookup_char_pos(span.lo);
    out.push(UnsupportedSyntax {
        name: name.to_string(),
        line: loc.line,
        col: loc.col_display,
    });
}

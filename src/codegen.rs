#[allow(dead_code)]
use anyhow::Result;
use cranelift::prelude::*;
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

pub const UNDEFINED: u64 = 0x7FFF_8000_0000_0001;
pub const NULL: u64 = 0x7FFF_8000_0000_0002;
pub const TRUE: u64 = 0x7FFF_0000_0000_0001;
pub const FALSE: u64 = 0x7FFF_0000_0000_0000;
pub const STRING_TAG: u64 = 0x7FFC_0000_0000_0000;
pub const ARRAY_TAG: u64 = 0x7FFB_0000_0000_0000;
pub const OBJECT_TAG: u64 = 0x7FFA_0000_0000_0000;

// 标准事件类型
pub const EVENT_CLICK: u64 = 1;
pub const EVENT_KEYDOWN: u64 = 2;
pub const EVENT_KEYUP: u64 = 3;
pub const EVENT_MOUSEMOVE: u64 = 4;
pub const EVENT_CHANGE: u64 = 5;
pub const EVENT_SUBMIT: u64 = 6;
pub const EVENT_FOCUS: u64 = 7;
pub const EVENT_BLUR: u64 = 8;
pub const EVENT_CHANGE_VALUE: u64 = 9;

fn map_event_type(event_name: &str) -> u64 {
    match event_name {
        "click" => EVENT_CLICK,
        "keydown" => EVENT_KEYDOWN,
        "keyup" => EVENT_KEYUP,
        "mousemove" => EVENT_MOUSEMOVE,
        "change" => EVENT_CHANGE,
        "submit" => EVENT_SUBMIT,
        "focus" => EVENT_FOCUS,
        "blur" => EVENT_BLUR,
        "changeValue" => EVENT_CHANGE_VALUE,
        _ => 0,  // 未知事件，运行时处理
    }
}

#[derive(Debug, Clone)]
pub enum HirExpr {
    Number(f64),
    Boolean(bool),
    String(String),
    Null,
    Undefined,
    Identifier(String),
    
    Binary { op: BinOp, left: Box<HirExpr>, right: Box<HirExpr> },
    Unary { op: UnaryOp, operand: Box<HirExpr> },
    Typeof(Box<HirExpr>),
    
    Call { callee: Box<HirExpr>, args: Vec<HirExpr> },
    
    Ternary { cond: Box<HirExpr>, then_expr: Box<HirExpr>, else_expr: Box<HirExpr> },
    
    Function { name: String, params: Vec<String>, body: Vec<HirExpr> },
    
    Return(Option<Box<HirExpr>>),
    Var { name: String, init: Option<Box<HirExpr>>, is_mut: bool },
    
    If { cond: Box<HirExpr>, then_body: Vec<HirExpr>, else_body: Option<Vec<HirExpr>> },
    While { cond: Box<HirExpr>, body: Vec<HirExpr> },
    For { init: Option<Box<HirExpr>>, cond: Option<Box<HirExpr>>, update: Option<Box<HirExpr>>, body: Vec<HirExpr> },
    
    Block(Vec<HirExpr>),
    
    Break,
    Continue,
    
    Assign { target: Box<HirExpr>, value: Box<HirExpr> },
    
    Array(Vec<HirExpr>),
    Index { object: Box<HirExpr>, index: Box<HirExpr> },
    
    Object(Vec<(String, HirExpr)>),
    Property { object: Box<HirExpr>, name: String },
    
    TryCatch {
        try_body: Vec<HirExpr>,
        catch_param: Option<String>,
        catch_body: Option<Vec<HirExpr>>,
        finally_body: Option<Vec<HirExpr>>,
    },
    Throw(Option<Box<HirExpr>>),
    ExprStmt(Box<HirExpr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Not, Neg, PreInc, PreDec, PostInc, PostDec,
}

/// FNV-1a 64-bit hash for property key strings.
/// More robust than djb2 for avoiding collisions with similar property names.
fn fnv1a_64(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in name.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub struct CodeGen {
    builder_context: FunctionBuilderContext,
    function_ids: std::collections::HashMap<String, cranelift_module::FuncId>,
    function_param_counts: std::collections::HashMap<String, usize>,
    string_pool: std::collections::HashSet<String>,
    external_functions: std::collections::HashMap<String, String>,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            builder_context: FunctionBuilderContext::new(),
            function_ids: std::collections::HashMap::new(),
            function_param_counts: std::collections::HashMap::new(),
            string_pool: std::collections::HashSet::new(),
            external_functions: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_external_functions(mut self, functions: std::collections::HashMap<String, String>) -> Self {
        self.external_functions = functions;
        self
    }
    
    pub fn set_external_functions(&mut self, functions: std::collections::HashMap<String, String>) {
        self.external_functions = functions;
    }
    
    pub fn compile(&mut self, exprs: &[HirExpr]) -> Result<Vec<u8>> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa = cranelift_native::builder()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let isa = isa.finish(settings::Flags::new(flag_builder))?;
        
        let builder = ObjectBuilder::new(
            isa,
            "ts_native",
            cranelift_module::default_libcall_names(),
        )?;
        let mut module: ObjectModule = ObjectModule::new(builder);
        
        // 第一遍：收集所有函数签名和字符串常量
        for expr in exprs {
            if let HirExpr::Function { name, params, body: _ } = expr {
                let mut sig = module.make_signature();
                for _ in params {
                    sig.params.push(AbiParam::new(types::F64));
                }
                sig.returns.push(AbiParam::new(types::F64));
                
                let func_id = module.declare_function(name, Linkage::Export, &sig)?;
                self.function_ids.insert(name.clone(), func_id);
                self.function_param_counts.insert(name.clone(), params.len());
            }
            self.collect_strings(expr);
        }
        
        // 创建字符串数据对象
        let mut string_data: std::collections::HashMap<String, (cranelift_module::DataId, Vec<u8>)> = std::collections::HashMap::new();
        for string in self.string_pool.iter() {
            let data_name = format!("str_{}", string_data.len());
            let data_id = module.declare_data(&data_name, Linkage::Local, false, false)?;
            
            let mut data_ctx = DataContext::new();
            let mut bytes: Vec<u8> = string.bytes().collect();
            bytes.push(0);
            data_ctx.define(bytes.clone().into_boxed_slice());
            
            string_data.insert(string.clone(), (data_id, bytes));
            module.define_data(data_id, &data_ctx)?;
        }
        
        // 第二遍：编译函数体
        for expr in exprs {
            if let HirExpr::Function { name, params, body } = expr {
                self.compile_function(&mut module, name, params, body, &string_data)?;
            }
        }
        
        let product = module.finish();
        Ok(product.emit()?)
    }
    
    fn collect_strings(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::String(s) => {
                self.string_pool.insert(s.clone());
            }
            HirExpr::Binary { left, right, .. } => {
                self.collect_strings(left);
                self.collect_strings(right);
            }
            HirExpr::Unary { operand, .. } => {
                self.collect_strings(operand);
            }
            HirExpr::Typeof(expr) => {
                self.collect_strings(expr);
            }
            HirExpr::Call { callee, args } => {
                self.collect_strings(callee);
                for arg in args {
                    self.collect_strings(arg);
                }
            }
            HirExpr::Ternary { cond, then_expr, else_expr } => {
                self.collect_strings(cond);
                self.collect_strings(then_expr);
                self.collect_strings(else_expr);
            }
            HirExpr::Function { body, .. } => {
                for stmt in body {
                    self.collect_strings(stmt);
                }
            }
            HirExpr::Return(val) => {
                if let Some(v) = val {
                    self.collect_strings(v);
                }
            }
            HirExpr::Var { init, .. } => {
                if let Some(i) = init {
                    self.collect_strings(i);
                }
            }
            HirExpr::If { cond, then_body, else_body } => {
                self.collect_strings(cond);
                for stmt in then_body {
                    self.collect_strings(stmt);
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.collect_strings(stmt);
                    }
                }
            }
            HirExpr::While { cond, body } => {
                self.collect_strings(cond);
                for stmt in body {
                    self.collect_strings(stmt);
                }
            }
            HirExpr::For { init, cond, update, body } => {
                if let Some(i) = init {
                    self.collect_strings(i);
                }
                if let Some(c) = cond {
                    self.collect_strings(c);
                }
                if let Some(u) = update {
                    self.collect_strings(u);
                }
                for stmt in body {
                    self.collect_strings(stmt);
                }
            }
            HirExpr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_strings(stmt);
                }
            }
            HirExpr::Assign { target, value } => {
                self.collect_strings(target);
                self.collect_strings(value);
            }
            HirExpr::Array(elements) => {
                for elem in elements {
                    self.collect_strings(elem);
                }
            }
            HirExpr::Index { object, index } => {
                self.collect_strings(object);
                self.collect_strings(index);
            }
            HirExpr::Object(properties) => {
                for (_, value) in properties {
                    self.collect_strings(value);
                }
            }
            HirExpr::Property { object, .. } => {
                self.collect_strings(object);
            }
            HirExpr::TryCatch { try_body, catch_body, finally_body, .. } => {
                for stmt in try_body {
                    self.collect_strings(stmt);
                }
                if let Some(body) = catch_body {
                    for stmt in body {
                        self.collect_strings(stmt);
                    }
                }
                if let Some(body) = finally_body {
                    for stmt in body {
                        self.collect_strings(stmt);
                    }
                }
            }
            HirExpr::Throw(val) => {
                if let Some(v) = val {
                    self.collect_strings(v);
                }
            }
            HirExpr::ExprStmt(expr) => {
                self.collect_strings(expr);
            }
            _ => {}
        }
    }

    fn compile_function(
        &mut self,
        module: &mut ObjectModule,
        name: &str,
        params: &[String],
        body: &[HirExpr],
        string_data: &std::collections::HashMap<String, (cranelift_module::DataId, Vec<u8>)>,
    ) -> Result<()> {
        let func_id = *self.function_ids.get(name)
            .ok_or_else(|| anyhow::anyhow!("Function {} not found", name))?;
        
        let mut ctx = module.make_context();
        let param_count = params.len();
        for _ in 0..param_count {
            ctx.func.signature.params.push(AbiParam::new(types::F64));
        }
        ctx.func.signature.returns.push(AbiParam::new(types::F64));
        
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_context);
        
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        
        let mut variables = std::collections::HashMap::new();
        for (i, param) in params.iter().enumerate() {
            let var = Variable::new(i);
            variables.insert(param.clone(), var);
            let val = builder.block_params(entry_block)[i];
            builder.declare_var(var, types::F64);
            builder.def_var(var, val);
        }
        
        let mut has_return = false;
        for stmt in body.iter() {
            if compile_stmt(stmt, &mut builder, &mut variables, module, &self.function_ids, &self.function_param_counts, string_data) {
                has_return = true;
                break;
            }
        }
        
        if !has_return {
            let undef = builder.ins().f64const(f64::from_bits(UNDEFINED));
            builder.ins().return_(&[undef]);
        }
        
        builder.finalize();
        
        module.define_function(func_id, &mut ctx)?;
        module.clear_context(&mut ctx);
        
        Ok(())
    }
}

fn compile_stmt(
    stmt: &HirExpr,
    builder: &mut FunctionBuilder,
    variables: &mut std::collections::HashMap<String, Variable>,
    module: &mut ObjectModule,
    function_ids: &std::collections::HashMap<String, cranelift_module::FuncId>,
    function_param_counts: &std::collections::HashMap<String, usize>,
    _string_data: &std::collections::HashMap<String, (cranelift_module::DataId, Vec<u8>)>,
) -> bool {
    match stmt {
        HirExpr::Return(val) => {
            if let Some(v) = val {
                let result = compile_expr(v, builder, variables, module, function_ids, function_param_counts, _string_data);
                builder.ins().return_(&[result]);
            } else {
                let undef = builder.ins().f64const(f64::from_bits(UNDEFINED));
                builder.ins().return_(&[undef]);
            }
            true
        }
        HirExpr::Var { name, init, is_mut: _ } => {
            let val = if let Some(i) = init {
                compile_expr(i, builder, variables, module, function_ids, function_param_counts, _string_data)
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            };
            
            let var_idx = variables.len();
            let var = Variable::new(var_idx);
            variables.insert(name.clone(), var);
            builder.declare_var(var, types::F64);
            builder.def_var(var, val);
            false
        }
        HirExpr::If { cond, then_body, else_body } => {
            let cond_val = compile_expr(cond, builder, variables, module, function_ids, function_param_counts, _string_data);
            let cond_bool = is_truthy(cond_val, builder, module);
            
            let then_block = builder.create_block();
            let else_block = builder.create_block();
            let merge_block = builder.create_block();
            
            builder.ins().brif(cond_bool, then_block, &[], else_block, &[]);
            
            builder.switch_to_block(then_block);
            builder.seal_block(then_block);
            let mut then_returns = false;
            for s in then_body {
                if compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data) {
                    then_returns = true;
                }
            }
            if !then_returns {
                builder.ins().jump(merge_block, &[]);
            }
            
            builder.switch_to_block(else_block);
            builder.seal_block(else_block);
            let mut else_returns = false;
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    if compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data) {
                        else_returns = true;
                    }
                }
            }
            if !else_returns {
                builder.ins().jump(merge_block, &[]);
            }
            
            if !then_returns || !else_returns {
                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
            }
            
            then_returns && else_returns
        }
        HirExpr::While { cond, body } => {
            let header_block = builder.create_block();
            let body_block = builder.create_block();
            let exit_block = builder.create_block();
            
            builder.ins().jump(header_block, &[]);
            
            builder.switch_to_block(header_block);
            let cond_val = compile_expr(cond, builder, variables, module, function_ids, function_param_counts, _string_data);
            let cond_bool = is_truthy(cond_val, builder, module);
            builder.ins().brif(cond_bool, body_block, &[], exit_block, &[]);
            
            builder.switch_to_block(body_block);
            for s in body {
                compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data);
            }
            builder.ins().jump(header_block, &[]);
            
            builder.seal_block(header_block);
            builder.seal_block(body_block);
            
            builder.switch_to_block(exit_block);
            builder.seal_block(exit_block);
            
            false
        }
        HirExpr::For { init, cond, update, body } => {
            if let Some(i) = init {
                compile_stmt(i, builder, variables, module, function_ids, function_param_counts, _string_data);
            }
            
            let header_block = builder.create_block();
            let body_block = builder.create_block();
            let update_block = builder.create_block();
            let exit_block = builder.create_block();
            
            builder.ins().jump(header_block, &[]);
            
            builder.switch_to_block(header_block);
            if let Some(c) = cond {
                let cond_val = compile_expr(c, builder, variables, module, function_ids, function_param_counts, _string_data);
                let cond_bool = is_truthy(cond_val, builder, module);
                builder.ins().brif(cond_bool, body_block, &[], exit_block, &[]);
            } else {
                builder.ins().jump(body_block, &[]);
            }
            
            builder.switch_to_block(body_block);
            for s in body {
                compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data);
            }
            builder.ins().jump(update_block, &[]);
            
            builder.switch_to_block(update_block);
            if let Some(u) = update {
                compile_stmt(u, builder, variables, module, function_ids, function_param_counts, _string_data);
            }
            builder.ins().jump(header_block, &[]);
            
            builder.seal_block(header_block);
            builder.seal_block(body_block);
            builder.seal_block(update_block);
            
            builder.switch_to_block(exit_block);
            builder.seal_block(exit_block);
            
            false
        }
        HirExpr::Block(stmts) => {
            let mut has_return = false;
            for s in stmts {
                if compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data) {
                    has_return = true;
                }
            }
            has_return
        }
        HirExpr::Assign { target, value } => {
            let val = compile_expr(value, builder, variables, module, function_ids, function_param_counts, _string_data);
            match target.as_ref() {
                HirExpr::Identifier(name) => {
                    if let Some(&var) = variables.get(name) {
                        builder.def_var(var, val);
                    }
                }
                HirExpr::Index { object, index } => {
                    let obj_val = compile_expr(object, builder, variables, module, function_ids, function_param_counts, _string_data);
                    let idx_val = compile_expr(index, builder, variables, module, function_ids, function_param_counts, _string_data);
                    
                    let mut sig_set = module.make_signature();
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.returns.push(AbiParam::new(types::F64));
                    
                    if let Ok(id) = module.declare_function("js_array_set", cranelift_module::Linkage::Import, &sig_set) {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        builder.ins().call(func_ref, &[obj_val, idx_val, val]);
                    }
                }
                HirExpr::Property { object, name } => {
                    let obj_val = compile_expr(object, builder, variables, module, function_ids, function_param_counts, _string_data);
                    
                    let mut sig_set = module.make_signature();
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.returns.push(AbiParam::new(types::F64));
                    
                    if let Ok(id) = module.declare_function("js_object_set", cranelift_module::Linkage::Import, &sig_set) {
                        // NOTE: Property keys are stored as string hashes (not real JsString pointers).
                        // The C runtime compares keys by 64-bit value, not by dereferencing.
                        let hash = fnv1a_64(name);
                        let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF)));
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        builder.ins().call(func_ref, &[obj_val, key_val, val]);
                    }
                }
                _ => {}
            }
            false
        }
        HirExpr::TryCatch { try_body, catch_param, catch_body, finally_body } => {
            // try-catch 实现策略：使用运行时函数 js_try_push / js_try_pop / js_throw
            // 
            // 生成代码结构：
            //   js_try_push(jmpbuf_ptr)
            //   if (setjmp == 0) { try_body } else { catch_body(exception) }
            //   js_try_pop()
            //   finally_body
            //
            // 简化实现：使用运行时的 js_try_begin/js_try_end 包装，
            // 异常通过 js_throw 设置全局异常值，try 块内检测异常后跳转到 catch

            // 声明运行时函数
            let mut sig_try_begin = module.make_signature();
            sig_try_begin.returns.push(AbiParam::new(types::I32));
            let try_begin_id = module.declare_function("js_try_begin", cranelift_module::Linkage::Import, &sig_try_begin);

            let sig_try_end = module.make_signature();
            let try_end_id = module.declare_function("js_try_end", cranelift_module::Linkage::Import, &sig_try_end);

            let mut sig_get_exception = module.make_signature();
            sig_get_exception.returns.push(AbiParam::new(types::F64));
            let get_exception_id = module.declare_function("js_get_exception", cranelift_module::Linkage::Import, &sig_get_exception);

            let sig_clear_exception = module.make_signature();
            let clear_exception_id = module.declare_function("js_clear_exception", cranelift_module::Linkage::Import, &sig_clear_exception);

            let try_block = builder.create_block();
            let catch_block = builder.create_block();
            let finally_block = builder.create_block();
            let merge_block = builder.create_block();

            // js_try_begin() -> i32 (0 = 正常, 1 = 有异常)
            let entered_normally = if let Ok(id) = try_begin_id {
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                let call = builder.ins().call(func_ref, &[]);
                builder.inst_results(call)[0]
            } else {
                builder.ins().iconst(types::I32, 0)
            };

            let zero = builder.ins().iconst(types::I32, 0);
            let is_normal = builder.ins().icmp(IntCC::Equal, entered_normally, zero);
            builder.ins().brif(is_normal, try_block, &[], catch_block, &[]);

            // try 块
            builder.switch_to_block(try_block);
            builder.seal_block(try_block);
            let mut try_returns = false;
            for s in try_body {
                if compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data) {
                    try_returns = true;
                    break;
                }
            }
            // try 块正常完成 -> 跳到 finally
            if !try_returns {
                // js_try_end()
                if let Ok(id) = try_end_id {
                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                    builder.ins().call(func_ref, &[]);
                }
                // js_clear_exception()
                if let Ok(id) = clear_exception_id {
                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                    builder.ins().call(func_ref, &[]);
                }
                builder.ins().jump(finally_block, &[]);
            }

            // catch 块
            builder.switch_to_block(catch_block);
            builder.seal_block(catch_block);
            if let Some(body) = catch_body {
                // 将异常值赋给 catch 参数
                if let Some(param) = catch_param {
                    let exception_val = if let Ok(id) = get_exception_id {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        let call = builder.ins().call(func_ref, &[]);
                        builder.inst_results(call)[0]
                    } else {
                        builder.ins().f64const(f64::from_bits(UNDEFINED))
                    };

                    let var_idx = variables.len();
                    let var = Variable::new(var_idx);
                    variables.insert(param.clone(), var);
                    builder.declare_var(var, types::F64);
                    builder.def_var(var, exception_val);
                }
                // js_clear_exception()
                if let Ok(id) = clear_exception_id {
                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                    builder.ins().call(func_ref, &[]);
                }
                for s in body {
                    compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data);
                }
            }
            // js_try_end()
            if let Ok(id) = try_end_id {
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                builder.ins().call(func_ref, &[]);
            }
            builder.ins().jump(finally_block, &[]);

            // finally 块
            builder.switch_to_block(finally_block);
            builder.seal_block(finally_block);
            if let Some(body) = finally_body {
                for s in body {
                    compile_stmt(s, builder, variables, module, function_ids, function_param_counts, _string_data);
                }
            }
            builder.ins().jump(merge_block, &[]);

            builder.switch_to_block(merge_block);
            builder.seal_block(merge_block);

            try_returns
        }
        HirExpr::Throw(val) => {
            let throw_val = if let Some(v) = val {
compile_expr(v, builder, variables, module, function_ids, function_param_counts, _string_data)
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            };

            // 调用 js_throw(value) - 设置异常值
            let mut sig_throw = module.make_signature();
            sig_throw.params.push(AbiParam::new(types::F64));
            if let Ok(id) = module.declare_function("js_throw", cranelift_module::Linkage::Import, &sig_throw) {
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                builder.ins().call(func_ref, &[throw_val]);
            }

            // throw 后返回 undefined（异常值已设置，调用者会检查）
            let undef_val = builder.ins().f64const(f64::from_bits(UNDEFINED));
            builder.ins().return_(&[undef_val]);

            true // throw 终止当前执行路径
        }
        HirExpr::ExprStmt(expr) => {
            match expr.as_ref() {
                HirExpr::Assign { .. } | HirExpr::Var { .. } | HirExpr::Return(_) | HirExpr::Break | HirExpr::Continue | HirExpr::Throw(_) => {
                    compile_stmt(expr, builder, variables, module, function_ids, function_param_counts, _string_data)
                }
                _ => {
                    compile_expr(expr, builder, variables, module, function_ids, function_param_counts, _string_data);
                    false
                }
            }
        }
        _ => {
            compile_expr(stmt, builder, variables, module, function_ids, function_param_counts, _string_data);
            false
        }
    }
}

fn is_truthy(val: Value, builder: &mut FunctionBuilder, module: &mut ObjectModule) -> Value {
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(types::F64));
    sig.returns.push(AbiParam::new(types::I32));
    if let Ok(id) = module.declare_function("js_is_truthy", cranelift_module::Linkage::Import, &sig) {
        let func_ref = module.declare_func_in_func(id, &mut builder.func);
        let call = builder.ins().call(func_ref, &[val]);
        let i32_result = builder.inst_results(call)[0];
        let zero_i32 = builder.ins().iconst(types::I32, 0);
        builder.ins().icmp(IntCC::NotEqual, i32_result, zero_i32)
    } else {
        let zero = builder.ins().f64const(0.0);
        builder.ins().fcmp(FloatCC::NotEqual, val, zero)
    }
}

fn compile_expr(
    expr: &HirExpr,
    builder: &mut FunctionBuilder,
    variables: &std::collections::HashMap<String, Variable>,
    module: &mut ObjectModule,
    function_ids: &std::collections::HashMap<String, cranelift_module::FuncId>,
    function_param_counts: &std::collections::HashMap<String, usize>,
    _string_data: &std::collections::HashMap<String, (cranelift_module::DataId, Vec<u8>)>,
) -> Value {
    match expr {
        HirExpr::Number(n) => builder.ins().f64const(*n),
        HirExpr::Boolean(true) => builder.ins().f64const(f64::from_bits(TRUE)),
        HirExpr::Boolean(false) => builder.ins().f64const(f64::from_bits(FALSE)),
        HirExpr::Null => builder.ins().f64const(f64::from_bits(NULL)),
        HirExpr::Undefined => builder.ins().f64const(f64::from_bits(UNDEFINED)),
        HirExpr::String(s) => {
            if let Some((data_id, _bytes)) = _string_data.get(s) {
                let gv = module.declare_data_in_func(*data_id, &mut builder.func);
                let addr = builder.ins().global_value(types::I64, gv);
                
                let mut sig = module.make_signature();
                sig.params.push(AbiParam::new(types::I64));
                sig.returns.push(AbiParam::new(types::F64));
                
                if let Ok(id) = module.declare_function("js_string_from_static", cranelift_module::Linkage::Import, &sig) {
                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                    let call = builder.ins().call(func_ref, &[addr]);
                    builder.inst_results(call)[0]
                } else {
                    builder.ins().f64const(f64::from_bits(UNDEFINED))
                }
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }

        
        HirExpr::Identifier(name) => {
            if let Some(&var) = variables.get(name) {
                builder.use_var(var)
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }
        
        HirExpr::Binary { op, left, right } => {
            let l = compile_expr(left, builder, variables, module, function_ids, function_param_counts, _string_data);
            let r = compile_expr(right, builder, variables, module, function_ids, function_param_counts, _string_data);
            
            match op {
                BinOp::Add => {
                    let mut sig = module.make_signature();
                    sig.params.push(AbiParam::new(types::F64));
                    sig.params.push(AbiParam::new(types::F64));
                    sig.returns.push(AbiParam::new(types::F64));
                    
                    if let Ok(id) = module.declare_function("js_add", cranelift_module::Linkage::Import, &sig) {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        let call = builder.ins().call(func_ref, &[l, r]);
                        builder.inst_results(call)[0]
                    } else {
                        builder.ins().fadd(l, r)
                    }
                }
                BinOp::Sub => builder.ins().fsub(l, r),
                BinOp::Mul => builder.ins().fmul(l, r),
                BinOp::Div => builder.ins().fdiv(l, r),
                BinOp::Mod => {
                    let mut sig = module.make_signature();
                    sig.params.push(AbiParam::new(types::F64));
                    sig.params.push(AbiParam::new(types::F64));
                    sig.returns.push(AbiParam::new(types::F64));
                    
                    if let Ok(id) = module.declare_function("js_mod", cranelift_module::Linkage::Import, &sig) {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        let call = builder.ins().call(func_ref, &[l, r]);
                        builder.inst_results(call)[0]
                    } else {
                        builder.ins().f64const(f64::from_bits(UNDEFINED))
                    }
                }
                
                BinOp::Eq => {
                    let mut sig = module.make_signature();
                    sig.params.push(AbiParam::new(types::F64));
                    sig.params.push(AbiParam::new(types::F64));
                    sig.returns.push(AbiParam::new(types::F64));
                    if let Ok(id) = module.declare_function("js_eq", cranelift_module::Linkage::Import, &sig) {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        let call = builder.ins().call(func_ref, &[l, r]);
                        builder.inst_results(call)[0]
                    } else {
                        let cmp = builder.ins().fcmp(FloatCC::Equal, l, r);
                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        builder.ins().select(cmp, one, zero)
                    }
                }
                BinOp::Ne => {
                    let mut sig = module.make_signature();
                    sig.params.push(AbiParam::new(types::F64));
                    sig.params.push(AbiParam::new(types::F64));
                    sig.returns.push(AbiParam::new(types::F64));
                    if let Ok(id) = module.declare_function("js_neq", cranelift_module::Linkage::Import, &sig) {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        let call = builder.ins().call(func_ref, &[l, r]);
                        builder.inst_results(call)[0]
                    } else {
                        let cmp = builder.ins().fcmp(FloatCC::NotEqual, l, r);
                        let one = builder.ins().f64const(1.0);
                        let zero = builder.ins().f64const(0.0);
                        builder.ins().select(cmp, one, zero)
                    }
                }
                BinOp::Lt => {
                    let cmp = builder.ins().fcmp(FloatCC::LessThan, l, r);
                    let one = builder.ins().f64const(1.0);
                    let zero = builder.ins().f64const(0.0);
                    builder.ins().select(cmp, one, zero)
                }
                BinOp::Le => {
                    let cmp = builder.ins().fcmp(FloatCC::LessThanOrEqual, l, r);
                    let one = builder.ins().f64const(1.0);
                    let zero = builder.ins().f64const(0.0);
                    builder.ins().select(cmp, one, zero)
                }
                BinOp::Gt => {
                    let cmp = builder.ins().fcmp(FloatCC::GreaterThan, l, r);
                    let one = builder.ins().f64const(1.0);
                    let zero = builder.ins().f64const(0.0);
                    builder.ins().select(cmp, one, zero)
                }
                BinOp::Ge => {
                    let cmp = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, l, r);
                    let one = builder.ins().f64const(1.0);
                    let zero = builder.ins().f64const(0.0);
                    builder.ins().select(cmp, one, zero)
                }
                
                BinOp::And => {
                    let mut sig_truthy = module.make_signature();
                    sig_truthy.params.push(AbiParam::new(types::F64));
                    sig_truthy.returns.push(AbiParam::new(types::I32));
                    let l_bool = if let Ok(id) = module.declare_function("js_is_truthy", cranelift_module::Linkage::Import, &sig_truthy) {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        let call = builder.ins().call(func_ref, &[l]);
                        let i32_result = builder.inst_results(call)[0];
                        let zero_i32 = builder.ins().iconst(types::I32, 0);
                        builder.ins().icmp(IntCC::NotEqual, i32_result, zero_i32)
                    } else {
                        let zero = builder.ins().f64const(0.0);
                        builder.ins().fcmp(FloatCC::NotEqual, l, zero)
                    };
                    
                    let short_circuit_block = builder.create_block();
                    let eval_right_block = builder.create_block();
                    let merge_block = builder.create_block();
                    builder.append_block_param(merge_block, types::F64);
                    
                    builder.ins().brif(l_bool, eval_right_block, &[], short_circuit_block, &[]);
                    
                    builder.switch_to_block(short_circuit_block);
                    builder.seal_block(short_circuit_block);
                    builder.ins().jump(merge_block, &[l]);
                    
                    builder.switch_to_block(eval_right_block);
                    builder.seal_block(eval_right_block);
                    builder.ins().jump(merge_block, &[r]);
                    
                    builder.switch_to_block(merge_block);
                    builder.seal_block(merge_block);
                    builder.block_params(merge_block)[0]
                }
                BinOp::Or => {
                    let mut sig_truthy = module.make_signature();
                    sig_truthy.params.push(AbiParam::new(types::F64));
                    sig_truthy.returns.push(AbiParam::new(types::I32));
                    let l_bool = if let Ok(id) = module.declare_function("js_is_truthy", cranelift_module::Linkage::Import, &sig_truthy) {
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        let call = builder.ins().call(func_ref, &[l]);
                        let i32_result = builder.inst_results(call)[0];
                        let zero_i32 = builder.ins().iconst(types::I32, 0);
                        builder.ins().icmp(IntCC::NotEqual, i32_result, zero_i32)
                    } else {
                        let zero = builder.ins().f64const(0.0);
                        builder.ins().fcmp(FloatCC::NotEqual, l, zero)
                    };
                    
                    let short_circuit_block = builder.create_block();
                    let eval_right_block = builder.create_block();
                    let merge_block = builder.create_block();
                    builder.append_block_param(merge_block, types::F64);
                    
                    builder.ins().brif(l_bool, short_circuit_block, &[], eval_right_block, &[]);
                    
                    builder.switch_to_block(short_circuit_block);
                    builder.seal_block(short_circuit_block);
                    builder.ins().jump(merge_block, &[l]);
                    
                    builder.switch_to_block(eval_right_block);
                    builder.seal_block(eval_right_block);
                    builder.ins().jump(merge_block, &[r]);
                    
                    builder.switch_to_block(merge_block);
                    builder.seal_block(merge_block);
                    builder.block_params(merge_block)[0]
                }
                
                BinOp::BitAnd => {
                    // Convert f64 to i64, bitwise AND, convert back
                    let l_int = builder.ins().fcvt_to_sint_sat(types::I64, l);
                    let r_int = builder.ins().fcvt_to_sint_sat(types::I64, r);
                    let result = builder.ins().band(l_int, r_int);
                    builder.ins().fcvt_from_sint(types::F64, result)
                }
                BinOp::BitOr => {
                    let l_int = builder.ins().fcvt_to_sint_sat(types::I64, l);
                    let r_int = builder.ins().fcvt_to_sint_sat(types::I64, r);
                    let result = builder.ins().bor(l_int, r_int);
                    builder.ins().fcvt_from_sint(types::F64, result)
                }
                BinOp::BitXor => {
                    let l_int = builder.ins().fcvt_to_sint_sat(types::I64, l);
                    let r_int = builder.ins().fcvt_to_sint_sat(types::I64, r);
                    let result = builder.ins().bxor(l_int, r_int);
                    builder.ins().fcvt_from_sint(types::F64, result)
                }
                BinOp::Shl => {
                    let l_int = builder.ins().fcvt_to_sint_sat(types::I64, l);
                    let r_int = builder.ins().fcvt_to_sint_sat(types::I64, r);
                    let result = builder.ins().ishl(l_int, r_int);
                    builder.ins().fcvt_from_sint(types::F64, result)
                }
                BinOp::Shr => {
                    let l_int = builder.ins().fcvt_to_sint_sat(types::I64, l);
                    let r_int = builder.ins().fcvt_to_sint_sat(types::I64, r);
                    let result = builder.ins().ushr(l_int, r_int);
                    builder.ins().fcvt_from_sint(types::F64, result)
                }
            }
        }
        
        HirExpr::Unary { op, operand } => {
            let val = compile_expr(operand, builder, variables, module, function_ids, function_param_counts, _string_data);
            match op {
                UnaryOp::Not => {
                    let zero = builder.ins().f64const(0.0);
                    let cmp = builder.ins().fcmp(FloatCC::Equal, val, zero);
                    builder.ins().fcvt_from_sint(types::F64, cmp)
                }
                UnaryOp::Neg => builder.ins().fneg(val),
                _ => val,
            }
        }
        
        HirExpr::Typeof(expr) => {
            let val = compile_expr(expr, builder, variables, module, function_ids, function_param_counts, _string_data);
            
            let mut sig = module.make_signature();
            sig.params.push(AbiParam::new(types::F64));
            sig.returns.push(AbiParam::new(types::F64));
            
            if let Ok(id) = module.declare_function("js_typeof", cranelift_module::Linkage::Import, &sig) {
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                let call = builder.ins().call(func_ref, &[val]);
                builder.inst_results(call)[0]
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }
        
        HirExpr::Call { callee, args } => {
            if let HirExpr::Identifier(name) = callee.as_ref() {
                if let Some(&func_id) = function_ids.get(name) {
                    let expected_params = function_param_counts.get(name).copied().unwrap_or(args.len());
                    let arg_values: Vec<Value> = args.iter()
                        .map(|arg| compile_expr(arg, builder, variables, module, function_ids, function_param_counts, _string_data))
                        .collect();
                    let undef_val = builder.ins().f64const(f64::from_bits(UNDEFINED));
                    let padded_args: Vec<Value> = if arg_values.len() < expected_params {
                        let mut v = arg_values;
                        while v.len() < expected_params {
                            v.push(undef_val);
                        }
                        v
                    } else {
                        arg_values
                    };
                    
                    let func_ref = module.declare_func_in_func(func_id, &mut builder.func);
                    let call = builder.ins().call(func_ref, &padded_args);
                    let results = builder.inst_results(call);
                    if results.len() > 0 {
                        results[0]
                    } else {
                        builder.ins().f64const(f64::from_bits(UNDEFINED))
                    }
                } else {
                    match name.as_str() {
                        "print" | "console.log" => {
                            for (i, arg) in args.iter().enumerate() {
                                let val = compile_expr(arg, builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig_print = module.make_signature();
                                sig_print.params.push(AbiParam::new(types::F64));
                                sig_print.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_print", cranelift_module::Linkage::Import, &sig_print) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    builder.ins().call(func_ref, &[val]);
                                }
                                
                                if i + 1 < args.len() {
                                    let mut sig_sep = module.make_signature();
                                    sig_sep.returns.push(AbiParam::new(types::F64));
                                    if let Ok(id) = module.declare_function("js_print_space", cranelift_module::Linkage::Import, &sig_sep) {
                                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                        builder.ins().call(func_ref, &[]);
                                    }
                                }
                            }
                            let mut sig_nl = module.make_signature();
                            sig_nl.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_print_newline", cranelift_module::Linkage::Import, &sig_nl) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                builder.ins().call(func_ref, &[]);
                            }
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                        "Math.sin" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_math_sin", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Math.cos" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_math_cos", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Math.sqrt" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_math_sqrt", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Math.pow" => {
                            if args.len() >= 2 {
                                let base = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                let exp = compile_expr(&args[1], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_math_pow", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[base, exp]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Math.abs" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_math_abs", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Math.floor" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_math_floor", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Math.ceil" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_math_ceil", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "document.createElement" => {
                            if !args.is_empty() {
                                let tag = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_dom_create_element", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[tag]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "element.appendChild" => {
                            if args.len() >= 2 {
                                let parent = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                let child = compile_expr(&args[1], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.params.push(AbiParam::new(types::F64));
                                sig.returns.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_dom_append_child", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    let call = builder.ins().call(func_ref, &[parent, child]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().f64const(0.0)
                                }
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "element.textContent" => {
                            if args.len() >= 2 {
                                let elem = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                let text = compile_expr(&args[1], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                sig.params.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_dom_set_text_content", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    builder.ins().call(func_ref, &[elem, text]);
                                }
                            }
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                        "browser.setHTML" => {
                            if !args.is_empty() {
                                let html = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                
                                let mut sig = module.make_signature();
                                sig.params.push(AbiParam::new(types::F64));
                                
                                if let Ok(id) = module.declare_function("js_browser_set_html", Linkage::Import, &sig) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    builder.ins().call(func_ref, &[html]);
                                }
                            }
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                        "browser.render" => {
                            let sig = module.make_signature();
                            
                            if let Ok(id) = module.declare_function("js_browser_render", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                builder.ins().call(func_ref, &[]);
                            }
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                        "parseInt" => {
                            if !args.is_empty() {
                                compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data)
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "parseFloat" => {
                            if !args.is_empty() {
                                compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data)
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "String" => {
                            if !args.is_empty() {
                                compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data)
                            } else {
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }
                        }
                        "Number" => {
                            if !args.is_empty() {
                                compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data)
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Boolean" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                                let zero = builder.ins().f64const(0.0);
                                let cmp = builder.ins().fcmp(FloatCC::NotEqual, val, zero);
                                builder.ins().fcvt_from_sint(types::F64, cmp)
                            } else {
                                builder.ins().f64const(f64::from_bits(FALSE))
                            }
                        }
                        _ => {
                            let arg_values: Vec<Value> = args.iter()
                                .map(|arg| compile_expr(arg, builder, variables, module, function_ids, function_param_counts, _string_data))
                                .collect();
                            
                            let mut sig = module.make_signature();
                            for _ in 0..arg_values.len() {
                                sig.params.push(AbiParam::new(types::F64));
                            }
                            sig.returns.push(AbiParam::new(types::F64));
                            
                            let c_name = if name.starts_with("js_") {
                                name.clone()
                            } else if name.contains(".") {
                                format!("js_{}", name.replace(".", "_"))
                            } else {
                                name.clone()
                            };
                            
                            if let Ok(id) = module.declare_function(&c_name, Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &arg_values);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }

                        }
                    }
                }
            } else if let HirExpr::Property { object, name } = callee.as_ref() {
                let full_name = match object.as_ref() {
                    HirExpr::Identifier(obj_name) => format!("{}.{}", obj_name, name),
                    HirExpr::Property { object: inner_obj, name: inner_name } => {
                        match inner_obj.as_ref() {
                            HirExpr::Identifier(obj_name) => format!("{}.{}.{}", obj_name, inner_name, name),
                            _ => format!("{}.{}", inner_name, name),
                        }
                    }
                    _ => name.clone(),
                };
                
                match full_name.as_str() {
                    "console.log" | "console.error" | "console.warn" => {
                        for (i, arg) in args.iter().enumerate() {
                            let val = compile_expr(arg, builder, variables, module, function_ids, function_param_counts, _string_data);
                            
                            let mut sig_print = module.make_signature();
                            sig_print.params.push(AbiParam::new(types::F64));
                            sig_print.returns.push(AbiParam::new(types::F64));
                            
                            if let Ok(id) = module.declare_function("js_print", cranelift_module::Linkage::Import, &sig_print) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                builder.ins().call(func_ref, &[val]);
                            }
                            
                            if i + 1 < args.len() {
                                let mut sig_sep = module.make_signature();
                                sig_sep.returns.push(AbiParam::new(types::F64));
                                if let Ok(id) = module.declare_function("js_print_space", cranelift_module::Linkage::Import, &sig_sep) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    builder.ins().call(func_ref, &[]);
                                }
                            }
                        }
                        let mut sig_nl = module.make_signature();
                        sig_nl.returns.push(AbiParam::new(types::F64));
                        if let Ok(id) = module.declare_function("js_print_newline", cranelift_module::Linkage::Import, &sig_nl) {
                            let func_ref = module.declare_func_in_func(id, &mut builder.func);
                            builder.ins().call(func_ref, &[]);
                        }
                        builder.ins().f64const(f64::from_bits(UNDEFINED))
                    }
                    "Math.sin" | "Math.cos" | "Math.sqrt" | "Math.abs" | "Math.floor" | "Math.ceil" => {
                        let c_name = format!("js_math_{}", name);
                        if !args.is_empty() {
                            let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function(&c_name, Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        } else {
                            builder.ins().f64const(0.0)
                        }
                    }
                    "Math.pow" => {
                        if args.len() >= 2 {
                            let base = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let exp = compile_expr(&args[1], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_math_pow", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[base, exp]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        } else {
                            builder.ins().f64const(0.0)
                        }
                    }
                    "Math.random" => {
                        let mut sig = module.make_signature();
                        sig.returns.push(AbiParam::new(types::F64));
                        if let Ok(id) = module.declare_function("js_math_random", Linkage::Import, &sig) {
                            let func_ref = module.declare_func_in_func(id, &mut builder.func);
                            let call = builder.ins().call(func_ref, &[]);
                            builder.inst_results(call)[0]
                        } else {
                            builder.ins().f64const(0.0)
                        }
                    }
                    "Date.now" => {
                        let mut sig = module.make_signature();
                        sig.returns.push(AbiParam::new(types::F64));
                        if let Ok(id) = module.declare_function("js_date_now", Linkage::Import, &sig) {
                            let func_ref = module.declare_func_in_func(id, &mut builder.func);
                            let call = builder.ins().call(func_ref, &[]);
                            builder.inst_results(call)[0]
                        } else {
                            builder.ins().f64const(0.0)
                        }
                    }
                    "JSON.stringify" => {
                        if !args.is_empty() {
                            let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_json_stringify", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }
                        } else {
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                    }
                    "JSON.parse" => {
                        if !args.is_empty() {
                            let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_json_parse", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }
                        } else {
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                    }
                    "Object.keys" => {
                        if !args.is_empty() {
                            let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_object_keys", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }
                        } else {
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                    }
                    "Object.values" => {
                        if !args.is_empty() {
                            let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_object_values", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }
                        } else {
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                    }
                    "Object.assign" => {
                        if args.len() >= 2 {
                            let target = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let source = compile_expr(&args[1], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_object_assign", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[target, source]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }
                        } else {
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                    }
                    "Array.isArray" => {
                        if !args.is_empty() {
                            let val = compile_expr(&args[0], builder, variables, module, function_ids, function_param_counts, _string_data);
                            let mut sig = module.make_signature();
                            sig.params.push(AbiParam::new(types::F64));
                            sig.returns.push(AbiParam::new(types::F64));
                            if let Ok(id) = module.declare_function("js_array_is_array", Linkage::Import, &sig) {
                                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                let call = builder.ins().call(func_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                builder.ins().f64const(f64::from_bits(FALSE))
                            }
                        } else {
                            builder.ins().f64const(f64::from_bits(FALSE))
                        }
                    }
                    _ => {
                        let obj_val = compile_expr(object, builder, variables, module, function_ids, function_param_counts, _string_data);
                        let arg_values: Vec<Value> = args.iter()
                            .map(|arg| compile_expr(arg, builder, variables, module, function_ids, function_param_counts, _string_data))
                            .collect();
                        let mut all_args = vec![obj_val];
                        all_args.extend_from_slice(&arg_values);
                        let mut sig = module.make_signature();
                        for _ in 0..all_args.len() {
                            sig.params.push(AbiParam::new(types::F64));
                        }
                        sig.returns.push(AbiParam::new(types::F64));
                        let c_name = format!("js_{}", name);
                        if let Ok(id) = module.declare_function(&c_name, Linkage::Import, &sig) {
                            let func_ref = module.declare_func_in_func(id, &mut builder.func);
                            let call = builder.ins().call(func_ref, &all_args);
                            builder.inst_results(call)[0]
                        } else {
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                    }
                }
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }
        
        HirExpr::Ternary { cond, then_expr, else_expr } => {
            let cond_val = compile_expr(cond, builder, variables, module, function_ids, function_param_counts, _string_data);
            let cond_bool = is_truthy(cond_val, builder, module);
            
            let then_block = builder.create_block();
            let else_block = builder.create_block();
            let merge_block = builder.create_block();
            builder.append_block_param(merge_block, types::F64);
            
            builder.ins().brif(cond_bool, then_block, &[], else_block, &[]);
            
            builder.switch_to_block(then_block);
            builder.seal_block(then_block);
            let then_val = compile_expr(then_expr, builder, variables, module, function_ids, function_param_counts, _string_data);
            builder.ins().jump(merge_block, &[then_val]);
            
            builder.switch_to_block(else_block);
            builder.seal_block(else_block);
            let else_val = compile_expr(else_expr, builder, variables, module, function_ids, function_param_counts, _string_data);
            builder.ins().jump(merge_block, &[else_val]);
            
            builder.switch_to_block(merge_block);
            builder.seal_block(merge_block);
            
            builder.block_params(merge_block)[0]
        }
        
        HirExpr::Array(elements) => {
            let len = elements.len();
            
            let mut sig_new = module.make_signature();
            sig_new.params.push(AbiParam::new(types::F64));
            sig_new.returns.push(AbiParam::new(types::F64));
            
            let arr_new_id = module.declare_function("js_array_new", cranelift_module::Linkage::Import, &sig_new);
            
            if let Ok(id) = arr_new_id {
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                let capacity = builder.ins().f64const(len as f64);
                let call = builder.ins().call(func_ref, &[capacity]);
                let arr_val = builder.inst_results(call)[0];
                
                let mut sig_push = module.make_signature();
                sig_push.params.push(AbiParam::new(types::F64));
                sig_push.params.push(AbiParam::new(types::F64));
                sig_push.returns.push(AbiParam::new(types::F64));
                
                let arr_push_id = module.declare_function("js_array_push", cranelift_module::Linkage::Import, &sig_push);
                
                let mut final_arr = arr_val;
                
                if let Ok(push_id) = arr_push_id {
                    let push_ref = module.declare_func_in_func(push_id, &mut builder.func);
                    
                    for elem in elements {
                        let elem_val = compile_expr(elem, builder, variables, module, function_ids, function_param_counts, _string_data);
                        let call = builder.ins().call(push_ref, &[final_arr, elem_val]);
                        final_arr = builder.inst_results(call)[0];
                    }
                }
                
                final_arr
            } else {
                let len_bits = len as u64;
                builder.ins().f64const(f64::from_bits(ARRAY_TAG | (len_bits & 0x0000_FFFF_FFFF_FFFF)))
            }
        }
        HirExpr::Index { object, index } => {
            let obj_val = compile_expr(object, builder, variables, module, function_ids, function_param_counts, _string_data);
            let idx_val = compile_expr(index, builder, variables, module, function_ids, function_param_counts, _string_data);
            
            let mut sig_get = module.make_signature();
            sig_get.params.push(AbiParam::new(types::F64));
            sig_get.params.push(AbiParam::new(types::F64));
            sig_get.returns.push(AbiParam::new(types::F64));
            
            let arr_get_id = module.declare_function("js_array_get", cranelift_module::Linkage::Import, &sig_get);
            
            if let Ok(id) = arr_get_id {
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                let call = builder.ins().call(func_ref, &[obj_val, idx_val]);
                builder.inst_results(call)[0]
            } else {
                idx_val
            }
        }
        HirExpr::Object(properties) => {
            let mut sig_new = module.make_signature();
            sig_new.returns.push(AbiParam::new(types::F64));
            
            let obj_new_id = module.declare_function("js_object_new", cranelift_module::Linkage::Import, &sig_new);
            
            if let Ok(id) = obj_new_id {
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                let call = builder.ins().call(func_ref, &[]);
                let obj_val = builder.inst_results(call)[0];
                
                let mut sig_set = module.make_signature();
                sig_set.params.push(AbiParam::new(types::F64));
                sig_set.params.push(AbiParam::new(types::F64));
                sig_set.params.push(AbiParam::new(types::F64));
                sig_set.returns.push(AbiParam::new(types::F64));
                
                if let Ok(set_id) = module.declare_function("js_object_set", cranelift_module::Linkage::Import, &sig_set) {
                    let set_ref = module.declare_func_in_func(set_id, &mut builder.func);
                    
                    for (key, value) in properties {
                        // NOTE: Property keys are stored as string hashes (not real JsString pointers).
                        let hash = fnv1a_64(key);
                        let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF)));
                        let val = compile_expr(value, builder, variables, module, function_ids, function_param_counts, _string_data);
                        builder.ins().call(set_ref, &[obj_val, key_val, val]);
                    }
                }
                
                obj_val
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }
        HirExpr::Property { object, name } => {
            let obj_val = compile_expr(object, builder, variables, module, function_ids, function_param_counts, _string_data);
            
            if name == "length" {
                let mut sig = module.make_signature();
                sig.params.push(AbiParam::new(types::F64));
                sig.returns.push(AbiParam::new(types::F64));
                if let Ok(id) = module.declare_function("js_property_length", cranelift_module::Linkage::Import, &sig) {
                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                    let call = builder.ins().call(func_ref, &[obj_val]);
                    builder.inst_results(call)[0]
                } else {
                    builder.ins().f64const(f64::from_bits(UNDEFINED))
                }
            } else {
                let mut sig_get = module.make_signature();
                sig_get.params.push(AbiParam::new(types::F64));
                sig_get.params.push(AbiParam::new(types::F64));
                sig_get.returns.push(AbiParam::new(types::F64));
                
                if let Ok(id) = module.declare_function("js_object_get", cranelift_module::Linkage::Import, &sig_get) {
                    let hash = fnv1a_64(name);
                    let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF)));
                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                    let call = builder.ins().call(func_ref, &[obj_val, key_val]);
                    builder.inst_results(call)[0]
                } else {
                    builder.ins().f64const(f64::from_bits(UNDEFINED))
                }
            }
        }
        
        HirExpr::ExprStmt(expr) => compile_expr(expr, builder, variables, module, function_ids, function_param_counts, _string_data),
        
        _ => builder.ins().f64const(f64::from_bits(UNDEFINED)),
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}

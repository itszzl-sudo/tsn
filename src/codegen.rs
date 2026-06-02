use anyhow::Result;
use cranelift::prelude::*;

use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::hir::{HirExpr, BinOp, UnaryOp};
use crate::builtins;

pub const UNDEFINED: u64 = 0x7FFF_8000_0000_0001;
pub const NULL: u64 = 0x7FFF_8000_0000_0002;
pub const TRUE: u64 = 0x7FFF_0000_0000_0001;
pub const FALSE: u64 = 0x7FFF_0000_0000_0000;
pub const STRING_TAG: u64 = 0x7FFC_0000_0000_0000;
pub const ARRAY_TAG: u64 = 0x7FFB_0000_0000_0000;
#[allow(dead_code)]
pub const OBJECT_TAG: u64 = 0x7FFA_0000_0000_0000;

#[allow(dead_code)]
pub const EVENT_CLICK: u64 = 1;
#[allow(dead_code)]
pub const EVENT_KEYDOWN: u64 = 2;
#[allow(dead_code)]
pub const EVENT_KEYUP: u64 = 3;
#[allow(dead_code)]
pub const EVENT_MOUSEMOVE: u64 = 4;
#[allow(dead_code)]
pub const EVENT_CHANGE: u64 = 5;
#[allow(dead_code)]
pub const EVENT_SUBMIT: u64 = 6;
#[allow(dead_code)]
pub const EVENT_FOCUS: u64 = 7;
#[allow(dead_code)]
pub const EVENT_BLUR: u64 = 8;
#[allow(dead_code)]
pub const EVENT_CHANGE_VALUE: u64 = 9;

#[allow(dead_code)]
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


pub struct CodeGen {
    builder_context: FunctionBuilderContext,
    function_ids: std::collections::HashMap<String, cranelift_module::FuncId>,
    string_pool: std::collections::HashSet<String>,
    #[allow(dead_code)]
    external_functions: std::collections::HashMap<String, String>,
    registry: Option<crate::extension::ExtensionRegistry>,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            builder_context: FunctionBuilderContext::new(),
            function_ids: std::collections::HashMap::new(),
            string_pool: std::collections::HashSet::new(),
            external_functions: std::collections::HashMap::new(),
            registry: None,
        }
    }

    pub fn with_registry(mut self, registry: crate::extension::ExtensionRegistry) -> Self {
        self.registry = Some(registry);
        self
    }


    #[allow(dead_code)]
    pub fn with_external_functions(mut self, functions: std::collections::HashMap<String, String>) -> Self {
        self.external_functions = functions;
        self
    }
    
    #[allow(dead_code)]
    pub fn set_external_functions(&mut self, functions: std::collections::HashMap<String, String>) {
        self.external_functions = functions;
    }
    
    pub fn compile(&mut self, exprs: &[HirExpr]) -> Result<Vec<u8>> {

        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa = cranelift_native::builder()
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .finish(settings::Flags::new(flag_builder))?;
        
        let builder = ObjectBuilder::new(
            isa,
            "ts_native",
            cranelift_module::default_libcall_names(),
        )?;
        let mut module: ObjectModule = ObjectModule::new(builder);
        
        // 第一遍：收集所有函数签名和字符串常量
        for expr in exprs {
            if let HirExpr::Function { name, params, body: _, span: _ } = expr {
                let mut sig = module.make_signature();
                for _ in params {
                    sig.params.push(AbiParam::new(types::F64));
                }
                sig.returns.push(AbiParam::new(types::F64));
                
                let func_id = module.declare_function(name, Linkage::Export, &sig)?;
                self.function_ids.insert(name.clone(), func_id);
            }
            self.collect_strings(expr);
        }
        
        // 创建字符串数据对象
        let mut string_data: std::collections::HashMap<String, (cranelift_module::DataId, Vec<u8>)> = std::collections::HashMap::new();
        for string in self.string_pool.iter() {
            let data_name = format!("str_{}", string_data.len());
            let data_id = module.declare_data(&data_name, Linkage::Local, false, false)?;
            
            let mut data_ctx = DataDescription::new();
            let mut bytes: Vec<u8> = string.bytes().collect();
            bytes.push(0);
            data_ctx.define(bytes.clone().into_boxed_slice());
            
            string_data.insert(string.clone(), (data_id, bytes));
            module.define_data(data_id, &data_ctx)?;
        }
        
        let registry = self.registry.clone();
        for expr in exprs {
            if let HirExpr::Function { name, params, body, span } = expr {
                println!("  [{}] function {}({:?})", span, name, params);
                self.compile_function(&mut module, name, params, body, &string_data, &registry)?;
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
        registry: &Option<crate::extension::ExtensionRegistry>,
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
        for stmt in body {
            if compile_stmt(stmt, &mut builder, &mut variables, module, &self.function_ids, string_data, None, None, registry) {
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
    _string_data: &std::collections::HashMap<String, (cranelift_module::DataId, Vec<u8>)>,

    loop_exit: Option<Block>,
    loop_continue: Option<Block>,
    registry: &Option<crate::extension::ExtensionRegistry>,
) -> bool {
    match stmt {
        HirExpr::Break => {
            if let Some(exit) = loop_exit {
                builder.ins().jump(exit, &[]);
            }
            true
        }
        HirExpr::Continue => {
            if let Some(cont) = loop_continue {
                builder.ins().jump(cont, &[]);
            }
            true
        }
        HirExpr::Return(val) => {
            if let Some(v) = val {
                let result = compile_expr(v, builder, variables, module, function_ids, _string_data, registry);
                builder.ins().return_(&[result]);
            } else {
                let undef = builder.ins().f64const(f64::from_bits(UNDEFINED));
                builder.ins().return_(&[undef]);
            }
            true
        }
        HirExpr::Var { name, init, is_mut: _ } => {
            let val = if let Some(i) = init {
                compile_expr(i, builder, variables, module, function_ids, _string_data, registry)
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            };
            
            let var = if let Some(&existing_var) = variables.get(name) {
                existing_var
            } else {
                let var_idx = variables.len();
                let var = Variable::new(var_idx);
                variables.insert(name.clone(), var);
                builder.declare_var(var, types::F64);
                var
            };
            builder.def_var(var, val);
            false
        }
        HirExpr::If { cond, then_body, else_body } => {
            let cond_val = compile_expr(cond, builder, variables, module, function_ids, _string_data, registry);
            let zero = builder.ins().f64const(0.0);
            let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
            
            let then_block = builder.create_block();
            let else_block = builder.create_block();
            let merge_block = builder.create_block();
            
            builder.ins().brif(cond_bool, then_block, &[], else_block, &[]);
            
            builder.switch_to_block(then_block);
            builder.seal_block(then_block);
            let mut then_returns = false;
            for s in then_body {
                if compile_stmt(s, builder, variables, module, function_ids, _string_data, loop_exit, loop_continue, registry) {
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
                    if compile_stmt(s, builder, variables, module, function_ids, _string_data, loop_exit, loop_continue, registry) {
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
            let cond_val = compile_expr(cond, builder, variables, module, function_ids, _string_data, registry);
            let zero = builder.ins().f64const(0.0);
            let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
            builder.ins().brif(cond_bool, body_block, &[], exit_block, &[]);
            
            builder.switch_to_block(body_block);
            let mut body_terminated = false;
            for s in body {
                if compile_stmt(s, builder, variables, module, function_ids, _string_data, Some(exit_block), Some(header_block), registry) {
                    body_terminated = true;
                    break;
                }
            }
            if !body_terminated {
                builder.ins().jump(header_block, &[]);
            }
            
            builder.seal_block(header_block);
            builder.seal_block(body_block);
            
            builder.switch_to_block(exit_block);
            builder.seal_block(exit_block);
            
            false
        }
        HirExpr::For { init, cond, update, body } => {
            if let Some(i) = init {
                compile_stmt(i, builder, variables, module, function_ids, _string_data, loop_exit, loop_continue, registry);
            }
            
            let header_block = builder.create_block();
            let body_block = builder.create_block();
            let update_block = builder.create_block();
            let exit_block = builder.create_block();
            
            builder.ins().jump(header_block, &[]);
            
            builder.switch_to_block(header_block);
            if let Some(c) = cond {
                let cond_val = compile_expr(c, builder, variables, module, function_ids, _string_data, registry);
                let zero = builder.ins().f64const(0.0);
                let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
                builder.ins().brif(cond_bool, body_block, &[], exit_block, &[]);
            } else {
                builder.ins().jump(body_block, &[]);
            }
            
            builder.switch_to_block(body_block);
            let mut body_terminated = false;
            for s in body {
                if compile_stmt(s, builder, variables, module, function_ids, _string_data, Some(exit_block), Some(update_block), registry) {
                    body_terminated = true;
                    break;
                }
            }
            if !body_terminated {
                builder.ins().jump(update_block, &[]);
            }
            
            builder.switch_to_block(update_block);
            if let Some(u) = update {
                compile_stmt(u, builder, variables, module, function_ids, _string_data, Some(exit_block), Some(update_block), registry);
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
                if compile_stmt(s, builder, variables, module, function_ids, _string_data, loop_exit, loop_continue, registry) {
                    has_return = true;
                }
            }
            has_return
        }
        HirExpr::Assign { target, value } => {
            let val = compile_expr(value, builder, variables, module, function_ids, _string_data, registry);
            match target.as_ref() {
                HirExpr::Identifier(name) => {
                    if let Some(&var) = variables.get(name) {
                        builder.def_var(var, val);
                    }
                }
                HirExpr::Index { object, index } => {
                    let obj_val = compile_expr(object, builder, variables, module, function_ids, _string_data, registry);
                    let idx_val = compile_expr(index, builder, variables, module, function_ids, _string_data, registry);
                    
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
                    let obj_val = compile_expr(object, builder, variables, module, function_ids, _string_data, registry);
                    
                    let mut sig_set = module.make_signature();
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.params.push(AbiParam::new(types::F64));
                    sig_set.returns.push(AbiParam::new(types::F64));
                    
                    if let Ok(id) = module.declare_function("js_object_set", cranelift_module::Linkage::Import, &sig_set) {
                        let hash = name.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
                        let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF)));
                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                        builder.ins().call(func_ref, &[obj_val, key_val, val]);
                    }
                }
                _ => {}
            }
            false
        }
        _ => {
            compile_expr(stmt, builder, variables, module, function_ids, _string_data, registry);
            false
        }
    }
}

fn compile_expr(
    expr: &HirExpr,
    builder: &mut FunctionBuilder,
    variables: &std::collections::HashMap<String, Variable>,
    module: &mut ObjectModule,
    function_ids: &std::collections::HashMap<String, cranelift_module::FuncId>,
    _string_data: &std::collections::HashMap<String, (cranelift_module::DataId, Vec<u8>)>,
    registry: &Option<crate::extension::ExtensionRegistry>,
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
            let l = compile_expr(left, builder, variables, module, function_ids, _string_data, registry);
            let r = compile_expr(right, builder, variables, module, function_ids, _string_data, registry);
            
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
                        builder.ins().fdiv(l, r)
                    }
                }
                
                BinOp::Eq => {
                    let cmp = builder.ins().fcmp(FloatCC::Equal, l, r);
                    let one = builder.ins().f64const(1.0);
                    let zero = builder.ins().f64const(0.0);
                    builder.ins().select(cmp, one, zero)
                }
                BinOp::Ne => {
                    let cmp = builder.ins().fcmp(FloatCC::NotEqual, l, r);
                    let one = builder.ins().f64const(1.0);
                    let zero = builder.ins().f64const(0.0);
                    builder.ins().select(cmp, one, zero)
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
                    let zero = builder.ins().f64const(0.0);
                    let l_bool = builder.ins().fcmp(FloatCC::NotEqual, l, zero);
                    let r_bool = builder.ins().fcmp(FloatCC::NotEqual, r, zero);
                    let result = builder.ins().band(l_bool, r_bool);
                    builder.ins().fcvt_from_sint(types::F64, result)
                }
                BinOp::Or => {
                    let zero = builder.ins().f64const(0.0);
                    let l_bool = builder.ins().fcmp(FloatCC::NotEqual, l, zero);
                    let r_bool = builder.ins().fcmp(FloatCC::NotEqual, r, zero);
                    let result = builder.ins().bor(l_bool, r_bool);
                    builder.ins().fcvt_from_sint(types::F64, result)
                }
                
                _ => builder.ins().f64const(0.0),
            }
        }
        
        HirExpr::Unary { op, operand } => {
            let val = compile_expr(operand, builder, variables, module, function_ids, _string_data, registry);
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
            let val = compile_expr(expr, builder, variables, module, function_ids, _string_data, registry);
            
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
                    let arg_values: Vec<Value> = args.iter()
                        .map(|arg| compile_expr(arg, builder, variables, module, function_ids, _string_data, registry))
                        .collect();
                    
                    let func_ref = module.declare_func_in_func(func_id, &mut builder.func);
                    let call = builder.ins().call(func_ref, &arg_values);
                    let results = builder.inst_results(call);
                    if results.len() > 0 {
                        results[0]
                    } else {
                        builder.ins().f64const(f64::from_bits(UNDEFINED))
                    }
                } else {
                    match name.as_str() {
                        "print" | "console.log" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, _string_data, registry);
                                let mut sig_print = module.make_signature();
                                sig_print.params.push(AbiParam::new(types::F64));
                                sig_print.returns.push(AbiParam::new(types::F64));
                                if let Ok(id) = module.declare_function("js_print", cranelift_module::Linkage::Import, &sig_print) {
                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                    builder.ins().call(func_ref, &[val]);
                                }
                            }
                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                        }
                        "parseInt" | "parseFloat" | "String" | "Number" => {
                            if !args.is_empty() {
                                compile_expr(&args[0], builder, variables, module, function_ids, _string_data, registry)
                            } else {
                                builder.ins().f64const(0.0)
                            }
                        }
                        "Boolean" => {
                            if !args.is_empty() {
                                let val = compile_expr(&args[0], builder, variables, module, function_ids, _string_data, registry);
                                let zero = builder.ins().f64const(0.0);
                                let cmp = builder.ins().fcmp(FloatCC::NotEqual, val, zero);
                                builder.ins().fcvt_from_sint(types::F64, cmp)
                            } else {
                                builder.ins().f64const(f64::from_bits(FALSE))
                            }
                        }
                        _ => {
                            if let Some(info) = builtins::lookup_builtin(name) {
                                let arg_values: Vec<Value> = args.iter()
                                    .take(info.param_count())
                                    .map(|arg| compile_expr(arg, builder, variables, module, function_ids, _string_data, registry))
                                    .collect();

                                if arg_values.len() < info.param_count() {
                                    builder.ins().f64const(0.0)
                                } else {
                                    let mut sig = module.make_signature();
                                    for &pt in info.param_types {
                                        sig.params.push(AbiParam::new(match pt {
                                            builtins::ArgType::F64 => types::F64,
                                            builtins::ArgType::I64 => types::I64,
                                            builtins::ArgType::I32 => types::I32,
                                        }));
                                    }
                                    match info.ret_type {
                                        builtins::RetType::F64 => sig.returns.push(AbiParam::new(types::F64)),
                                        builtins::RetType::I64 => sig.returns.push(AbiParam::new(types::I64)),
                                        builtins::RetType::I32 => sig.returns.push(AbiParam::new(types::I32)),
                                        builtins::RetType::Void => {}
                                    }
                                    if let Ok(id) = module.declare_function(info.c_name, Linkage::Import, &sig) {
                                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                        let converted_args: Vec<Value> = arg_values.iter().zip(info.param_types.iter())
                                            .map(|(&val, &pt)| match pt {
                                                builtins::ArgType::F64 => val,
                                                builtins::ArgType::I64 => builder.ins().fcvt_to_sint(types::I64, val),
                                                builtins::ArgType::I32 => builder.ins().fcvt_to_sint(types::I32, val),
                                            })
                                            .collect();
                                        let call = builder.ins().call(func_ref, &converted_args);
                                        if info.has_return() {
                                            let raw = builder.inst_results(call)[0];
                                            match info.ret_type {
                                                builtins::RetType::I64 => builder.ins().fcvt_from_sint(types::F64, raw),
                                                builtins::RetType::I32 => builder.ins().fcvt_from_sint(types::F64, raw),
                                                _ => raw,
                                            }
                                        } else {
                                            builder.ins().f64const(f64::from_bits(UNDEFINED))
                                        }
                                    } else {
                                        builder.ins().f64const(0.0)
                                    }
                                }
                            } else if let Some(ref reg) = registry {
                                if let Some(func_info) = reg.get_function_info(name) {
                                    let c_name = &func_info.impl_name;
                                    let arg_values: Vec<Value> = args.iter()
                                        .map(|arg| compile_expr(arg, builder, variables, module, function_ids, _string_data, registry))
                                        .collect();
                                    let mut sig = module.make_signature();
                                    for at in &func_info.args {
                                        sig.params.push(AbiParam::new(match at {
                                            crate::extension::ArgType::Number | crate::extension::ArgType::Any => types::F64,
                                            crate::extension::ArgType::Boolean => types::I32,
                                            crate::extension::ArgType::String => types::I64,
                                            _ => types::F64,
                                        }));
                                    }
                                    match func_info.ret {
                                        crate::extension::RetType::Number | crate::extension::RetType::Any => sig.returns.push(AbiParam::new(types::F64)),
                                        crate::extension::RetType::Boolean => sig.returns.push(AbiParam::new(types::I32)),
                                        crate::extension::RetType::String => sig.returns.push(AbiParam::new(types::I64)),
                                        crate::extension::RetType::Void => {}
                                        _ => sig.returns.push(AbiParam::new(types::F64)),
                                    }
                                    if let Ok(id) = module.declare_function(c_name, Linkage::Import, &sig) {
                                        let func_ref = module.declare_func_in_func(id, &mut builder.func);
                                        let converted_args: Vec<Value> = arg_values.iter().zip(func_info.args.iter())
                                            .map(|(&val, at)| match at {
                                                crate::extension::ArgType::Number | crate::extension::ArgType::Any => val,
                                                crate::extension::ArgType::Boolean => builder.ins().fcvt_to_sint(types::I32, val),
                                                crate::extension::ArgType::String => {
                                                    let mut unbox_sig = module.make_signature();
                                                    unbox_sig.params.push(AbiParam::new(types::F64));
                                                    unbox_sig.returns.push(AbiParam::new(types::I64));
                                                    if let Ok(unbox_id) = module.declare_function("js_unbox_string", Linkage::Import, &unbox_sig) {
                                                        let unbox_ref = module.declare_func_in_func(unbox_id, &mut builder.func);
                                                        let unbox_call = builder.ins().call(unbox_ref, &[val]);
                                                        builder.inst_results(unbox_call)[0]
                                                    } else {
                                                        builder.ins().fcvt_to_sint(types::I64, val)
                                                    }
                                                }
                                                _ => val,
                                            })
                                            .collect();
                                        let call = builder.ins().call(func_ref, &converted_args);
                                        let raw = builder.inst_results(call)[0];
                                        match func_info.ret {
                                            crate::extension::RetType::Boolean => builder.ins().fcvt_from_sint(types::F64, raw),
                                            crate::extension::RetType::String => {
                                                let mut box_sig = module.make_signature();
                                                box_sig.params.push(AbiParam::new(types::I64));
                                                box_sig.returns.push(AbiParam::new(types::F64));
                                                if let Ok(box_id) = module.declare_function("js_box_string", Linkage::Import, &box_sig) {
                                                    let box_ref = module.declare_func_in_func(box_id, &mut builder.func);
                                                    let box_call = builder.ins().call(box_ref, &[raw]);
                                                    builder.inst_results(box_call)[0]
                                                } else {
                                                    builder.ins().fcvt_from_sint(types::F64, raw)
                                                }
                                            }
                                            _ => raw,
                                        }
                                    } else {
                                        builder.ins().f64const(f64::from_bits(UNDEFINED))
                                    }
                                } else {
                                    builder.ins().f64const(f64::from_bits(UNDEFINED))
                                }
                            } else {
                                eprintln!("[codegen] Warning: unresolved call '{}' - not in builtins or registry", name);
                                builder.ins().f64const(f64::from_bits(UNDEFINED))
                            }
                        }
                    }
                }
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }
        
        HirExpr::Ternary { cond, then_expr, else_expr } => {
            let cond_val = compile_expr(cond, builder, variables, module, function_ids, _string_data, registry);
            let zero = builder.ins().f64const(0.0);
            let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
            
            let then_block = builder.create_block();
            let else_block = builder.create_block();
            let merge_block = builder.create_block();
            builder.append_block_param(merge_block, types::F64);
            
            builder.ins().brif(cond_bool, then_block, &[], else_block, &[]);
            
            builder.switch_to_block(then_block);
            builder.seal_block(then_block);
            let then_val = compile_expr(then_expr, builder, variables, module, function_ids, _string_data, registry);
            builder.ins().jump(merge_block, &[then_val]);
            
            builder.switch_to_block(else_block);
            builder.seal_block(else_block);
            let else_val = compile_expr(else_expr, builder, variables, module, function_ids, _string_data, registry);
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
                        let elem_val = compile_expr(elem, builder, variables, module, function_ids, _string_data, registry);
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
            let obj_val = compile_expr(object, builder, variables, module, function_ids, _string_data, registry);
            let idx_val = compile_expr(index, builder, variables, module, function_ids, _string_data, registry);
            
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
                        let hash = key.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
                        let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF)));
                        let val = compile_expr(value, builder, variables, module, function_ids, _string_data, registry);
                        builder.ins().call(set_ref, &[obj_val, key_val, val]);
                    }
                }
                
                obj_val
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }
        HirExpr::Property { object, name } => {
            let obj_val = compile_expr(object, builder, variables, module, function_ids, _string_data, registry);
            
            let mut sig_get = module.make_signature();
            sig_get.params.push(AbiParam::new(types::F64));
            sig_get.params.push(AbiParam::new(types::F64));
            sig_get.returns.push(AbiParam::new(types::F64));
            
            if let Ok(id) = module.declare_function("js_object_get", cranelift_module::Linkage::Import, &sig_get) {
                let hash = name.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
                let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF)));
                let func_ref = module.declare_func_in_func(id, &mut builder.func);
                let call = builder.ins().call(func_ref, &[obj_val, key_val]);
                builder.inst_results(call)[0]
            } else {
                builder.ins().f64const(f64::from_bits(UNDEFINED))
            }
        }
        
        _ => builder.ins().f64const(f64::from_bits(UNDEFINED)),
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::SourceSpan;

    fn compile_exprs(exprs: &[HirExpr]) -> Result<Vec<u8>> {
        let mut codegen = CodeGen::new();
        codegen.compile(exprs)
    }

    #[test]
    fn test_codegen_simple_function() {
        let hir = vec![
            HirExpr::Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![HirExpr::Return(Some(Box::new(HirExpr::Number(42.0))))],
                span: SourceSpan::unknown(),
            },
        ];
        let result = compile_exprs(&hir);
        assert!(result.is_ok());
        assert!(result.unwrap().len() > 0);
    }

    #[test]
    fn test_codegen_add_function() {
        let hir = vec![
            HirExpr::Function {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: vec![HirExpr::Return(Some(Box::new(
                    HirExpr::Binary {
                        op: BinOp::Add,
                        left: Box::new(HirExpr::Identifier("a".to_string())),
                        right: Box::new(HirExpr::Identifier("b".to_string())),
                    }
                )))],
                span: SourceSpan::unknown(),
            },
        ];
        let result = compile_exprs(&hir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_codegen_while_loop() {
        let hir = vec![
            HirExpr::Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    HirExpr::Var { name: "i".to_string(), init: Some(Box::new(HirExpr::Number(10.0))), is_mut: true },
                    HirExpr::While {
                        cond: Box::new(HirExpr::Binary {
                            op: BinOp::Gt,
                            left: Box::new(HirExpr::Identifier("i".to_string())),
                            right: Box::new(HirExpr::Number(0.0)),
                        }),
                        body: vec![HirExpr::Assign {
                            target: Box::new(HirExpr::Identifier("i".to_string())),
                            value: Box::new(HirExpr::Binary {
                                op: BinOp::Sub,
                                left: Box::new(HirExpr::Identifier("i".to_string())),
                                right: Box::new(HirExpr::Number(1.0)),
                            }),
                        }],
                    },
                    HirExpr::Return(Some(Box::new(HirExpr::Identifier("i".to_string())))),
                ],
                span: SourceSpan::unknown(),
            },
        ];
        let result = compile_exprs(&hir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_codegen_break_continue() {
        let hir = vec![
            HirExpr::Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    HirExpr::While {
                        cond: Box::new(HirExpr::Boolean(true)),
                        body: vec![
                            HirExpr::Break,
                            HirExpr::Continue,
                        ],
                    },
                    HirExpr::Return(Some(Box::new(HirExpr::Number(0.0)))),
                ],
                span: SourceSpan::unknown(),
            },
        ];
        let result = compile_exprs(&hir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_codegen_string_literal() {
        let hir = vec![
            HirExpr::Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    HirExpr::Var { name: "s".to_string(), init: Some(Box::new(HirExpr::String("hello".to_string()))), is_mut: false },
                    HirExpr::Return(Some(Box::new(HirExpr::Identifier("s".to_string())))),
                ],
                span: SourceSpan::unknown(),
            },
        ];
        let result = compile_exprs(&hir);
        assert!(result.is_ok());
    }
}

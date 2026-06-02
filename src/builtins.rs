#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ArgType {
    F64,
    I64,
    I32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum RetType {
    F64,
    I64,
    I32,
    Void,
}

pub struct BuiltinInfo {
    pub c_name: &'static str,
    pub param_types: &'static [ArgType],
    pub ret_type: RetType,
}

impl BuiltinInfo {
    pub fn param_count(&self) -> usize {
        self.param_types.len()
    }

    pub fn has_return(&self) -> bool {
        self.ret_type != RetType::Void
    }
}

pub fn lookup_builtin(name: &str) -> Option<BuiltinInfo> {
    use ArgType as A;
    use RetType as R;
    match name {
        "Math.sin" => Some(BuiltinInfo { c_name: "js_math_sin", param_types: &[A::F64], ret_type: R::F64 }),
        "Math.cos" => Some(BuiltinInfo { c_name: "js_math_cos", param_types: &[A::F64], ret_type: R::F64 }),
        "Math.sqrt" => Some(BuiltinInfo { c_name: "js_math_sqrt", param_types: &[A::F64], ret_type: R::F64 }),
        "Math.abs" => Some(BuiltinInfo { c_name: "js_math_abs", param_types: &[A::F64], ret_type: R::F64 }),
        "Math.floor" => Some(BuiltinInfo { c_name: "js_math_floor", param_types: &[A::F64], ret_type: R::F64 }),
        "Math.ceil" => Some(BuiltinInfo { c_name: "js_math_ceil", param_types: &[A::F64], ret_type: R::F64 }),
        "Math.pow" => Some(BuiltinInfo { c_name: "js_math_pow", param_types: &[A::F64, A::F64], ret_type: R::F64 }),

        _ => None,
    }
}

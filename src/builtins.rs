pub struct BuiltinInfo {
    pub c_name: &'static str,
    pub param_count: usize,
    pub has_return: bool,
}

pub fn lookup_builtin(name: &str) -> Option<BuiltinInfo> {
    match name {
        "Math.sin" => Some(BuiltinInfo { c_name: "js_math_sin", param_count: 1, has_return: true }),
        "Math.cos" => Some(BuiltinInfo { c_name: "js_math_cos", param_count: 1, has_return: true }),
        "Math.sqrt" => Some(BuiltinInfo { c_name: "js_math_sqrt", param_count: 1, has_return: true }),
        "Math.abs" => Some(BuiltinInfo { c_name: "js_math_abs", param_count: 1, has_return: true }),
        "Math.floor" => Some(BuiltinInfo { c_name: "js_math_floor", param_count: 1, has_return: true }),
        "Math.ceil" => Some(BuiltinInfo { c_name: "js_math_ceil", param_count: 1, has_return: true }),
        "Math.pow" => Some(BuiltinInfo { c_name: "js_math_pow", param_count: 2, has_return: true }),
        "document.createElement" => Some(BuiltinInfo { c_name: "js_dom_create_element", param_count: 1, has_return: true }),
        "element.appendChild" => Some(BuiltinInfo { c_name: "js_dom_append_child", param_count: 2, has_return: true }),
        "element.textContent" => Some(BuiltinInfo { c_name: "js_dom_set_text_content", param_count: 2, has_return: false }),
        "browser.setHTML" => Some(BuiltinInfo { c_name: "js_browser_set_html", param_count: 1, has_return: false }),
        "browser.render" => Some(BuiltinInfo { c_name: "js_browser_render", param_count: 0, has_return: false }),
        _ => None,
    }
}

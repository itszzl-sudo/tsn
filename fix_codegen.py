# Fix all 11 if-let patterns in codegen.rs

filepath = r"E:\Administrator\Documents\codebuddy-projects\tsn_new\src\codegen.rs"

with open(filepath, "r", encoding="utf-8") as f:
    content = f.read()

replacements = []

# 1. js_array_set - no else branch
replacements.append((
    "                if let Ok(id) = module.declare_function(\"js_array_set\", cranelift_module::Linkage::Import, &sig_set) {\n"
    "                    let func_ref = module.declare_func_in_func(id, &mut builder.func);\n"
    "                    builder.ins().call(func_ref, &[obj_val, idx_val, val]);\n"
    "                }",

    "                let id = module.declare_function(\"js_array_set\", cranelift_module::Linkage::Import, &sig_set)\n"
    "                    .expect(\"failed to declare import: js_array_set\");\n"
    "                let func_ref = module.declare_func_in_func(id, &mut builder.func);\n"
    "                builder.ins().call(func_ref, &[obj_val, idx_val, val]);",

    "js_array_set"
))

# 2. js_object_set - no else branch
replacements.append((

    "                if let Ok(id) = module.declare_function(\"js_object_set\", cranelift_module::Linkage::Import, &sig_set) {\n"
    "                    let hash = name.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));\n"
    "                    let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF_FFFF)));\n"
    "                    let func_ref = module.declare_func_in_func(id, &mut builder.func);\n"
    "                    builder.ins().call(func_ref, &[obj_val, key_val, val]);\n"
    "                }",

    "                let id = module.declare_function(\"js_object_set\", cranelift_module::Linkage::Import, &sig_set)\n"
    "                    .expect(\"failed to declare import: js_object_set\");\n"
    "                let hash = name.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));\n"
    "                let key_val = builder.ins().f64const(f64:from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF_FFFF)));\n"
    "                let func_ref = module.declare_func_in_func(id, &mut builder.func);\n"
    "                 builder.ins().call(func_ref, &[obj_val, key_val, val]);",

    "js_object_set"
))

# 3. js_string_from_static - has else returning UNDEFIND
bf = \"             if let Ok(id) = module.declare_function(\"js_string_from_static\", cranelift_module::Linkage::Import, &sig) {\n\"
bf += \"                 let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                 let call = builder.ins().call(func_ref, &[addr]);\n\"
bf += \"                 builder.inst_results(call)[0]\n\"
bf += \"             } else {\n\"
bf += \"                 builder.ins().f64const(f64::from_bits(UNDEFIND))\n\"
bf += \"             }\n\"

ns = \"              let id = module.declare_function(\"js_string_from_static\", cranelift_module::Linkage::Import, &sig)\n\"
ns += \"                  .expect(\"failed to declare import: js_string_from_static\");\n\"
ns += \"              let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"              let call = builder.ins().call(func_ref, &[addr]);\n\"
ns += \"              builder.inst_result(call)[0]\"\n"

replacements.append((bf, ns, "js_string_from_static"))

# 4. js_add - has else returning fadd
bf = \"                    if let Ok(id) = module.declare_function(\"js_add\", cranelift_module::Linkage::Import, &sig) {\n\"
bf += \"                        let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                        let call = builder.ins().call(func_ref, &[l, r]);\n\"
bf += \"                        builder.inst_results(call)[0]\n\"
bf += \"                    } else {\n\"
bf += \"                        builder.ins().fadd(l, r)\n\"
bf += \"                    }\n\"

ns = \"                    let id = module.declare_function(\"js_add\", cranelift_module::Linkage::Import, &sig)\n\"
ns += \"                        .expect(\"failed to declare import: js_add\");\n\"
ns += \"                    let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"                    let call = builder.ins().call(func_ref, &[l, r]);\n\"
ns += \"                    builder.inst_result(call)[0]\"\n"

replacements.append((bf, ns, "js_add"))

# 5. js_mod - has else returning fdiv
bf = \"                    if let Ok(id) = module.declare_function(\"js_mod\", cranelift_module::Linkage::Import, &sig) {\n\"
bf += \"                        let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                        let call = builder.ins().call(func_ref, &[l, r]);\n\"
bf += \"                        builder.inst_results(call)[0]\n\"
bf += \"                    } else {\n\"
bf += \"                        builder.ins().fdiv(l, r)\n\"
bf += \"                    }\n\"

ns = \"                    let id = module.declare_function(\"js_mod\", cranelift_module::Linkage::Import, &sig)\n\"
ns += \"                        .expect(\"failed to declare import: js_mod\");\n\"
ns += \"                    let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"                    let call = builder.ins().call(func_ref, &[l, r]);\n\"
ns += \"                    builder.inst_result(call)[0]\"\n"

replacements.append((bf, ns, "js_mod"))

# 6. js_typeof - has else returning UNDEFIND
bf = \"            if let Ok(id) = module.declare_function(\"js_typeof\", cranelift_module::Linkage::Import, &sig) {\n\"
bf += \"                let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                let call = builder.ins().call(func_ref, &[val]);\n\"
bf += \"                builder.inst_result(call)[0]\n\"
bf += \"            } else {\n\"
bf += \"                builder.ins().f64const(f64::from_bits(UNDEFIND))\n\"
bf += \"            }\n\"

ns = \"            let id = module.declare_function(\"js_typeof\", cranelift_module::Linkage::Import, &sig)\n\"
ns += \"                 .expect(\"failed to declare import: js_typeof\");\n\"
ns += \"            let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"            let call = builder.ins().call(func_ref, &[val]);\n\"
ns += \"            builder.inst_result(call)[0]\n"

replacements.append((bf, ns, "js_typeof"))

# 7. js_print - no else branch
bf = \"                              if let Ok(id) = module.declare_function(\"js_print\", cranelift_module::Linkage::Import, &sig_print) {\n\"
bf += \"                                  let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                                  builder.ins().call(func_ref, &[val]);\n\"
bf += \"                              }\n\"

ns = \"                              let id = module.declare_function(\"js_print\", cranelift_module::Linkage::Import, &sig_print)\n\"
ns += \"                                  .expect(\"failed to declare import: js_print\");\n\"
ns += \"                              let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"                              builder.ins().call(func_ref, &[val]);\n\"

replacements.append((bf, ns, \"js_print\"))

# 8. buhtins info.c_name - has else returning 0.0
bf = \"                                    if let Ok(id) = module.declare_function(info.c_name, Linkage::Import, &sig) {\n\"
bf += \"                                        let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                                        let call = builder.ins().call(func_ref, &arg_values);\n\"
bf += \"                                        if info.has_return {\n\"
bf += \"                                           builder.inst_result(call)[0]\n\"
bf += \"                                        } else {\n\"
bf += \"                                            builder.ins().f64const(f64::from_bits(UNDEFIND))\n\"
bf += \"                                        }\n\"
bf += \"                                    } else {\n\"
bf += \"                                        builder.ins().f64const(0.0)\n\"
bf += \"                                    }\n\"

ns = \"                                    let id = module.declare_function(info.c_name, Linkage::Import, &sig)\n\"
ns += \"                                         .expect(&format!(\"failed to declare import: {}\", info.c_name));\n\"
ns += \"                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"                                    let call = builder.ins().call(func_ref, &arg_values);\n\"
ns += \"                                    if info.has_return {\n\"
ns += \"                                        builder.inst_result(call)[0]\n\"
ns += \"                                    } else {\n\"
ns += \"                                        builder.ins().f64const(f64::from_bits(UNDEFIND))\n\"
ns += \"                                    }\n\"

replacements.append((bf, ns, \"builtins info.c_name\"))

# 9. registry c_name - has else returning UNDEFIND
bf = \"                                     if let Ok(id) = module.declare_function(c_name, Linkage::Import, &sig) {\n\"
bf += \"                                         let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                                         let call = builder.ins().call(func_ref, &arg_values);\n\"
bf += \"                                         builder.inst_result(call)[0]\n\"
bf += \"                                     } else {\n\"
bf += \"                                         builder.ins().f64const(f64:from_bits(UNDEFIND))\n\"
bf += \"                                     }\n\"

ns = \"                                     let id = module.declare_function(c_name, Linkage::Import, &sig)\n\"
ns += \"                                          .expect(&format!(\"failed to declare import: {}\", c_name));\n\"
ns += \"                                     let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"                                     let call = builder.ins().call(func_ref, &arg_values);\n\"
ns += \"                                     builder.inst_result(call)[0]\n\"

replacements.append((bf, ns, \"registry c_name\"))

# 10. auto-named c_name - has else returning UNDEFIND
bf = \"                                if let Ok(id) = module.declare_function(&c_name, Linkage::Import, &sig) {\n\"
bf += \"                                    let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                                   let call = builder.ins().call(func_ref, &arg_values);\n\"
bf += \"                                   builder.inst_result(call)[0]\n\"
bf += \"                               } else {\n\"
bf += \"                                   builder.ins().f64const(f64:from_bits(UNDEFIND))\n\"
bf += \"                               }\n\"

ns = \"                               let id = module.declare_function(&c_name, Linkage::Import, &sig)\n\"
ns += \"                                    .expect(&format!(\"failed to declare import: {}\", c_name));\n\"
ns += \"                                   let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"                                   let call = builder.ins().call(func_ref, &arg_values);\n\"
ns += \"                                   builder.inst_result(call)[0]\n\"

replacements.append((bf, ns, \"auto-named c_name\"))

# 11. js_object_get - has else returning Undefined
bf = \"            if let Ok(id) = module.declare_function(\"js_object_get\", cranelift_module::Linkage::Import, &sig_get) {\n\"
bf += \"                let hash = name.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));\n\"
bf += \"                let key_val = builder.ins().f64const(f64::from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF_FFFF)));\n\"
bf += \"                let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
bf += \"                let call = builder.ins().call(func_ref, &[obj_val, key_val]);\n\"
bf += \"                builder.inst_result(call)[0]\n\"
bf += \"            } else {\n\"
bf += \"                builder.ins().f64const(f64::from_bits(UNDEFIND))\n\"
bf += \"            }\n\"

ns = \"            let id = module.declare_function(\"js_object_get\", cranelift_module::Linkage::Import, &sig_get)\n\"
ns += \"                 .expect(\"failed to declare import: js_object_get\");\n\"
ns += \"            let hash = name.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));\n\"
ns += \"            let key_val = builder.ins().f64const(f64:from_bits(STRING_TAG | (hash & 0x0000_FFFF_FFFF_FFFF_FFFF)));\n\"
ns += \"            let func_ref = module.declare_func_in_func(id, &mut builder.func);\n\"
ns += \"           let call = builder.ins().call(func_ref, &[obj_val, key_val]);\n\"
ns += \"            builder.inst_result(call)[0]\n\"

replacements.append((bf, ns, \"js_object_get\"))

# Perform all replacements
for old, new, name in replacements:
    if old not in content:
        print(f\"ERROR: {name} pattern not found!\")
        # Try to find partial match
        first_line = old.split(chr(10))[0]
        if first_line in content:
            idx = content.index(first_line)
            print(f\" First line found at pos {idx}\")
            print(f"  Context: {repr(content[idx:idx+300])}\")
        else:
            print(f\" First line not found: {repr(first_line)}\")
    else:
        content = content.replace(old, new, 1)
        print(f\"OK: {name} replaced\")

remaining = content.count(\"if let Ok(id) = module.declare_function\")
print(f'\RRemaining if let Ok(id) = module.declare_function patterns: {remaining}')

with open(filepath, "w", encoding="utf-8") as f:
    f.write(content)

print('Done!')

# tsn

TypeScript to native executable compiler.

**Nightly Release** - Latest features, experimental builds.

**For stable production-ready releases, see [ts-native](https://github.com/itszzl-sudo/ts-native).**

## Quick Start

```bash
# Install nightly build
cargo install tsn

# Compile TypeScript to native executable
tsn main.ts
./a.exe
```

## Project Management (Recommended)

```bash
# Install project manager
cargo install cargo-tsn

# Create a new project
cargo tsn new my-project
cd my-project

# Add dependencies
cargo tsn add regex

# Interactively add FFI functions
cargo tsn func

# Compile
tsn main.ts
./a.exe
```

## What is tsn

A compiler that converts TypeScript to native executable (10-14KB), no Node.js required.

## Key Features

- **Tiny executables**: 10-14KB native binaries
- **No runtime**: Zero dependencies, no Node.js
- **FFI integration**: Call Rust/C functions directly
- **NaN-boxing**: Efficient value representation
- **Latest features**: Modular C runtime, tsnp plugin spec, rapid iteration

## Architecture

```
TypeScript source
      ↓
   Lexer
      ↓
   Parser
      ↓
    HIR (High-level IR)
      ↓
   Codegen (Cranelift)
      ↓
   Object file (.o)
      ↓
   Linker (.exe)
      ↓
Native executable
```

## Value Representation

All JavaScript values fit in 64 bits using NaN-boxing:

```
STRING_TAG  = 0x7FFC_0000_0000_0000
ARRAY_TAG   = 0x7FFB_0000_0000_0000
OBJECT_TAG  = 0x7FFA_0000_0000_0000
UNDEFINED   = 0x7FFF_8000_0000_0001
NULL        = 0x7FFF_8000_0000_0002
TRUE        = 0x7FFF_0000_0000_0001
FALSE       = 0x7FFF_0000_0000_0000
```

## Extension System

tsn scans `./tsnp/` directory for plugin configurations:

```toml
# tsnp/my-plugin/ts-native.toml
[package]
name = "my-plugin"
version = "0.1.0"
description = "My plugin for tsn"

[ffi]
c_module = "runtime/runtime_plugin.c"

[functions]
"add" = { args = ["number", "number"], ret = "number", impl_name = "js_add" }

[link]
libs = ["mosquitto"]
flags = []
```

## Modular C Runtime

tsn features a modular C runtime architecture:

```
runtime/
├── runtime.h          # Public headers (NaN-boxing, callbacks)
├── runtime_core.c     # Core runtime (callback dispatcher)
├── runtime_dom.c      # DOM extension (js_dom_*)
└── runtime_mqtt.c     # MQTT extension (js_mqtt_*)
```

Link only the modules you need:

```bash
# Minimal runtime
gcc ui.o runtime_core.c -o app.exe

# DOM + MQTT
gcc ui.o runtime_core.c runtime_dom.c runtime_mqtt.c -o app.exe -lmosquitto
```

## Supported TypeScript

### Data Types
- ✅ Numbers (integers, floats)
- ✅ Strings (dynamic allocation, concatenation)
- ✅ Arrays (dynamic allocation, nested)
- ✅ Objects (dynamic allocation)
- ✅ Boolean, null, undefined

### Operators
- ✅ Arithmetic: `+ - * / %`
- ✅ Comparison: `== != < > <= >=`
- ✅ Logical: `&& || !`
- ✅ Ternary: `cond ? then : else`
- ✅ `typeof` operator

### Control Flow
- ✅ `if` statement
- ✅ `if-else` statement
- ✅ `while` loop
- ✅ `for` loop
- ✅ `return` statement

### Functions
- ✅ Function definition
- ✅ Function call
- ✅ Multiple parameters
- ✅ Return values
- ✅ Recursion

### Data Structures
- ✅ Array literals `[1, 2, 3]`
- ✅ Array indexing `arr[i]`
- ✅ Array assignment `arr[i] = value`
- ✅ Object literals `{x: 10}`
- ✅ Property access `obj.x`
- ✅ Property assignment `obj.x = value`

## Not Implemented

- `break`, `continue`
- `switch` statement
- `class` syntax
- `async/await`
- Modules
- Generics

## Project Structure

```
tsn/
├── src/
│   ├── main.rs       # CLI entry point
│   ├── ts_parser.rs  # TypeScript lexer & parser
│   ├── codegen.rs    # Cranelift code generation
│   ├── linker.rs     # Native linker
│   ├── extension.rs  # Plugin loading
│   ├── builtins.rs   # Built-in functions (Math.*)
│   └── hir.rs        # High-level IR
├── runtime/
│   ├── runtime.h     # Public headers
│   ├── runtime_core.c
│   ├── runtime_dom.c
│   └── runtime_mqtt.c
├── tsnp/
│   ├── dom-iot/
│   │   └── ts-native.toml
│   └── mqtt/
│       └── ts-native.toml
└── Cargo.toml
```

## Release Schedule

- **Stable (ts-native)**: Weekly updates, production-ready
- **Nightly (tsn)**: Random updates, latest features

Choose **tsn** for experimenting with new features, **ts-native** for production.

## Links

- [ts-native (Stable)](https://github.com/itszzl-sudo/ts-native) - Production-ready
- [cargo-tsn](https://github.com/itszzl-sudo/cargo-tsn) - Project manager
- [tsnp](https://github.com/itszzl-sudo/tsnp) - Plugin generator

## License

MIT

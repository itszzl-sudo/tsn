# tsn

TypeScript to native executable compiler.

**Quick start with cargo-tsn:**

```bash
cargo install cargo-tsn
cargo tsn new my-project
cd my-project
tsn main.ts
./a.exe
```

See [cargo-tsn README](https://crates.io/crates/cargo-tsn) for project management.

## What is it

A compiler that converts TypeScript to native executable (10-14KB), no Node.js required.

## What can it do

- Compile TypeScript to native `.exe`
- Generate dependency-free executables
- Call Rust FFI functions
- No runtime overhead

## How to use

```bash
tsn main.ts
./a.exe
```

**Output:**
- `a.o` - Object file
- `a.exe` - Native executable

## Limitations

tsn has very few built-in functions (only `print`).

## Solution

Extend capabilities by integrating existing Rust crates via FFI.

## Extension mechanism

tsn scans `./tsnp/` directory for plugin configurations:

```
project/
├── tsnp/
│   └── my-plugin/
│       └── ts-native.toml
└── main.ts

tsn main.ts  # Automatically loads plugins from tsnp/
```

### ts-native.toml format

```toml
[package]
name = "tsnp-my-plugin"
version = "0.1.0"
tsnpVersion = "0.1.0"

[functions]
"add" = { args = ["number", "number"], ret = "number", impl_name = "add" }

[link]
lib = "my-plugin"
```

**Version Fields:**
- `version` - The original Rust crate version
- `tsnpVersion` - The tsnp plugin version (allows independent versioning)

### Using cargo-tsn (Recommended)

```bash
# Create project
cargo tsn new my-project
cd my-project

# Add Rust crate dependency
cargo tsn add regex

# Interactively add FFI functions
cargo tsn func

# Compile
tsn main.ts
```

See [cargo-tsn](https://crates.io/crates/cargo-tsn) for details.

### Using tsnp directly

```bash
# Generate plugin from Rust crate
tsnp gen my-plugin

# Output: tsnp/my-plugin/ts-native.toml
```

See [tsnp](https://crates.io/crates/tsnp) for details.

## How it works

### Compilation pipeline

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

### Value representation: NaN-boxing

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

**Benefits:**
- No boxing/unboxing overhead
- Type checks are bitmask operations
- Pointers stored directly in value

### Backend: Cranelift

- Modern code generator
- Optimized x86_64 output
- No LLVM dependency
- Fast compilation

### Memory management

- Arena allocation for AST
- Heap allocation for strings/arrays/objects
- No garbage collector (manual memory management)

### FFI integration

When tsn encounters a function call:

1. Check if function exists in `tsnp/*/ts-native.toml`
2. Load function signature (args, return type)
3. Generate FFI call to Rust function
4. Handle type conversions

Example:

```typescript
// TypeScript
const result = add(1, 2);
```

```rust
// Rust FFI
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

tsn generates:
- Load arguments to registers
- Call Rust function
- Store return value

## Supported TypeScript features

### Data types

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

### Control flow

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

### Data structures

- ✅ Array literals `[1, 2, 3]`
- ✅ Array indexing `arr[i]`
- ✅ Array assignment `arr[i] = value`
- ✅ Object literals `{x: 10}`
- ✅ Property access `obj.x`
- ✅ Property assignment `obj.x = value`

## Not implemented

- `break`, `continue`
- `switch` statement
- `class` syntax
- `async/await`
- Modules
- Generics

## Installation

```bash
cargo install tsn
```

## Project structure

```
tsn/
├── src/
│   ├── main.rs       # CLI entry point
│   ├── ts_parser.rs  # TypeScript lexer & parser
│   ├── codegen.rs    # Cranelift code generation
│   ├── linker.rs     # Native linker
│   ├── extension.rs  # Plugin loading
│   └── runtime.rs    # Runtime helpers
└── Cargo.toml
```

## Links

- [cargo-tsn](https://crates.io/crates/cargo-tsn) - Project manager
- [tsnp](https://crates.io/crates/tsnp) - Plugin generator
- [GitHub](https://github.com/itszzl-sudo/ts-native)

## License

MIT

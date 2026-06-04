# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-05-25

### Added
- GitHub Actions CI/CD configuration
  - Automatic testing on push and PR
  - Code formatting check
  - Clippy lint
  - Automatic publish to crates.io
- Issue templates (bug report, feature request)
- Pull request template
- Contributing guide (CONTRIBUTING.md)
- Code of Conduct (CODE_OF_CONDUCT.md)
- Project summary and release documentation

### Changed
- Improved README with installation guide and examples
- Added crates.io badges
- Enhanced Cargo.toml metadata

### Fixed
- Removed unused files from package (publish.sh)

## [0.1.0] - 2025-05-25

### Added
- Complete TypeScript subset compiler with Cranelift backend
- NaN-boxing value representation for all types
- Zero-dependency native executable generation

#### Data Types
- Numbers (integers and floats)
- Strings (dynamic allocation, concatenation)
- Arrays (dynamic allocation, nesting)
- Objects (dynamic allocation)
- Boolean, null, undefined

#### Operators
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Ternary: `cond ? then : else`
- Compound assignment: `+=`, `-=`, `*=`, `/=`
- Increment/Decrement: `++`, `--`
- typeof operator
- String concatenation with `+`

#### Control Flow
- `if` statements
- `if-else` statements
- `while` loops
- `for` loops
- `return` statements

#### Functions
- Function definitions and calls
- Multiple parameters
- Return values
- Recursion
- Mutual recursion

#### Data Structures
- Array literals `[1, 2, 3]`
- Array indexing `arr[i]`
- Array modification `arr[i] = value`
- Nested arrays
- Object literals `{x: 10}`
- Property access `obj.x`
- Property modification `obj.x = value`

#### Runtime
- Memory management (malloc, realloc)
- Type system (typeof)
- Array operations (new, push, get, set)
- Object operations (new, get, set)
- String operations (new, concat, add)
- I/O (print)

### Changed
- Initial release

### Performance
- Executable size: 10-14 KB
- Startup time: < 1ms
- No external dependencies

### Documentation
- README with installation guide
- Examples for common patterns
- Project summary document
- Test suite (60 test files)

### Testing
- 60 test files covering all features
- All tests passing
- Performance benchmarks included

## [Unreleased]

### To Add
- `break` and `continue` statements
- `switch` statements
- `do-while` loops
- Default parameters
- Rest parameters (`...args`)
- Classes and inheritance
- Module system
- Async/await
- Generics

---

[0.1.0]: https://github.com/itszzl-sudo/ts-native/releases/tag/v0.1.0

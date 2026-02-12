# Typed Path Project Setup

## Build Commands

```bash
# Build the project
cargo build

# Build with release
cargo build --release

# Run tests
cargo test

# Run specific tests
cargo test <test_name>

# Build documentation
cargo doc

# Run clippy linting
cargo clippy

# Format code
cargo fmt
```

## Code Style Guidelines

### Rust Code Style

- **Imports**: Uses `rustfmt.toml` with:
  - Max width 100 characters  
  - Unix newlines
  - Block indentation style
  - `imports_granularity = "Module"`
  - `group_imports = "StdExternalCrate"`
  - `unstable_features = true`

- **Formatting**: 
  - Uses rustfmt with consistent formatting
  - 100 character line width limit
  - Tab-indented with block style

- **Naming Conventions**: 
  - Module names in `snake_case`
  - Type names in `PascalCase`
  - Constants in `UPPER_CASE`
  - Associated types in `camelCase` with the prefix `T` (e.g., `T` for `Path<T>`)

- **Error Handling**:
  - Custom error types for path operations (`CheckedPathError`)
  - Uses `Result` and `Option` types for error handling
  - Error messages are descriptive with specific error cases
  - Provides type-safe error messages to avoid runtime panics

- **Documentation**:
  - Doc comments using `///`
  - Inline documentation for all public APIs
  - Examples in documentation demonstrating usage

## Cursor/Copilot Rules

- Uses Rust language features including:
  - Generic types with trait bounds
  - Feature flags (`std`, `no_std`)
  - Macros for code generation
  - `no_std` support for embedded environments
  - Cross-platform path handling for Unix and Windows
  - UTF8 enforcement capabilities

- Code structure and patterns:
  - Modular design with clear separation between:
    - `unix` and `windows` platforms
    - `non_utf8` and `utf8` path types
    - `typed` for runtime determination
    - `native` and `platform` for specific implementations
  - Reuse of common functionality through trait-based design
  - Support for `no_std` environments (feature flag based)
  - Path checking for traversal attacks (safe path joining/pushing)
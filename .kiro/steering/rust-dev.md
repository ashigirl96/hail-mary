# Rust Development Tools and Commands

## Unused Dependencies Detection
**When**: Analyzing Cargo.toml for unused dependencies
- cargo tree --duplicates shows version conflicts
- cargo-udeps available but requires nightly

```bash
# ✅ Good: Primary workflow
cargo install cargo-machete

# ✅ Good: Check for duplicates
cargo tree --duplicates

# ✅ Good: Verify actual usage
grep -r "library_name" src/

# ❌ Avoid: nightly-dependent tools unless necessary
cargo +nightly udeps
```

## Workspace Dependencies Detection
**When**: Detecting unused dependencies at workspace level
- cargo-shear analyzes workspace-level unused dependencies
- Use cargo-shear --fix for automatic removal
- Workspace deps defined but not referenced with { workspace = true } are detected
- cargo-shear uses syn crate for precise Rust code parsing

```bash
# ✅ Good: Complete detection workflow
cargo shear                  # check workspace
cargo shear --fix .          # auto-remove workspace unused

# ❌ Bad: Missing workspace-level analysis
cargo machete  # Only detects crate dependencies
```

## Test Execution With Just
**When**: Running tests in this project
- Always use `just test` instead of `cargo test`
- `just test` includes format check and clippy before testing
- Supports all cargo test arguments pass-through
- CI workflow configured to use just test

```bash
# ✅ Good: Use just for testing
just test                           # Run all tests with checks
just test domain::                  # Test specific module
just test -- --nocapture           # Pass cargo test flags
just test test_specific_name       # Run specific test

# ❌ Bad: Direct cargo test usage
cargo test                         # Missing format and lint checks
cargo test --quiet                 # Use just test instead
```
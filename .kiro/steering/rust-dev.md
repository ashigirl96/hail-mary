# Rust Development Tools and Commands

## Unused Dependencies Detection
**When**: Analyzing Cargo.toml for unused dependencies
- Use cargo-machete as primary tool (stable Rust, fast)
- Run with --with-metadata flag for better accuracy
- Verify with grep searches for actual usage
- cargo tree --duplicates shows version conflicts
- cargo-udeps available but requires nightly

```bash
# ✅ Good: Primary workflow
cargo install cargo-machete
cargo machete
cargo machete --with-metadata

# ✅ Good: Check for duplicates
cargo tree --duplicates

# ✅ Good: Verify actual usage
grep -r "library_name" src/

# ❌ Avoid: nightly-dependent tools unless necessary
cargo +nightly udeps
```

## Workspace Dependencies Detection
**When**: Detecting unused dependencies at workspace level
- cargo-machete only checks crate-level dependencies
- cargo-shear analyzes workspace-level unused dependencies
- Use cargo-shear --fix for automatic removal
- Workspace deps defined but not referenced with { workspace = true } are detected
- cargo-shear uses syn crate for precise Rust code parsing

```bash
# ✅ Good: Complete detection workflow
cargo install cargo-machete  # crate-level
cargo install cargo-shear    # workspace-level
cargo machete                 # check crates
cargo shear                   # check workspace
cargo shear --fix            # auto-remove workspace unused

# ❌ Bad: Missing workspace-level analysis
cargo machete  # Only detects crate dependencies
```
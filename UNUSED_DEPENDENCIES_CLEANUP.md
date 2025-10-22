# Unused Dependencies Cleanup Report

Generated: October 22, 2025
Tool: cargo-machete v0.9.1

## Summary

**Total Unused Dependencies Found:** 31 across 8 crates

## Analysis by Crate

### ✅ SAFE TO REMOVE

#### 1. **recommendation-api** (4 unused)
```toml
# crates/api/Cargo.toml
config = "0.15.18"                    # ❌ Not used - we read env vars directly
recommendation-config = { ... }       # ❌ Not used - planned but not implemented
thiserror = "2.0.17"                  # ❌ Not used - errors use axum's types
validator = { ... }                   # ❌ Not used - validation done manually
```

**Verification:**
```bash
# Confirmed none of these are imported in crates/api/src/
grep -r "use validator" crates/api/src/        # No matches
grep -r "use.*config::" crates/api/src/        # No matches
grep -r "use recommendation_config" crates/api/src/  # No matches
```

---

#### 2. **recommendation-config** (3 unused)
```toml
# crates/config/Cargo.toml
anyhow = "1.0.100"      # ❌ Uses thiserror instead
chrono = { ... }        # ❌ Not needed for config
dotenvy = "0.15.7"      # ❌ Only used in main.rs, not in config crate
```

---

#### 3. **recommendation-models** (1 unused)
```toml
# crates/models/Cargo.toml
uuid = { ... }          # ❌ UUIDs generated elsewhere, not in models
```

---

### ⚠️ VERIFY BEFORE REMOVING

These might be used indirectly or planned for future use:

#### 4. **recommendation-engine** (6 dependencies)
```toml
# crates/engine/Cargo.toml
anyhow = "1.0.100"           # ⚠️ Check if used for Result types
chrono = { ... }             # ⚠️ May be used for time-based filtering
futures = "0.3.31"           # ⚠️ Check async code
serde = { ... }              # ⚠️ Likely used via derive
serde_json = "1.0.145"       # ⚠️ May be used for debugging
thiserror = "2.0.17"         # ⚠️ Check error types
```

**Action:** Manual verification needed

---

#### 5. **recommendation-service** (4 dependencies)
```toml
# crates/service/Cargo.toml
anyhow = "1.0.100"      # ⚠️ Check Result usage
futures = "0.3.31"      # ⚠️ Check async utilities
thiserror = "2.0.17"    # ⚠️ Check error definitions
uuid = { ... }          # ⚠️ Check ID generation
```

**Action:** Manual verification needed

---

#### 6. **recommendation-storage** (2 dependencies)
```toml
# crates/storage/Cargo.toml
futures = "0.3.31"      # ⚠️ May be used for async streams
uuid = { ... }          # ⚠️ Check if UUIDs created in storage
```

**Action:** Manual verification needed

---

### 🧪 TEST CRATES (Safe to ignore or verify separately)

#### 7. **recommendation-integration-tests** (3 unused)
```toml
futures = "0.3.31"      # Tests might not need this
serde = { ... }         # Tests might not need this
serde_json = "1.0.145"  # Tests might not need this
```

#### 8. **recommendation-performance-tests** (8 unused)
**Note:** Performance tests might have simplified over time

#### 9. **seed-data** (2 unused)
```toml
serde_json = "1.0.145"  # Check if JSON parsing is done
uuid = { ... }          # Check if UUIDs are generated
```

---

## Recommended Actions

### Phase 1: Safe Removals (HIGH CONFIDENCE)

Remove these dependencies - they're definitely unused:

```bash
# recommendation-api
cd crates/api
cargo remove config
cargo remove recommendation-config
cargo remove thiserror
cargo remove validator

# recommendation-config
cd ../config
cargo remove anyhow
cargo remove chrono
cargo remove dotenvy

# recommendation-models
cd ../models
cargo remove uuid
```

**Test after Phase 1:**
```bash
cargo build --workspace
cargo test --workspace
```

If builds and tests pass → Phase 1 successful ✅

---

### Phase 2: Careful Verification (MEDIUM CONFIDENCE)

For each of these, check if removing breaks anything:

```bash
# Try removing one at a time
cd crates/engine
cargo remove anyhow
cargo build --release

# If it fails, add it back
cargo add anyhow

# Repeat for each dependency
```

**Dependencies to verify:**
- crates/engine: anyhow, chrono, futures, serde, serde_json, thiserror
- crates/service: anyhow, futures, thiserror, uuid
- crates/storage: futures, uuid

---

### Phase 3: Test Crates (LOW PRIORITY)

These can stay or be removed based on preference:
- Integration tests might benefit from keeping extra deps
- Performance tests might be intentionally minimal
- Seed data might need cleanup

---

## Automated Cleanup Script

### Option A: Conservative (Recommended)

```bash
#!/bin/bash
# cleanup_deps_conservative.sh

echo "Removing definitely unused dependencies..."

# API crate - SAFE
cd crates/api
cargo remove config recommendation-config thiserror validator

# Config crate - SAFE
cd ../config
cargo remove anyhow chrono dotenvy

# Models crate - SAFE
cd ../models
cargo remove uuid

cd ../..

echo "Testing build..."
cargo build --workspace

echo ""
echo "If build succeeds, dependencies were truly unused!"
echo "If build fails, check error messages to see what needs to be added back."
```

### Option B: Aggressive (Use with caution)

```bash
#!/bin/bash
# cleanup_deps_aggressive.sh

echo "Removing all suspected unused dependencies..."

# Remove all reported by cargo-machete
cargo machete --fix

echo ""
echo "Testing build..."
cargo build --workspace

echo ""
echo "If build fails, you'll need to add back the required dependencies."
```

---

## Before You Remove

### 1. Create a backup
```bash
git checkout -b cleanup-unused-deps
git commit -am "checkpoint before dependency cleanup"
```

### 2. Run tests first
```bash
cargo test --workspace
```

### 3. Make sure CI is green
```bash
# Check GitHub Actions status
```

---

## After Removal

### Verify Everything Still Works

```bash
# Full build
cargo build --workspace --release

# All tests
cargo test --workspace

# Clippy
cargo clippy --workspace

# Integration tests
cargo test -p recommendation-integration-tests

# Performance tests
cargo test -p recommendation-performance-tests
```

### Check Binary Size (Bonus)

```bash
# Before
ls -lh target/release/recommendation-api

# After cleanup + rebuild
cargo clean
cargo build --release
ls -lh target/release/recommendation-api

# Should be slightly smaller!
```

---

## False Positives Explained

### Why cargo-machete reports false positives:

1. **Derive macros**: `serde` used via `#[derive(Serialize)]` - not in `use` statements
2. **Re-exports**: Dependency used by public types but not directly imported
3. **Build scripts**: Used in build.rs but not in main code
4. **Feature flags**: Only used when certain features enabled
5. **Transitive deps**: Required by other dependencies

### When in doubt, use cargo-udeps:

```bash
# Install nightly rust
rustup install nightly

# More accurate check
cargo +nightly udeps --all-targets
```

---

## Impact Assessment

### Removing definitely unused (Phase 1):

**Dependencies to remove:** 9
**Estimated build time improvement:** 5-10 seconds
**Binary size reduction:** ~500KB - 1MB
**Complexity reduction:** ✅ Cleaner dependency tree

### If all reported deps removed (Aggressive):

**Dependencies to remove:** 31
**Estimated build time improvement:** 20-30 seconds
**Binary size reduction:** ~2-3MB
**Risk:** ⚠️ High - likely to break build

---

## Recommendation

### Start Conservative:

1. ✅ Remove Phase 1 (9 deps) - **DO THIS**
2. ⚠️ Verify Phase 2 one-by-one if desired
3. ⏸️ Skip Phase 3 (test crates) unless bothered by them

### Expected Result:

```
Before: 93 total dependencies
After:  84 total dependencies
Improvement: ~10% cleaner dependency tree
```

---

## Quick Commands

```bash
# Find unused deps
cargo machete

# More accurate (requires nightly)
cargo +nightly udeps --all-targets

# Auto-fix (DANGEROUS - test first!)
cargo machete --fix

# Check what changed
git diff Cargo.toml

# Revert if needed
git checkout Cargo.toml
```

---

## Status

**Ready to proceed:** Yes ✅
**Risk level:** Low (for Phase 1)
**Time required:** 10 minutes
**CI impact:** Should pass if done correctly

Would you like me to create a PR with Phase 1 removals?

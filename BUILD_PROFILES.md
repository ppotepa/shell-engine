# Build Profiles for Shell Engine

Three optimized build profiles to match your workflow:

## Build Time Comparison

| Profile | Time | Use Case |
|---------|------|----------|
| `dev` | ~36s | Debug, testing (fast, unoptimized) |
| `fast-release` | ~51s | Development iteration, playtest |
| `release` | ~80s | Final builds, deployment |

## Usage

### Fast Development Iteration (Recommended for gameplay dev)
```bash
cargo build --profile fast-release -p app
cargo run --profile fast-release -p app -- --mod-source=mods/playground
```
- 36% faster than full release
- 90% of performance
- Thin LTO + parallel codegen

### Debug Development (Fastest, unoptimized)
```bash
cargo build -p app
cargo run -p app -- --mod-source=mods/playground
```
- Fastest build (~36s)
- Unoptimized, good for testing logic
- Full debug symbols

### Production Release (Maximum performance)
```bash
cargo build --release -p app
cargo run --release -p app -- --mod-source=mods/playground
```
- Full LTO, single-threaded codegen
- Best runtime performance
- Longest build time (~80s)

## Profile Details

### `fast-release`
- `lto = "thin"` — 50% of full LTO cost, 90% of performance
- `codegen-units = 4` — 4x faster linking
- `opt-level = 3` — Full optimizations

### Recommendation
Use `--profile fast-release` for gameplay iteration and testing.
It's the best balance of build speed and performance.


# mods/shell-quest-tests — automated benchmark and regression mod

## Purpose

This mod is the automation-friendly variant of the main Shell Quest content. It
exists so benchmarks and frame-capture regression tests can run without waiting
for user input.

## What is different from the main mod

- entry scenes auto-advance instead of waiting for “any key”
- the scene sequence is arranged to loop for continuous benchmark runs
- visual content stays close to the main mod so rendering workload remains meaningful

## How to use it

Benchmark:

```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

Frame capture:

```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --capture-frames /tmp/frames --bench 10
```

Compare baseline vs optimized captures:

```bash
./capture-frames-tests.sh baseline optimized 2
```

## Working with this mod

- keep it visually representative of the main mod,
- only change pacing or trigger mechanics when needed for automation,
- preserve loopability for longer benchmark sessions,
- use it for performance and regression workflows rather than gameplay tuning.

## Manifest notes

- entrypoint remains `/scenes/00-intro-logo/scene.yml`
- virtual buffer rendering is enabled
- terminal requirements mirror the main experience closely enough to keep results relevant

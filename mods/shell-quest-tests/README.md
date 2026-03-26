# shell-quest-tests

Automation-friendly benchmark and regression mod.

## Purpose

This mod mirrors the main Shell Quest content closely enough to exercise the
same rendering path, but removes or replaces user-blocking interactions so it
can be used in automated benchmarks and frame-capture regression runs.

## Typical usage

```bash
cargo run -p app -- --mod shell-quest-tests --bench 5
cargo run -p app -- --mod shell-quest-tests --capture-frames /tmp/frames --bench 5
```

## Related docs

- `README.AGENTS.MD` for benchmark/regression workflow details
- root `BENCHMARKING.md` for broader performance workflows

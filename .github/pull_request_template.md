## Description

Brief description of what this PR does.

## Related Issue

Closes # (issue number)

## Changes

- What changed
- Why it changed

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Refactor (no functional changes)

## Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] New tests added (if applicable)
- [ ] Doc-comments added / updated (if applicable)
- [ ] Contract compiles to WASM: `cargo build --target wasm32v1-none --release`

## Testing

Describe the tests you ran and how to reproduce them:

```
cargo test -p trickle-stream
```

## Screenshots / Logs

If applicable, add transaction output or test output.

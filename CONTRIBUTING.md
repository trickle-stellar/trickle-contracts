# Contributing to trickle-contracts

Thanks for your interest in contributing to Trickle! This guide will help you get started.

## Getting Started

### Prerequisites

- Rust (stable, 1.94+)
- [Stellar CLI](https://developers.stellar.org/docs/tools/smart-contracts-cli)
- WASM target: `rustup target add wasm32v1-none`

### Setup

```bash
git clone https://github.com/trickle-stellar/trickle-contracts.git
cd trickle-contracts
cargo test
```

If tests pass, you're good to go.

## Difficulty Map

| Level | What's involved | Example issues |
|-------|----------------|----------------|
| **Good First Issue** | Pure functions, math, storage helpers, events | Implement `calculate_vested`, add storage read/write helpers, write event emitters |
| **Intermediate** | Contract functions with auth, token transfers, state changes | Implement `withdraw`, `pause`/`resume`, `claim`, `add_recipient` |
| **Advanced** | Cross-contract calls, factory integration, integration tests | Implement `deploy_stream`, `transfer` (NFT), `cancel` (multi-contract refund), `update_recipient` |

## Picking Up Issues

1. Browse [Issues](https://github.com/trickle-stellar/trickle-contracts/issues) — look for labels:
   - `good-first-issue` — self-contained, well-scoped, great for first-time contributors
   - `intermediate` — requires understanding of Soroban auth or token patterns
   - `advanced` — multi-contract interactions, factory patterns, integration tests
2. Comment on the issue to claim it
3. Fork the repo and create a branch: `git checkout -b feat/short-description`
4. Implement, test, and open a PR

## Project Structure

```
trickle-contracts/
├── Cargo.toml                    # Workspace root
├── contracts/
│   ├── common/                   # Shared types (rlib)
│   │   └── src/lib.rs
│   ├── stream/                   # Per-stream contract
│   │   └── src/
│   │       ├── lib.rs            # Thin entry points
│   │       ├── storage.rs        # DataKey + read/write helpers
│   │       ├── math.rs           # Pure calculations
│   │       ├── events.rs         # Event emission
│   │       └── test.rs           # Unit tests
│   ├── factory/                  # Deploys stream instances
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── storage.rs
│   │       ├── deploy.rs         # Contract deployment logic
│   │       ├── events.rs
│   │       └── test.rs
│   ├── fees/                     # Protocol fee collection
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── storage.rs
│   │       ├── math.rs           # fee calculation
│   │       ├── events.rs
│   │       └── test.rs
│   ├── multistream/              # Multi-recipient split
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── storage.rs
│   │       ├── math.rs           # proportional share math
│   │       ├── events.rs
│   │       └── test.rs
│   ├── vesting/                  # Time-locked release
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── storage.rs
│   │       ├── math.rs           # vesting schedule math
│   │       ├── events.rs
│   │       └── test.rs
│   └── stream-nft/               # Receiver role as token
│       └── src/
│           ├── lib.rs
│           ├── storage.rs
│           ├── events.rs
│           └── test.rs
└── .github/                      # Issue & PR templates
```

## Code Style

### Rust

- Format with `cargo fmt` — all code must be formatted before merging
- Lint with `cargo clippy` — no warnings allowed
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) where applicable

### Soroban Conventions

- Keep `#![no_std]` at the top of every contract file
- Use `#[contracttype]` on all structs/enums that go into storage
- Use `#[contracterror]` with `#[repr(u32)]` for error enums
- Every public contract function takes `Env` as its first parameter
- Use `require_auth()` for any mutating operation that should be authorized
- Use `soroban_sdk::token::Client` for token transfers (never raw balance manipulation)
- `symbol_short!` strings must be 9 characters or fewer

### Naming

- Contract structs: `PascalCase` (e.g., `StreamContract`)
- Storage key enums: `PascalCase` variants (e.g., `DataKey::Stream(u32)`)
- Error enums: `PascalCase` variants (e.g., `StreamError::StreamNotFound`)
- Functions: `snake_case` (e.g., `create_stream`, `get_claimable_balance`)

## PR Guidelines

1. **One concern per PR** — don't bundle unrelated changes
2. **Write tests** — every new function needs at least one test
3. **Document with doc-comments** — explain intent, not just mechanics
4. **Keep stubs honest** — if a function is not yet implemented, use `todo!()` with a comment explaining what it should do
5. **Run the full check suite before submitting:**

```bash
cargo fmt
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
```

6. **PR description** should reference the issue it addresses (e.g., "Closes #12")
7. **Keep PRs small** — easier to review, faster to merge

## Testing Requirements

- Every contract function must have at least one test
- Use the `setup()` helper pattern shown in `contracts/stream/src/test.rs`
- Use `env.mock_all_auths()` to avoid manual auth setup in tests
- Use `Address::generate(&env)` for test addresses
- Test both success paths and error paths
- For functions involving time, use `env.ledger().with_mut(|l| l.timestamp = ...)` to advance the ledger
- Prefer testing pure math functions in `math.rs` directly — they don't need an Env

### Test naming convention

```
test_<function_name>_<scenario>
```

Examples:
- `test_create_stream_success`
- `test_create_stream_zero_amount_errors`
- `test_withdraw_returns_accrued_balance`
- `test_pause_stream_unauthorized_errors`

## Questions?

Open a [Discussion](https://github.com/trickle-stellar/trickle-contracts/discussions) or ask in the Stellar Discord `#trickle` channel.

# trickle-contracts

Soroban smart contracts for **Trickle** — a continuous payment streaming protocol on Stellar.

> Every second counts.

Money flows per second. No more end-of-month payroll. No more 30-day invoice cycles. Stream salary, subscriptions, royalties, and contributions in real time.

## Architecture

```
                            ┌─────────────────────────┐
                            │   Factory Contract      │
                            │   (deploy + registry)   │
                            └───────────┬─────────────┘
                                        │ deploy
            ┌───────────────────────────┼───────────────────────────┐
            ▼                           ▼                           ▼
   ┌─────────────────┐        ┌─────────────────┐        ┌─────────────────┐
   │ Stream Contract │        │ Stream Contract │        │ Stream Contract │
   │     #0          │        │     #1          │        │     #2          │
   │ (per-stream)    │        │ (per-stream)    │        │ (per-stream)    │
   └────────┬────────┘        └────────┬────────┘        └────────┬────────┘
            │                          │                          │
            │  ┌───────────────────────┼──────────────────────┐   │
            │  │                       │                      │   │
            ▼  ▼                       ▼                      ▼   ▼
   ┌─────────────────┐        ┌─────────────────┐    ┌─────────────────┐
   │  Stream NFT     │        │  Multistream    │    │  Vesting        │
   │  (receiver role │        │  (split across  │    │  (time-locked   │
   │   as token)     │        │   recipients)   │    │   release)      │
   └─────────────────┘        └─────────────────┘    └─────────────────┘

   ┌─────────────────┐        ┌─────────────────┐
   │  Fees Contract  │◄───────│  Factory        │
   │  (protocol fee  │        │  (on create)    │
   │   collection)   │        └─────────────────┘
   └─────────────────┘

   ┌───────────────────────────────────────────────────────────────┐
   │  trickle-common (rlib)                                       │
   │  Shared types: StreamInfo, RecipientInfo, VestingInfo,       │
   │  VestingStatus, StreamError, MultistreamError,               │
   │  VestingError, FeeError, StreamNftError                      │
   └───────────────────────────────────────────────────────────────┘
```

## Contracts

### `trickle-common` (shared types)

Shared types and error enums used by all contracts. An `rlib` crate — not deployed directly.

### `trickle-stream` (scaffolded)

Per-stream contract instance. Deployed by the factory. Holds escrowed tokens and manages its own accrual, withdrawal, pause/resume, and cancellation logic.

| Function | Description |
|----------|-------------|
| `initialize` | One-time setup after factory deployment |
| `withdraw` | Recipient claims accrued balance |
| `pause` | Sender pauses accrual (resumable) |
| `resume` | Sender resumes a paused stream |
| `cancel` | Sender cancels — unstreamed funds returned |
| `get_balance` | Read-only: current claimable amount |
| `get_info` | Read-only: full stream metadata |
| `update_recipient` | NFT contract transfers receivership |

### `trickle-factory` (scaffolded)

Factory that deploys per-stream contract instances and maintains a registry.

| Function | Description |
|----------|-------------|
| `initialize` | One-time setup with admin + WASM hash |
| `create_stream` | Deploy new stream contract |
| `get_stream` | Cached stream metadata by ID |
| `get_streams_by_sender` | All stream IDs for a sender |
| `get_streams_by_recipient` | All stream IDs for a recipient |

### `trickle-fees` (scaffolded)

Protocol fee collection. Upgradeable independently.

| Function | Description |
|----------|-------------|
| `initialize` | Set admin + fee recipient |
| `calculate_fee` | Pure: amount * rate / 10_000 |
| `collect_creation_fee` | Collect fee on stream creation |
| `collect_withdrawal_fee` | Collect fee on withdrawal |
| `set_fee_rate` | Update fee rate (admin only) |
| `withdraw_fees` | Send accumulated fees to recipient |

### `trickle-multistream` (scaffolded)

Split a single stream across multiple recipients by weight.

| Function | Description |
|----------|-------------|
| `initialize` | Set up multi-recipient stream |
| `add_recipient` | Add recipient with proportional weight |
| `remove_recipient` | Remove recipient, return share to sender |
| `withdraw` | Recipient claims their proportional share |
| `pause` / `resume` / `cancel` | Lifecycle management |
| `get_claimable` | Read-only: claimable for a recipient |

### `trickle-vesting` (scaffolded)

Time-locked token release with cliff + linear vesting.

| Function | Description |
|----------|-------------|
| `initialize` | Create vesting schedule |
| `claim` | Beneficiary claims vested tokens |
| `revoke` | Admin revokes (if revocable) |
| `get_status` | Read-only: full vesting info |
| `get_flow_rate` | Read-only: tokens per second |

### `trickle-stream-nft` (scaffolded)

Wraps stream receiver role as a Stellar custom asset. Transferring the token transfers the right to receive future payments.

| Function | Description |
|----------|-------------|
| `initialize` | Set admin + stream contract |
| `mint` | Create NFT for stream receiver |
| `transfer` | Transfer receivership to new address |
| `burn` | Revoke receiver role |
| `owner_of` | Read-only: current owner |

## Project Structure

```
trickle-contracts/
├── Cargo.toml                    # Workspace root (soroban-sdk v22)
├── LICENSE
├── README.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── .editorconfig
├── .github/
│   ├── workflows/ci.yml
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── contract_task.md
│   │   └── feature_request.md
│   └── pull_request_template.md
└── contracts/
    ├── common/                   # Shared types (trickle-common)
    │   └── src/lib.rs
    ├── stream/                   # Per-stream escrow contract
    ├── factory/                  # Deploy + registry
    ├── fees/                     # Protocol fee management
    ├── multistream/              # Multi-recipient splits
    ├── vesting/                  # Cliff + linear vesting
    └── stream-nft/               # Stream receivership as NFT
```

## Module Structure

Each contract follows a consistent modular layout:

```
contracts/<name>/
├── Cargo.toml
└── src/
    ├── lib.rs        # Thin entry points (contract functions)
    ├── storage.rs    # DataKey enum + read/write helpers
    ├── math.rs       # Pure calculation functions (no env access)
    ├── events.rs     # Event emission helpers
    └── test.rs       # Unit tests (in src/, imported via mod test)
```

## Local Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.94+)
- [Stellar CLI](https://developers.stellar.org/docs/tools/smart-contracts-cli)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32v1-none

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Build

```bash
# Clone the repo
git clone https://github.com/trickle-stellar/trickle-contracts.git
cd trickle-contracts

# Build all contracts (produces WASM in target/wasm32v1-none/release/)
cargo build --target wasm32v1-none --release

# Or use the Stellar CLI
stellar contract build
```

### Test

```bash
# Run all tests
cargo test

# Run tests for a specific contract
cargo test -p trickle-stream

# Run with output
cargo test -- --nocapture
```

### Deploy to Testnet

```bash
# Generate a testnet identity
stellar keys generate --global deployer --network testnet --fund

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/trickle_stream.wasm \
  --source deployer \
  --network testnet \
  -- \
  --constructor-arg deployer=<YOUR_ADDRESS>
```

## How It Works

1. **Sender** creates a stream via the factory: specifies recipient, token, total amount, and duration
2. The factory deploys a new **Stream Contract** instance with the tokens in escrow
3. The contract calculates a **per-second flow rate** (total_amount / duration)
4. **Recipient** can withdraw accrued balance at any time — the contract calculates exactly how much has streamed based on elapsed ledger time
5. Sender can **pause** (accrual stops), **resume** (accrual restarts), or **cancel** (unstreamed funds returned)
6. **Stream NFT** wraps the receiver role as a transferable token
7. **Multistream** splits payments across multiple recipients by weight
8. **Vesting** time-locks token release with cliff + linear schedule
9. **Fees** are collected on creation and/or withdrawal

All time-based accounting uses Stellar ledger close times — no external oracle needed.

## Architecture Decisions

- **Factory pattern**: Each stream is its own contract instance (cleaner isolation, upgradeable independently)
- **Shared types via `trickle-common`**: `rlib` crate with all shared structs/enums — no duplication
- **Internal vs public types**: `StreamConfig` is internal mutable state; `StreamInfo` is public read-only
- **Ledger timestamps**: All time accounting uses `env.ledger().timestamp()` — no external oracles
- **Token interface**: All token transfers go through `soroban_sdk::token::Client` (Stellar token standard)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines, difficulty map, and how to pick up issues.

## License

MIT © 2026 Trickle Protocol

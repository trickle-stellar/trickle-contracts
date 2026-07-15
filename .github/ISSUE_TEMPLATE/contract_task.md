---
name: Contract Task
about: Pick up a contract implementation task
title: "[contract] "
labels: contract-task
assignees: ''
---

## Contract

<!-- Which contract are you working on? -->
- [ ] common — shared types and error enums
- [ ] stream — per-stream contract (initialize, withdraw, pause, resume, cancel)
- [ ] factory — deploy stream instances and maintain registry
- [ ] fees — protocol fee calculation and collection
- [ ] multistream — multi-recipient proportional split
- [ ] vesting — time-locked release with cliff + linear vesting
- [ ] stream-nft — receiver role as transferable token

## Task Type

- [ ] New function implementation
- [ ] Test coverage
- [ ] Bug fix
- [ ] Optimization
- [ ] Documentation

## Difficulty

<!-- See CONTRIBUTING.md for the full difficulty map -->
- [ ] Good first issue — pure math, storage helpers, event emitters
- [ ] Intermediate — contract functions with auth, token transfers
- [ ] Advanced — cross-contract calls, factory integration, integration tests

## Description

<!-- What needs to be implemented or changed? -->

## Acceptance Criteria

- [ ] Function compiles and passes `cargo check`
- [ ] At least one test covers the new/changed function
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Doc comments added for public functions

## Related Contracts

<!-- If this involves cross-contract calls, list the other contracts involved. -->

## Implementation Notes

<!-- Any hints about implementation approach, storage patterns, or design decisions. -->

- See the contract's `lib.rs` for TODO comments that describe the intended implementation flow
- Each contract follows modular structure: `lib.rs` (entry points), `storage.rs` (DataKey + helpers), `math.rs` (pure calculations), `events.rs` (emission), `test.rs` (tests)
- Shared types live in `trickle-common` — import via `trickle_common::{TypeName}`

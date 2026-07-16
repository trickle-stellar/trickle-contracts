#![cfg(test)]

use super::*;

// ═══════════════════════════════════════════════════════════════════════════════
// Tests for vesting math (pure functions)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_vested_before_cliff() {
    let vested = math::calculate_vested(100_000, 0, 2_592_000, 31_536_000).unwrap();
    assert_eq!(vested, 0);
}

#[test]
fn test_vested_at_cliff() {
    let vested = math::calculate_vested(100_000, 2_592_000, 2_592_000, 31_536_000).unwrap();
    assert_eq!(vested, 0);
}

#[test]
fn test_vested_midway() {
    let elapsed = 2_592_000 + (31_536_000 - 2_592_000) / 2;
    let vested = math::calculate_vested(100_000, elapsed, 2_592_000, 31_536_000).unwrap();
    assert!(vested > 49_000 && vested < 51_000);
}

#[test]
fn test_vested_at_end() {
    let vested = math::calculate_vested(100_000, 31_536_000, 2_592_000, 31_536_000).unwrap();
    assert_eq!(vested, 100_000);
}

#[test]
fn test_claimable_basic() {
    let claimable =
        math::calculate_claimable(100_000, 31_536_000, 2_592_000, 31_536_000, 0).unwrap();
    assert_eq!(claimable, 100_000);
}

#[test]
fn test_claimable_after_partial_claim() {
    let claimable =
        math::calculate_claimable(100_000, 31_536_000, 2_592_000, 31_536_000, 60_000).unwrap();
    assert_eq!(claimable, 40_000);
}

#[test]
fn test_claimable_before_cliff() {
    let claimable = math::calculate_claimable(100_000, 0, 2_592_000, 31_536_000, 0).unwrap();
    assert_eq!(claimable, 0);
}

#[test]
fn test_invalid_schedule() {
    let result = math::calculate_vested(100_000, 0, 100, 50);
    assert_eq!(result, Err(VestingError::NothingToVest));
}

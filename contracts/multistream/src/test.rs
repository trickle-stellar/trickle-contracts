#![cfg(test)]

use super::*;

// ═══════════════════════════════════════════════════════════════════════════════
// Tests for multistream math (pure functions)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_recipient_share_basic() {
    let share = math::calculate_recipient_share(10_000, 3, 10, 0);
    assert_eq!(share, 3_000);
}

#[test]
fn test_recipient_share_after_withdrawal() {
    let share = math::calculate_recipient_share(10_000, 3, 10, 1_000);
    assert_eq!(share, 2_000);
}

#[test]
fn test_recipient_share_nothing_accrued() {
    let share = math::calculate_recipient_share(0, 3, 10, 0);
    assert_eq!(share, 0);
}

#[test]
fn test_recipient_share_full_weight() {
    let share = math::calculate_recipient_share(10_000, 10, 10, 0);
    assert_eq!(share, 10_000);
}

#[test]
fn test_recipient_share_overdrawn() {
    let share = math::calculate_recipient_share(10_000, 3, 10, 5_000);
    assert_eq!(share, 0);
}

#[test]
fn test_recipient_share_zero_total_weight() {
    let share = math::calculate_recipient_share(10_000, 3, 0, 0);
    assert_eq!(share, 0);
}

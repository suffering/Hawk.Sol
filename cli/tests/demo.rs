//! Demo unit tests — no validator required.

use solhawk::commands::demo::{compute_summary, DemoKeypairs, SummaryInput};

#[test]
fn summary_math_typical() {
    let out = compute_summary(SummaryInput {
        vault_initial: 1_000_000_000,
        vault_before_act_ii: 950_000_000,
        vault_after_act_ii: 900_000_000,
        threshold: 100_000_000,
        mint_decimals: 6,
        unit_price: 1.0,
    });
    assert_eq!(out.attack_extracted, 50_000_000);
    assert_eq!(out.attack_extracted_pct_of_initial, 5);
    assert_eq!(out.total_window_outflow, 100_000_000);
    assert_eq!(out.protected, 900_000_000);
}

#[test]
fn summary_math_at_cap() {
    let out = compute_summary(SummaryInput {
        vault_initial: 200,
        vault_before_act_ii: 150,
        vault_after_act_ii: 100,
        threshold: 100,
        mint_decimals: 0,
        unit_price: 2.0,
    });
    assert_eq!(out.attack_extracted, 50);
    assert_eq!(out.total_window_outflow, 100);
}

#[test]
fn deterministic_keypairs_same_seed() {
    let a = DemoKeypairs::from_seed(42);
    let b = DemoKeypairs::from_seed(42);
    assert_eq!(a.pubkeys(), b.pubkeys());
}

#[test]
fn deterministic_keypairs_different_seed() {
    let a = DemoKeypairs::from_seed(42);
    let b = DemoKeypairs::from_seed(43);
    assert_ne!(a.pubkeys().operator, b.pubkeys().operator);
}

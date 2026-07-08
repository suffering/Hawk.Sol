use anyhow::Result;
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;

use super::summary::{compute_summary, SummaryInput, SummaryOutput};

#[derive(Debug, Clone, Serialize)]
pub struct DemoJson {
    pub seed: u64,
    pub vault: String,
    pub mint: String,
    pub threshold: u64,
    pub vault_initial: u64,
    pub vault_before_act_ii: u64,
    pub vault_after_act_ii: u64,
    /// Number of `guarded_withdraw` attempts in Act II before trip is detected.
    ///
    /// Counts every `try_execute_transaction` call until the breaker is recognized as
    /// Tripped, **including** the exact-at-cap successful withdrawal (funds move, state
    /// stays Active) **and** the next successful velocity-trip transaction (returns Ok
    /// without moving funds). Excludes post-trip blocked attempts.
    pub attack_attempts_before_trip: u32,
    pub blocked_attempts_after_trip: u32,
    pub attack_extracted: u64,
    pub attack_extracted_pct_of_initial: u8,
    pub total_window_outflow: u64,
    pub protected: u64,
    pub protected_usd: f64,
    pub unit_price: f64,
    pub mint_decimals: u8,
}

pub fn build_demo_json(
    seed: u64,
    vault: &Pubkey,
    mint: &Pubkey,
    summary_input: SummaryInput,
    attack_attempts_before_trip: u32,
    blocked_attempts_after_trip: u32,
) -> DemoJson {
    let SummaryOutput {
        attack_extracted,
        attack_extracted_pct_of_initial,
        protected,
        protected_usd,
        total_window_outflow,
    } = compute_summary(summary_input);

    DemoJson {
        seed,
        vault: vault.to_string(),
        mint: mint.to_string(),
        threshold: summary_input.threshold,
        vault_initial: summary_input.vault_initial,
        vault_before_act_ii: summary_input.vault_before_act_ii,
        vault_after_act_ii: summary_input.vault_after_act_ii,
        attack_attempts_before_trip,
        blocked_attempts_after_trip,
        attack_extracted,
        attack_extracted_pct_of_initial,
        total_window_outflow,
        protected,
        protected_usd,
        unit_price: summary_input.unit_price,
        mint_decimals: summary_input.mint_decimals,
    }
}

pub fn print_demo_json(value: &DemoJson) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

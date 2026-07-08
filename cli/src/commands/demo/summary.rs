/// Attack-phase summary math (chain balances, not tx counting).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SummaryInput {
    pub vault_initial: u64,
    /// Vault SPL balance immediately before Act II begins.
    pub vault_before_act_ii: u64,
    /// Vault SPL balance after Act II completes (trip + blocked attempts).
    pub vault_after_act_ii: u64,
    pub threshold: u64,
    pub mint_decimals: u8,
    pub unit_price: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryOutput {
    /// Attack take only: `vault_before_act_ii - vault_after_act_ii`.
    pub attack_extracted: u64,
    pub attack_extracted_pct_of_initial: u8,
    pub protected: u64,
    pub protected_usd: f64,
    /// Total outflow in the demo window: `vault_initial - vault_after_act_ii`.
    pub total_window_outflow: u64,
}

pub fn compute_summary(input: SummaryInput) -> SummaryOutput {
    let attack_extracted = input.vault_before_act_ii.saturating_sub(input.vault_after_act_ii);
    let protected = input.vault_after_act_ii;
    let total_window_outflow = input.vault_initial.saturating_sub(input.vault_after_act_ii);

    let attack_extracted_pct_of_initial = if input.vault_initial == 0 {
        0
    } else {
        (attack_extracted
            .saturating_mul(100)
            .saturating_div(input.vault_initial))
        .min(100) as u8
    };

    let scale = 10f64.powi(input.mint_decimals as i32);
    let protected_usd = (protected as f64 / scale) * input.unit_price;

    SummaryOutput {
        attack_extracted,
        attack_extracted_pct_of_initial,
        protected,
        protected_usd,
        total_window_outflow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_math_typical_defaults() {
        // 1000 tokens initial, Act I removes 50, one 50-token attack chunk, trip blocks rest.
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
        assert_eq!(out.protected, 900_000_000);
        assert_eq!(out.total_window_outflow, 100_000_000);
        assert!((out.protected_usd - 900.0).abs() < f64::EPSILON);
    }

    #[test]
    fn summary_math_at_cap_edge() {
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
        assert_eq!(out.protected, 100);
        assert_eq!(out.protected_usd, 200.0);
    }
}

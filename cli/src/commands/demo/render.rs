use crate::branding::{terminal_palette, Palette};
use crate::model::StatusSnapshot;
use crate::render::{format_status, utilization_bar, utilization_line};

use super::summary::{compute_summary, SummaryInput, SummaryOutput};

pub struct DemoUi {
    palette: Palette,
    pub json: bool,
    mint_decimals: u8,
}

impl DemoUi {
    pub fn new(quiet: bool, json: bool, mint_decimals: u8) -> Self {
        Self {
            palette: terminal_palette(quiet || json),
            json,
            mint_decimals,
        }
    }

    pub fn println_preflight(&self, msg: &str) {
        if self.json {
            return;
        }
        println!("{}", self.palette.dim().apply_to(msg));
    }

    pub fn println_setup_ok(&self, msg: &str) {
        if self.json {
            return;
        }
        println!(
            "  {} {}",
            self.palette.ok().apply_to("✓"),
            msg
        );
    }

    pub fn println_act_header(&self, title: &str) {
        if self.json {
            return;
        }
        println!();
        println!("{}", self.palette.accent().apply_to(title));
    }

    pub fn println_withdraw_ok(&self, index: u32, total: u32, snap: &StatusSnapshot) {
        if self.json {
            return;
        }
        let bar = utilization_bar(snap.utilization_pct);
        println!(
            "  {} withdrawal {}/{}  [{}] {}",
            self.palette.ok().apply_to("✓"),
            index,
            total,
            bar,
            utilization_line(snap)
        );
    }

    pub fn println_trip_detected(&self, message: &str, snap: &StatusSnapshot) {
        if self.json {
            return;
        }
        println!();
        println!(
            "{}",
            self.palette
                .tripped()
                .apply_to("TRIPPED — velocity threshold exceeded")
        );
        println!("  {}", message);
        println!();
        print!("{}", format_status(snap, &self.palette));
    }

    pub fn println_blocked_attempt(&self, index: u32, message: &str) {
        if self.json {
            return;
        }
        println!(
            "  {} blocked attempt {}/2 — {}",
            self.palette.tripped().apply_to("✗"),
            index,
            message
        );
    }

    pub fn println_summary(
        &self,
        input: SummaryInput,
        attack_attempts_before_trip: u32,
        blocked_attempts_after_trip: u32,
    ) {
        if self.json {
            return;
        }

        let summary = compute_summary(input);
        println!();
        println!("{}", self.palette.brand().apply_to("Act III — summary"));
        print_summary_card(
            &self.palette,
            &summary,
            input.threshold,
            self.mint_decimals,
            input.unit_price,
            attack_attempts_before_trip,
            blocked_attempts_after_trip,
        );
    }
}

pub fn print_summary_card(
    palette: &Palette,
    summary: &SummaryOutput,
    threshold: u64,
    mint_decimals: u8,
    unit_price: f64,
    attack_attempts_before_trip: u32,
    blocked_attempts_after_trip: u32,
) {
    let extracted_ui = format_ui_amount(summary.attack_extracted, mint_decimals);
    let protected_ui = format_ui_amount(summary.protected, mint_decimals);
    let outflow_ui = format_ui_amount(summary.total_window_outflow, mint_decimals);
    let cap_ui = format_ui_amount(threshold, mint_decimals);

    println!();
    println!(
        "  {} Attacker extracted {} tokens ({}% of initial vault)",
        palette.tripped().apply_to("▸"),
        extracted_ui,
        summary.attack_extracted_pct_of_initial
    );
    println!(
        "  {} Total window outflow {} tokens (= {} cap)",
        palette.dim().apply_to("▸"),
        outflow_ui,
        cap_ui
    );
    println!(
        "  {} Protected {} tokens (~${:.2} at ${unit_price}/token)",
        palette.ok().apply_to("▸"),
        protected_ui,
        summary.protected_usd
    );
    println!(
        "  {} Trip after {} attack attempt(s); {} blocked post-trip",
        palette.dim().apply_to("▸"),
        attack_attempts_before_trip,
        blocked_attempts_after_trip
    );
}

pub fn format_ui_amount(amount: u64, decimals: u8) -> String {
    if decimals == 0 {
        return amount.to_string();
    }
    let scale = 10u64.pow(u32::from(decimals));
    let whole = amount / scale;
    let frac = amount % scale;
    let frac_str = format!("{:0width$}", frac, width = decimals as usize);
    let trimmed = frac_str.trim_end_matches('0');
    if trimmed.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{trimmed}")
    }
}

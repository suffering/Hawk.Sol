use crate::branding::Palette;
use crate::model::StatusSnapshot;
use circuit_breaker::state::{BreakerState, ThresholdMode};

use super::bar::{utilization_bar, EMPTY, FILLED};
use super::format::format_duration;
use super::format::format_tokens;

/// Uppercase state badge label (no styling).
pub fn state_badge_label(snapshot: &StatusSnapshot) -> &'static str {
    match snapshot.state {
        BreakerState::Active => "ACTIVE",
        BreakerState::Tripped => "TRIPPED",
        BreakerState::Paused => "PAUSED",
    }
}

/// Human-readable utilization line (threshold disabled uses dedicated copy).
pub fn utilization_line(snapshot: &StatusSnapshot) -> String {
    if snapshot.threshold_disabled {
        if snapshot.window_used == 0 {
            return "No outflow recorded in window (cap disabled)".to_string();
        }
        return format!(
            "{} used in window (cap disabled)",
            format_tokens(snapshot.window_used)
        );
    }

    format!(
        "{} / {} ({}% of window cap)",
        format_tokens(snapshot.window_used),
        format_tokens(snapshot.threshold),
        snapshot.utilization_pct
    )
}

pub fn format_status(snapshot: &StatusSnapshot, palette: &Palette) -> String {
    let mut out = String::new();

    let badge = state_badge_label(snapshot);
    let badge_styled = match snapshot.state {
        BreakerState::Active => palette.active().apply_to(badge).to_string(),
        BreakerState::Tripped => palette.tripped().apply_to(badge).to_string(),
        BreakerState::Paused => palette.paused().apply_to(badge).to_string(),
    };

    let bar = utilization_bar(snapshot.utilization_pct);
    let util_line = utilization_line(snapshot);

    out.push_str(&format!("Vault       {}\n", snapshot.vault));
    out.push_str(&format!("Breaker     {}\n", snapshot.breaker_config));
    out.push_str(&format!("State       {badge_styled}\n"));
    out.push_str(&format!("Utilization [{bar}] {util_line}\n"));
    out.push_str(&format!(
        "Mode        {}\n",
        threshold_mode_line(snapshot)
    ));
    out.push_str(&format!(
        "Bucket      rolls in {}\n",
        format_duration(snapshot.bucket_roll_in_secs)
    ));

    if snapshot.state == BreakerState::Tripped {
        out.push_str(&format!(
            "Tripped at  unix {}\n",
            snapshot.tripped_at
        ));
        if snapshot.resumable {
            out.push_str(&palette.ok().apply_to("Cooldown    elapsed — resumable now").to_string());
            out.push('\n');
        } else {
            out.push_str(&format!(
                "Cooldown    {} remaining\n",
                format_duration(snapshot.cooldown_remaining_secs)
            ));
        }
    }

    out.push_str(
        &palette
            .dim()
            .apply_to(&format!("On-chain    unix {}", snapshot.unix_timestamp))
            .to_string(),
    );
    out.push('\n');

    out
}

pub fn print(snapshot: &StatusSnapshot, palette: &Palette) -> anyhow::Result<()> {
    print!("{}", format_status(snapshot, palette));
    Ok(())
}

fn threshold_mode_line(snapshot: &StatusSnapshot) -> String {
    match snapshot.threshold_mode {
        ThresholdMode::Absolute => format!(
            "absolute cap {}",
            format_tokens(snapshot.max_absolute)
        ),
        ThresholdMode::PctOfBalance => format!("{} bps of vault balance", snapshot.max_bps),
        ThresholdMode::Max => format!(
            "max(absolute {}, {} bps of balance)",
            format_tokens(snapshot.max_absolute),
            snapshot.max_bps
        ),
    }
}

/// Exposed for fixture tests: bar characters used in render output.
pub const BAR_FILLED: char = FILLED;
pub const BAR_EMPTY: char = EMPTY;

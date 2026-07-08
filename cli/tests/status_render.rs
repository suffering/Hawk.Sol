//! Fixture-based render tests — no validator required.

use circuit_breaker::state::{BreakerState, ThresholdMode};
use solana_sdk::pubkey::Pubkey;
use solhawk::branding::Palette;
use solhawk::model::{bar_fill_count, StatusSnapshot, UTILIZATION_BAR_WIDTH};
use solhawk::render::{
    bar::utilization_bar, format_status, state_badge_label, to_json_value, BAR_EMPTY, BAR_FILLED,
};

fn base_snapshot() -> StatusSnapshot {
    StatusSnapshot {
        vault: Pubkey::new_unique(),
        breaker_config: Pubkey::new_unique(),
        window_state: Pubkey::new_unique(),
        unix_timestamp: 1_700_000_000,
        vault_balance: 10_000,
        state: BreakerState::Active,
        threshold_mode: ThresholdMode::Absolute,
        max_absolute: 1_000,
        max_bps: 500,
        window_seconds: 3_600,
        cooldown_seconds: 300,
        auto_recover: false,
        tripped_at: 0,
        threshold: 1_000,
        threshold_disabled: false,
        window_used: 0,
        utilization_pct: 0,
        bucket_roll_in_secs: 240,
        cooldown_remaining_secs: 0,
        resumable: false,
    }
}

fn count_bar_fill(bar: &str) -> usize {
    bar.chars().filter(|&c| c == BAR_FILLED).count()
}

fn extract_utilization_bar(rendered: &str) -> String {
    let line = rendered
        .lines()
        .find(|l| l.contains("Utilization"))
        .expect("utilization line");
    let start = line.find('[').expect("[") + 1;
    let end = line.find(']').expect("]");
    line[start..end].to_string()
}

#[test]
fn fixture_active_empty() {
    let snap = base_snapshot();
    assert_eq!(snap.utilization_pct, 0);
    assert_eq!(count_bar_fill(&utilization_bar(0)), 0);

    let text = format_status(&snap, &Palette::new(false));
    assert_eq!(state_badge_label(&snap), "ACTIVE");
    assert!(text.contains("ACTIVE"));
    assert_eq!(extract_utilization_bar(&text).chars().filter(|c| *c == BAR_FILLED).count(), 0);
}

#[test]
fn fixture_active_half() {
    let mut snap = base_snapshot();
    snap.window_used = 500;
    snap.utilization_pct = 50;

    let bar = utilization_bar(snap.utilization_pct);
    assert_eq!(count_bar_fill(&bar), 5);

    let text = format_status(&snap, &Palette::new(false));
    assert!(text.contains("500 / 1000 (50% of window cap)"));
    assert_eq!(extract_utilization_bar(&text).chars().filter(|c| *c == BAR_FILLED).count(), 5);
}

#[test]
fn fixture_active_full() {
    let mut snap = base_snapshot();
    snap.window_used = 1_000;
    snap.utilization_pct = 100;

    let bar = utilization_bar(snap.utilization_pct);
    assert_eq!(count_bar_fill(&bar), UTILIZATION_BAR_WIDTH);

    let text = format_status(&snap, &Palette::new(false));
    assert_eq!(extract_utilization_bar(&text).chars().filter(|c| *c == BAR_FILLED).count(), 10);
}

#[test]
fn fixture_tripped_cooldown_47pct_floor_rounding() {
    let mut snap = base_snapshot();
    snap.state = BreakerState::Tripped;
    snap.tripped_at = 1_699_999_880;
    snap.window_used = 470;
    snap.utilization_pct = 47;
    snap.cooldown_remaining_secs = 120;
    snap.resumable = false;

    // Floor: 47 * 10 / 100 = 4 filled segments (not 5).
    assert_eq!(bar_fill_count(47, UTILIZATION_BAR_WIDTH), 4);
    assert_eq!(count_bar_fill(&utilization_bar(47)), 4);

    let text = format_status(&snap, &Palette::new(false));
    assert_eq!(state_badge_label(&snap), "TRIPPED");
    assert!(text.contains("TRIPPED"));
    assert!(text.contains("2m 0s remaining"));
    assert!(!text.contains("resumable now"));
    assert_eq!(extract_utilization_bar(&text).chars().filter(|c| *c == BAR_FILLED).count(), 4);
    assert_eq!(
        extract_utilization_bar(&text).chars().filter(|c| *c == BAR_EMPTY).count(),
        6
    );
}

#[test]
fn fixture_tripped_resumable() {
    let mut snap = base_snapshot();
    snap.state = BreakerState::Tripped;
    snap.tripped_at = 1_699_999_000;
    snap.window_used = 1_000;
    snap.utilization_pct = 100;
    snap.cooldown_remaining_secs = 0;
    snap.resumable = true;

    let text = format_status(&snap, &Palette::new(false));
    assert!(text.contains("resumable now"));
    assert!(!text.contains("remaining"));
}

#[test]
fn fixture_threshold_disabled_message() {
    let mut snap = base_snapshot();
    snap.threshold = 0;
    snap.threshold_disabled = true;
    snap.window_used = 0;
    snap.utilization_pct = 0;

    let text = format_status(&snap, &Palette::new(false));
    assert!(text.contains("cap disabled"));
    assert!(!text.contains("0 / 0"));
}

#[test]
fn json_contract_field_names_and_round_trip() {
    let mut snap = base_snapshot();
    snap.state = BreakerState::Tripped;
    snap.utilization_pct = 47;
    snap.cooldown_remaining_secs = 120;
    snap.resumable = false;

    let json = to_json_value(&snap);
    let serialized = serde_json::to_string(&json).unwrap();
    let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    let expected_keys = [
        "vault",
        "breaker_config",
        "window_state",
        "state",
        "unix_timestamp",
        "vault_balance",
        "threshold_mode",
        "max_absolute",
        "max_bps",
        "window_seconds",
        "threshold",
        "threshold_disabled",
        "window_used",
        "utilization_pct",
        "bucket_roll_in_secs",
        "tripped_at",
        "cooldown_seconds",
        "cooldown_remaining_secs",
        "resumable",
        "auto_recover",
    ];

    for key in expected_keys {
        assert!(value.get(key).is_some(), "missing json field: {key}");
    }

    assert_eq!(value["state"], "tripped");
    assert_eq!(value["utilization_pct"], 47);
    assert_eq!(value["cooldown_remaining_secs"], 120);
    assert_eq!(value["resumable"], false);
}

#[test]
fn utilization_over_threshold_caps_display_pct() {
    let mut snap = base_snapshot();
    snap.window_used = 2_000;
    snap.threshold = 1_000;
    snap.utilization_pct = 100;

    let bar = utilization_bar(snap.utilization_pct);
    assert_eq!(count_bar_fill(&bar), UTILIZATION_BAR_WIDTH);
}

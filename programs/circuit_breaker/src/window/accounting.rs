use crate::state::{ThresholdMode, WindowState, BUCKET_COUNT};

/// Result of advancing the sliding window and checking an outflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowUpdateResult {
    pub window_sum: u64,
    pub threshold: u64,
    pub tripped: bool,
}

/// Compute the outflow threshold from live vault balance (no cached figures).
pub fn compute_threshold(
    mode: ThresholdMode,
    max_absolute: u64,
    max_bps: u16,
    vault_balance: u64,
) -> u64 {
    let pct_cap = vault_balance
        .checked_mul(max_bps as u64)
        .and_then(|v| v.checked_div(10_000))
        .unwrap_or(u64::MAX);

    match mode {
        ThresholdMode::Absolute => max_absolute,
        ThresholdMode::PctOfBalance => pct_cap,
        ThresholdMode::Max => max_absolute.max(pct_cap),
    }
}

/// Advance bucket-aligned window state and record outflow or signal trip.
///
/// Implements §6 of the architecture doc. Mutates `window` before any transfer CPI
/// would occur in the instruction handler.
pub fn advance_and_check(
    window: &mut WindowState,
    window_seconds: i64,
    now: i64,
    amount: u64,
    threshold: u64,
) -> WindowUpdateResult {
    let n = BUCKET_COUNT as i64;
    let bucket_duration = window_seconds / n;

    let elapsed = if now > window.last_bucket_ts {
        (now - window.last_bucket_ts) / bucket_duration
    } else {
        0
    };

    if elapsed >= n {
        window.buckets = [0; BUCKET_COUNT];
        window.current_idx = 0;
    } else if elapsed > 0 {
        let start_idx = window.current_idx as usize;
        for i in 1..=elapsed {
            let idx = (start_idx + i as usize) % BUCKET_COUNT;
            window.buckets[idx] = 0;
        }
        window.current_idx = ((start_idx as i64 + elapsed) % n) as u8;
    }

    window.last_bucket_ts += elapsed * bucket_duration;

    let window_sum: u64 = window.buckets.iter().sum();

    if window_sum.saturating_add(amount) > threshold {
        return WindowUpdateResult {
            window_sum,
            threshold,
            tripped: true,
        };
    }

    let idx = window.current_idx as usize;
    window.buckets[idx] = window.buckets[idx].saturating_add(amount);

    WindowUpdateResult {
        window_sum: window_sum.saturating_add(amount),
        threshold,
        tripped: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ThresholdMode;

    const WINDOW_SECONDS: i64 = 3600;
    const BUCKET_DURATION: i64 = WINDOW_SECONDS / BUCKET_COUNT as i64;

    fn fresh_window(start: i64) -> WindowState {
        let mut w = WindowState {
            buckets: [0; BUCKET_COUNT],
            current_idx: 0,
            last_bucket_ts: start,
        };
        w.initialize(start);
        w
    }

    #[test]
    fn threshold_boundary_exact_passes_one_trips() {
        let threshold = 1000u64;
        let mut window = fresh_window(0);

        let r = advance_and_check(&mut window, WINDOW_SECONDS, 0, 1000, threshold);
        assert!(!r.tripped);
        assert_eq!(r.window_sum, 1000);

        let mut window2 = fresh_window(0);
        let r2 = advance_and_check(&mut window2, WINDOW_SECONDS, 0, 1001, threshold);
        assert!(r2.tripped);
        assert_eq!(r2.window_sum, 0);
    }

    #[test]
    fn split_across_many_txs_accumulates_to_trip() {
        let threshold = 1000u64;
        let chunk = 100u64;
        let mut window = fresh_window(0);
        let mut tripped = false;

        for i in 0..15 {
            let r = advance_and_check(&mut window, WINDOW_SECONDS, 0, chunk, threshold);
            if r.tripped {
                tripped = true;
                assert_eq!(i, 10, "should trip on 11th withdrawal of 100");
                break;
            }
        }
        assert!(tripped);
    }

    #[test]
    fn bucket_boundary_drain_accumulates_across_advance() {
        let threshold = 10_000u64;
        let mut window = fresh_window(0);

        let r1 = advance_and_check(&mut window, WINDOW_SECONDS, 0, 400, threshold);
        assert!(!r1.tripped);

        let boundary_ts = BUCKET_DURATION;
        let r2 = advance_and_check(&mut window, WINDOW_SECONDS, boundary_ts, 400, threshold);
        assert!(!r2.tripped);
        assert_eq!(r2.window_sum, 800);

        let r3 = advance_and_check(
            &mut window,
            WINDOW_SECONDS,
            boundary_ts + 1,
            400,
            threshold,
        );
        assert!(!r3.tripped);
        assert_eq!(r3.window_sum, 1200);
    }

    #[test]
    fn stale_window_reset_clears_old_outflow() {
        let threshold = 500u64;
        let mut window = fresh_window(0);

        advance_and_check(&mut window, WINDOW_SECONDS, 0, 400, threshold);
        assert_eq!(window.buckets.iter().sum::<u64>(), 400);

        let stale_ts = WINDOW_SECONDS + BUCKET_DURATION;
        let r = advance_and_check(&mut window, WINDOW_SECONDS, stale_ts, 100, threshold);
        assert!(!r.tripped);
        assert_eq!(r.window_sum, 100);
        assert_eq!(window.buckets.iter().sum::<u64>(), 100);
    }

    #[test]
    fn last_bucket_ts_monotonic_on_time_rewind() {
        let mut window = fresh_window(1000);
        advance_and_check(&mut window, WINDOW_SECONDS, 2000, 100, 10_000);
        let ts_after_advance = window.last_bucket_ts;
        assert!(ts_after_advance > 1000);

        let sum_before = window.buckets.iter().sum::<u64>();
        let r = advance_and_check(&mut window, WINDOW_SECONDS, 500, 50, 10_000);
        assert!(!r.tripped);
        assert_eq!(window.last_bucket_ts, ts_after_advance);
        assert_eq!(window.buckets.iter().sum::<u64>(), sum_before + 50);
    }

    #[test]
    fn pct_of_balance_uses_live_balance() {
        let balance = 1_000_000u64;
        let bps = 1000u16; // 10%
        let threshold = compute_threshold(ThresholdMode::PctOfBalance, 0, bps, balance);
        assert_eq!(threshold, 100_000);

        let lower_balance = 500_000u64;
        let lower_threshold =
            compute_threshold(ThresholdMode::PctOfBalance, 0, bps, lower_balance);
        assert_eq!(lower_threshold, 50_000);
        assert!(lower_threshold < threshold);
    }

    #[test]
    fn max_mode_takes_greater_of_absolute_and_pct() {
        let balance = 1_000_000u64;
        let t = compute_threshold(ThresholdMode::Max, 200_000, 500, balance);
        assert_eq!(t, 200_000);

        let t2 = compute_threshold(ThresholdMode::Max, 10_000, 5000, balance);
        assert_eq!(t2, 500_000);
    }

    #[test]
    fn window_outflow_never_exceeds_threshold_before_trip() {
        let threshold = 777u64;
        let mut window = fresh_window(10_000);
        let mut now = 10_000i64;

        for step in 0..50 {
            let amount = (step % 7 + 1) as u64 * 11;
            now += (step % 3) as i64 * (BUCKET_DURATION / 2).max(1);
            let r = advance_and_check(&mut window, WINDOW_SECONDS, now, amount, threshold);
            if r.tripped {
                assert!(r.window_sum.saturating_add(amount) > threshold);
                return;
            }
            assert!(r.window_sum <= threshold);
        }
    }
}

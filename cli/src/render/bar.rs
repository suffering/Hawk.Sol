use crate::model::{bar_fill_count, UTILIZATION_BAR_WIDTH};

pub const FILLED: char = '▓';
pub const EMPTY: char = '░';

/// Build the utilization bar string. Fill count uses floor rounding via
/// [`bar_fill_count`].
pub fn utilization_bar(utilization_pct: u8) -> String {
    let filled = bar_fill_count(utilization_pct, UTILIZATION_BAR_WIDTH);
    let mut bar = String::with_capacity(UTILIZATION_BAR_WIDTH);
    for _ in 0..filled {
        bar.push(FILLED);
    }
    for _ in filled..UTILIZATION_BAR_WIDTH {
        bar.push(EMPTY);
    }
    bar
}

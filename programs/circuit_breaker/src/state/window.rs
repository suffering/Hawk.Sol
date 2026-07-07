use anchor_lang::prelude::*;

pub const BUCKET_COUNT: usize = 12;

#[account]
#[derive(InitSpace)]
pub struct WindowState {
    pub buckets: [u64; BUCKET_COUNT],
    pub current_idx: u8,
    pub last_bucket_ts: i64,
}

impl WindowState {
    pub fn initialize(&mut self, now: i64) {
        self.buckets = [0; BUCKET_COUNT];
        self.current_idx = 0;
        self.last_bucket_ts = now;
    }
}

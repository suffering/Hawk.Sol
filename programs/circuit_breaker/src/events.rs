use anchor_lang::prelude::*;

#[event]
pub struct Withdraw {
    pub amount: u64,
    pub destination: Pubkey,
    pub window_sum: u64,
    pub threshold: u64,
}

#[event]
pub struct Tripped {
    pub tripped_at: i64,
    pub window_sum: u64,
    pub attempted_amount: u64,
    pub threshold: u64,
    pub manual: bool,
}

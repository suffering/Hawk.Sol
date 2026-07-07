use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use crate::cli::GlobalOpts;
use crate::error::confirm_action;

pub fn run(global: &GlobalOpts, vault: Pubkey, yes: bool) -> Result<()> {
    confirm_action(&format!("Resume breaker for vault {vault}?"), yes)?;
    let _ = global;
    crate::commands::not_implemented("resume")
}

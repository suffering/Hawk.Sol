use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use crate::cli::GlobalOpts;

pub fn run(_global: &GlobalOpts, _vault: Pubkey, _json: bool) -> Result<()> {
    crate::commands::not_implemented("status")
}

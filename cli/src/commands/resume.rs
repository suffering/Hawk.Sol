use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::branding::terminal_palette;
use crate::cli::GlobalOpts;
use crate::commands::tx_output::print_confirmed_status;
use crate::error::confirm_action;
use crate::ix::build_resume_ix;
use crate::rpc::{
    authority::check_resume_authority, execute_transaction, fetch_breaker_config, keypair::load_keypair,
};

pub fn run(global: &GlobalOpts, vault: Pubkey, yes: bool) -> Result<()> {
    if !confirm_action(&format!("Resume breaker for vault {vault}?"), yes)? {
        eprintln!("cancelled");
        return Ok(());
    }

    let signer = load_keypair(global.keypair.as_deref())?;
    let rpc = RpcClient::new(global.rpc_url.clone());

    let config = fetch_breaker_config(&rpc, &vault)?;
    check_resume_authority(&signer.pubkey(), &config)?;

    let ix = build_resume_ix(&vault, &signer.pubkey());
    let outcome = execute_transaction(&rpc, &[ix], &[&signer])?;

    let palette = terminal_palette(global.quiet);
    print_confirmed_status(&rpc, &vault, &outcome.signature, &palette)
}

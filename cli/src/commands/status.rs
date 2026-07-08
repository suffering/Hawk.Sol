use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::branding::terminal_palette;
use crate::cli::GlobalOpts;
use crate::render;
use crate::rpc::fetch_status_snapshot;

pub fn run(global: &GlobalOpts, vault: Pubkey, json: bool) -> Result<()> {
    let rpc = RpcClient::new(global.rpc_url.clone());
    let snapshot = fetch_status_snapshot(&rpc, &vault)?;

    if json {
        render::json::print(&snapshot)?;
    } else {
        let palette = terminal_palette(global.quiet);
        render::status::print(&snapshot, &palette)?;
    }

    Ok(())
}

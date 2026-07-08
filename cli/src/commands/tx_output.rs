use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::branding::Palette;
use crate::render;
use crate::rpc::fetch_status_snapshot;

pub fn print_confirmed_status(
    rpc: &RpcClient,
    vault: &Pubkey,
    signature: &Signature,
    palette: &Palette,
) -> Result<()> {
    println!("Transaction confirmed: {signature}");
    let snapshot = fetch_status_snapshot(rpc, vault)?;
    render::status::print(&snapshot, palette)?;
    Ok(())
}

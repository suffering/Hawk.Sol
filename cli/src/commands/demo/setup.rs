use anyhow::{Context, Result};
use circuit_breaker::state::ThresholdMode;
use circuit_breaker::InitializeBreakerParams;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
};
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::{Account as TokenAccountState, Mint as MintState};

use crate::cli::DemoArgs;
use crate::ix::{build_initialize_ix, build_guarded_withdraw_ix, GuardedWithdrawAccounts, InitIxAccounts};
use crate::rpc::{decode::decode_token_account_amount, execute_transaction};

use super::keypairs::{DemoKeypairs, DemoTokenKeypairs};
use super::render::DemoUi;

pub struct DemoChain {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub safe_destination: Pubkey,
    pub drain_destination: Pubkey,
    pub vault_initial_balance: u64,
}

pub fn setup_chain(
    rpc: &RpcClient,
    keys: &DemoKeypairs,
    token_kps: &DemoTokenKeypairs,
    args: &DemoArgs,
    ui: &DemoUi,
) -> Result<DemoChain> {
    airdrop_actors(rpc, keys)?;

    let mint = token_kps.mint.pubkey();
    let vault = token_kps.vault.pubkey();
    let safe_destination = token_kps.safe.pubkey();
    let drain_destination = token_kps.drain.pubkey();

    create_mint(
        rpc,
        &keys.payer,
        &token_kps.mint,
        &keys.vault_authority.pubkey(),
        args.mint_decimals,
    )?;
    create_token_account(rpc, &keys.payer, &token_kps.vault, &mint, &keys.vault_authority.pubkey())?;
    create_token_account(rpc, &keys.payer, &token_kps.safe, &mint, &keys.guardian.pubkey())?;
    create_token_account(
        rpc,
        &keys.payer,
        &token_kps.drain,
        &mint,
        &keys.drain_owner.pubkey(),
    )?;

    mint_to(
        rpc,
        &keys.payer,
        &keys.vault_authority,
        &mint,
        &vault,
        args.vault_amount,
    )?;

    let params = InitializeBreakerParams {
        guardian: keys.guardian.pubkey(),
        operator: keys.operator.pubkey(),
        window_seconds: args.window_seconds,
        threshold_mode: ThresholdMode::Absolute,
        max_absolute: args.threshold,
        max_bps: 0,
        cooldown_seconds: args.cooldown_seconds,
        timelock_floor: 86_400,
        auto_recover: false,
        safe_destination,
    };

    let init_ix = build_initialize_ix(&InitIxAccounts {
        payer: keys.payer.pubkey(),
        vault_authority: keys.vault_authority.pubkey(),
        mint,
        vault,
        params,
    });

    execute_transaction(
        rpc,
        &[init_ix],
        &[&keys.payer, &keys.vault_authority],
    )?;

    ui.println_setup_ok("mint + vault funded");
    ui.println_setup_ok("breaker initialized (absolute cap)");

    Ok(DemoChain {
        mint,
        vault,
        safe_destination,
        drain_destination,
        vault_initial_balance: args.vault_amount,
    })
}

pub fn token_balance(rpc: &RpcClient, account: &Pubkey) -> Result<u64> {
    let data = rpc
        .get_account_data(account)
        .with_context(|| format!("fetch token account {account}"))?;
    decode_token_account_amount(&data)
}

fn airdrop_actors(rpc: &RpcClient, keys: &DemoKeypairs) -> Result<()> {
    let lamports = 10_000_000_000u64;
    for kp in [
        &keys.payer,
        &keys.guardian,
        &keys.operator,
        &keys.vault_authority,
        &keys.drain_owner,
    ] {
        let sig = rpc
            .request_airdrop(&kp.pubkey(), lamports)
            .with_context(|| format!("airdrop {}", kp.pubkey()))?;
        rpc.confirm_transaction(&sig)
            .with_context(|| format!("confirm airdrop {}", kp.pubkey()))?;
    }
    Ok(())
}

fn create_mint(
    rpc: &RpcClient,
    payer: &Keypair,
    mint_kp: &Keypair,
    mint_authority: &Pubkey,
    decimals: u8,
) -> Result<()> {
    let rent = rpc.get_minimum_balance_for_rent_exemption(MintState::LEN)?;
    let create = system_instruction::create_account(
        &payer.pubkey(),
        &mint_kp.pubkey(),
        rent,
        MintState::LEN as u64,
        &spl_token::ID,
    );
    let init = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &mint_kp.pubkey(),
        mint_authority,
        None,
        decimals,
    )?;
    execute_transaction(rpc, &[create, init], &[payer, mint_kp])?;
    Ok(())
}

fn create_token_account(
    rpc: &RpcClient,
    payer: &Keypair,
    account_kp: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<()> {
    let rent = rpc.get_minimum_balance_for_rent_exemption(TokenAccountState::LEN)?;
    let create = system_instruction::create_account(
        &payer.pubkey(),
        &account_kp.pubkey(),
        rent,
        TokenAccountState::LEN as u64,
        &spl_token::ID,
    );
    let init = spl_token::instruction::initialize_account(
        &spl_token::ID,
        &account_kp.pubkey(),
        mint,
        owner,
    )?;
    execute_transaction(rpc, &[create, init], &[payer, account_kp])?;
    Ok(())
}

fn mint_to(
    rpc: &RpcClient,
    payer: &Keypair,
    mint_authority: &Keypair,
    mint: &Pubkey,
    destination: &Pubkey,
    amount: u64,
) -> Result<()> {
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        mint,
        destination,
        &mint_authority.pubkey(),
        &[],
        amount,
    )?;
    execute_transaction(rpc, &[ix], &[payer, mint_authority])?;
    Ok(())
}

pub fn build_withdraw_ix(
    operator: Pubkey,
    vault: Pubkey,
    destination: Pubkey,
    amount: u64,
) -> solana_sdk::instruction::Instruction {
    build_guarded_withdraw_ix(
        &GuardedWithdrawAccounts {
            operator,
            vault,
            destination,
        },
        amount,
    )
}

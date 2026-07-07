use clap::{Parser, Subcommand, ValueEnum};
use solana_sdk::pubkey::Pubkey;

use crate::config::DEFAULT_RPC_URL;

#[derive(Parser)]
#[command(
    name = "solhawk",
    about = "SolHawk — Solana Circuit Breaker Protocol CLI",
    long_version = concat!(
        env!("CARGO_PKG_VERSION"),
        "\nhttps://github.com/SolHawk/solhawk"
    ),
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
pub struct GlobalOpts {
    /// Solana JSON-RPC endpoint
    #[arg(long, global = true, default_value = DEFAULT_RPC_URL, env = "SOLHAWK_RPC_URL")]
    pub rpc_url: String,

    /// Path to signer keypair (JSON array)
    #[arg(long, global = true, env = "SOLHAWK_KEYPAIR")]
    pub keypair: Option<String>,

    /// Default vault token account for subcommands
    #[arg(long, global = true, env = "SOLHAWK_VAULT")]
    pub vault: Option<Pubkey>,

    /// Suppress banner and color (also auto-disabled when stdout is not a TTY)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show breaker state and window utilization for a vault
    Status {
        /// Protected SPL token vault account
        vault: Pubkey,
        /// Machine-readable JSON output (no colors or banner)
        #[arg(long)]
        json: bool,
    },

    /// Initialize a circuit breaker on a vault
    Init(InitArgs),

    /// Guardian emergency stop
    Trip {
        vault: Pubkey,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },

    /// Resume after cooldown (guardian or auto_recover)
    Resume {
        vault: Pubkey,
        #[arg(long)]
        yes: bool,
    },

    /// §11 proof-of-efficacy demo on localnet.
    ///
    /// Requires a local validator (e.g. `solana-test-validator --reset`) with the
    /// circuit_breaker program deployed (`anchor deploy`).
    Demo(DemoArgs),
}

#[derive(Parser)]
pub struct InitArgs {
    /// Vault SPL token account to protect
    pub vault: Pubkey,
    /// Token mint
    #[arg(long)]
    pub mint: Pubkey,
    /// Guardian pubkey
    #[arg(long)]
    pub guardian: Option<Pubkey>,
    /// Operator pubkey
    #[arg(long)]
    pub operator: Option<Pubkey>,
    /// Immutable safe destination
    #[arg(long)]
    pub safe_destination: Option<Pubkey>,
    #[arg(long, default_value = "3600")]
    pub window_seconds: i64,
    #[arg(long, value_enum, default_value = "max")]
    pub threshold_mode: ThresholdModeArg,
    #[arg(long, default_value = "100000000")]
    pub max_absolute: u64,
    #[arg(long, default_value = "1000")]
    pub max_bps: u16,
    #[arg(long, default_value = "300")]
    pub cooldown_seconds: i64,
    #[arg(long, default_value = "86400")]
    pub timelock_floor: i64,
    #[arg(long, default_value = "false")]
    pub auto_recover: bool,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ThresholdModeArg {
    Absolute,
    PctOfBalance,
    Max,
}

#[derive(Parser)]
pub struct DemoArgs {
    /// Spawn `solana-test-validator` if RPC is unreachable
    #[arg(long)]
    pub spawn_validator: bool,

    /// Initial vault balance (base units)
    #[arg(long, default_value = "1000000000")]
    pub vault_amount: u64,

    /// Per-window outflow cap (base units)
    #[arg(long, default_value = "100000000")]
    pub threshold: u64,

    /// Number of normal withdrawals before attack simulation
    #[arg(long, default_value = "5")]
    pub normal_count: u32,

    /// Amount per normal withdrawal
    #[arg(long, default_value = "10000000")]
    pub normal_amount: u64,

    /// Amount per rapid-drain attempt
    #[arg(long, default_value = "50000000")]
    pub attack_chunk: u64,

    #[arg(long, default_value = "3600")]
    pub window_seconds: i64,

    #[arg(long, default_value = "0")]
    pub cooldown_seconds: i64,

    /// Deterministic keypair seed for reproducible demo output
    #[arg(long, default_value = "42")]
    pub seed: u64,

    #[arg(long, default_value = "6")]
    pub mint_decimals: u8,

    /// Display price per token for summary (USD)
    #[arg(long, default_value = "1.0")]
    pub unit_price: f64,
}

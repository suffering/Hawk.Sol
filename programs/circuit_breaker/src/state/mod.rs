pub mod config;
pub mod window;

pub use config::*;
pub use window::*;

/// PDA seed for `BreakerConfig`.
pub const BREAKER_CONFIG_SEED: &[u8] = b"breaker";
/// PDA seed for `WindowState`.
pub const WINDOW_STATE_SEED: &[u8] = b"window";
/// PDA seed for the vault authority signer.
pub const AUTHORITY_SEED: &[u8] = b"authority";

/// Phase 2: PDA seed for `PendingConfigChange`.
pub const PENDING_CONFIG_SEED: &[u8] = b"pending_config";

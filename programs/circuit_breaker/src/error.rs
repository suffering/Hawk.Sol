use anchor_lang::prelude::*;

#[error_code]
pub enum BreakerError {
    #[msg("Breaker is tripped; withdrawals are blocked")]
    Tripped,
    #[msg("Breaker is paused; withdrawals are blocked")]
    Paused,
    #[msg("Outflow velocity threshold exceeded; breaker tripped")]
    VelocityTripped,
    #[msg("Only the configured operator may call guarded_withdraw")]
    UnauthorizedOperator,
    #[msg("Only the configured guardian may call this instruction")]
    UnauthorizedGuardian,
    #[msg("Vault mint does not match configured token_mint")]
    MintMismatch,
    #[msg("window_seconds must be positive and divisible by bucket count")]
    InvalidWindowSeconds,
    #[msg("max_bps must be <= 10_000")]
    InvalidMaxBps,
    #[msg("Destination account does not match instruction argument")]
    DestinationMismatch,
    #[msg("Cooldown period has not elapsed since trip")]
    CooldownNotElapsed,
    #[msg("Breaker must be in Tripped state to resume")]
    NotTripped,
    #[msg("Guardian signature required when auto_recover is disabled")]
    GuardianRequired,
    #[msg("A config change is already pending for this breaker")]
    PendingConfigExists,
    #[msg("Timelock has not elapsed; execute_config_change is not yet allowed")]
    TimelockNotElapsed,
    #[msg("No pending config change exists for this breaker")]
    NoPendingConfig,
    #[msg("requested_delay must be non-negative")]
    InvalidRequestedDelay,
    #[msg("emergency_route_to_safe requires the breaker to be Tripped")]
    EmergencyRouteRequiresTripped,
}

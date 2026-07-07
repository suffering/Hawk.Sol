use std::io::Write;

use anyhow::{bail, Context, Result};
use circuit_breaker::error::BreakerError;

/// Map Anchor custom error codes (6000 + variant index) to human-readable messages.
pub fn format_breaker_error_code(code: u32) -> Option<&'static str> {
    match code {
        6000 => Some("Breaker is tripped; withdrawals are blocked"),
        6001 => Some("Breaker is paused; withdrawals are blocked"),
        6002 => Some("Outflow velocity threshold exceeded; breaker tripped"),
        6003 => Some("Only the configured operator may call guarded_withdraw"),
        6004 => Some("Only the configured guardian may call this instruction"),
        6005 => Some("Vault mint does not match configured token_mint"),
        6006 => Some("window_seconds must be positive and divisible by bucket count"),
        6007 => Some("max_bps must be <= 10_000"),
        6008 => Some("Destination account does not match instruction argument"),
        6009 => Some("Cooldown period has not elapsed since trip"),
        6010 => Some("Breaker must be in Tripped state to resume"),
        6011 => Some("Guardian signature required when auto_recover is disabled"),
        _ => None,
    }
}

pub fn breaker_error_from_code(code: u32) -> Option<BreakerError> {
    match code {
        6000 => Some(BreakerError::Tripped),
        6001 => Some(BreakerError::Paused),
        6002 => Some(BreakerError::VelocityTripped),
        6003 => Some(BreakerError::UnauthorizedOperator),
        6004 => Some(BreakerError::UnauthorizedGuardian),
        6005 => Some(BreakerError::MintMismatch),
        6006 => Some(BreakerError::InvalidWindowSeconds),
        6007 => Some(BreakerError::InvalidMaxBps),
        6008 => Some(BreakerError::DestinationMismatch),
        6009 => Some(BreakerError::CooldownNotElapsed),
        6010 => Some(BreakerError::NotTripped),
        6011 => Some(BreakerError::GuardianRequired),
        _ => None,
    }
}

pub fn confirm_action(prompt: &str, yes: bool) -> Result<()> {
    if yes {
        return Ok(());
    }

    print!("{prompt} [y/N] ");
    std::io::stdout().flush().context("flush stdout")?;

    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;

    if line.trim().eq_ignore_ascii_case("y") {
        Ok(())
    } else {
        bail!("cancelled")
    }
}

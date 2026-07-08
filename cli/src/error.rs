use std::io::Write;

use anyhow::{Context, Result};
use circuit_breaker::error::BreakerError;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::{RpcError, RpcResponseErrorData};
use solana_sdk::instruction::InstructionError;
use solana_sdk::transaction::TransactionError;

/// Returns `Ok(true)` to proceed, `Ok(false)` if the user declined.
pub fn confirm_action(prompt: &str, yes: bool) -> Result<bool> {
    if yes {
        return Ok(true);
    }

    print!("{prompt} [y/N] ");
    std::io::stdout().flush().context("flush stdout")?;

    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;

    Ok(line.trim().eq_ignore_ascii_case("y"))
}

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
        6012 => Some("A config change is already pending for this breaker"),
        6013 => Some("Timelock has not elapsed; execute_config_change is not yet allowed"),
        6014 => Some("No pending config change exists for this breaker"),
        6015 => Some("requested_delay must be non-negative"),
        6016 => Some("emergency_route_to_safe requires the breaker to be Tripped"),
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
        6012 => Some(BreakerError::PendingConfigExists),
        6013 => Some(BreakerError::TimelockNotElapsed),
        6014 => Some(BreakerError::NoPendingConfig),
        6015 => Some(BreakerError::InvalidRequestedDelay),
        6016 => Some(BreakerError::EmergencyRouteRequiresTripped),
        _ => None,
    }
}

/// Extract a circuit-breaker custom program error code from a transaction error.
pub fn extract_breaker_error_code(err: &TransactionError) -> Option<u32> {
    match err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => Some(*code),
        _ => None,
    }
}

/// Format a [`TransactionError`] for user display (simulation or send path).
pub fn format_transaction_error(err: &TransactionError) -> String {
    if let Some(code) = extract_breaker_error_code(err) {
        if let Some(msg) = format_breaker_error_code(code) {
            return msg.to_string();
        }
        return format!("transaction failed: unknown program error code {code}");
    }
    format!("transaction failed: {err}")
}

/// Format `simulate_transaction` inner failure (`Ok` response with `value.err`).
pub fn format_simulation_error(err: &TransactionError) -> String {
    format_transaction_error(err)
}

/// Format a [`ClientError`] from RPC simulate/send paths.
pub fn format_client_error(err: &ClientError) -> String {
    match err.kind() {
        ClientErrorKind::TransactionError(te) => format_transaction_error(te),
        ClientErrorKind::RpcError(rpc_err) => format_rpc_error(rpc_err),
        other => format!("RPC request failed: {other}"),
    }
}

fn format_rpc_error(err: &RpcError) -> String {
    match err {
        RpcError::RpcResponseError { message, data, .. } => {
            if let RpcResponseErrorData::SendTransactionPreflightFailure(preflight) = data {
                if let Some(te) = &preflight.err {
                    return format_transaction_error(te);
                }
            }
            format!("RPC error: {message}")
        }
        other => format!("RPC error: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_breaker_error_code_maps_to_message() {
        let cases: &[(u32, &str)] = &[
            (6000, "Breaker is tripped; withdrawals are blocked"),
            (6001, "Breaker is paused; withdrawals are blocked"),
            (6002, "Outflow velocity threshold exceeded; breaker tripped"),
            (6003, "Only the configured operator may call guarded_withdraw"),
            (6004, "Only the configured guardian may call this instruction"),
            (6005, "Vault mint does not match configured token_mint"),
            (
                6006,
                "window_seconds must be positive and divisible by bucket count",
            ),
            (6007, "max_bps must be <= 10_000"),
            (
                6008,
                "Destination account does not match instruction argument",
            ),
            (6009, "Cooldown period has not elapsed since trip"),
            (6010, "Breaker must be in Tripped state to resume"),
            (
                6011,
                "Guardian signature required when auto_recover is disabled",
            ),
            (6012, "A config change is already pending for this breaker"),
            (
                6013,
                "Timelock has not elapsed; execute_config_change is not yet allowed",
            ),
            (6014, "No pending config change exists for this breaker"),
            (6015, "requested_delay must be non-negative"),
            (
                6016,
                "emergency_route_to_safe requires the breaker to be Tripped",
            ),
        ];

        for (code, expected) in cases {
            assert_eq!(format_breaker_error_code(*code), Some(*expected));
            assert_eq!(breaker_error_from_code(*code).is_some(), true);
        }
    }

    #[test]
    fn transaction_error_custom_code_maps_via_concrete_type() {
        let err = TransactionError::InstructionError(0, InstructionError::Custom(6009));
        assert_eq!(
            format_transaction_error(&err),
            "Cooldown period has not elapsed since trip"
        );
        assert_eq!(
            format_simulation_error(&err),
            "Cooldown period has not elapsed since trip"
        );
    }

    #[test]
    fn client_error_from_transaction_error_maps() {
        let te = TransactionError::InstructionError(0, InstructionError::Custom(6004));
        let client_err: ClientError = te.into();
        assert_eq!(
            format_client_error(&client_err),
            "Only the configured guardian may call this instruction"
        );
    }

    #[test]
    fn unknown_custom_code_falls_back_without_hex_leak() {
        let err = TransactionError::InstructionError(0, InstructionError::Custom(9999));
        let formatted = format_transaction_error(&err);
        assert!(!formatted.contains("0x"));
        assert!(formatted.contains("unknown program error code 9999"));
    }
}

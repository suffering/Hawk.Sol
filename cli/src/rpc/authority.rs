use anyhow::{bail, Result};
use circuit_breaker::state::BreakerConfig;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeAuthority {
    Guardian,
    Permissionless,
}

pub fn required_resume_authority(config: &BreakerConfig) -> ResumeAuthority {
    if config.auto_recover {
        ResumeAuthority::Permissionless
    } else {
        ResumeAuthority::Guardian
    }
}

pub fn check_trip_authority(signer: &Pubkey, config: &BreakerConfig) -> Result<()> {
    if signer == &config.guardian {
        Ok(())
    } else {
        bail!(
            "trip requires guardian {}, but --keypair is {}",
            config.guardian,
            signer
        )
    }
}

pub fn check_resume_authority(signer: &Pubkey, config: &BreakerConfig) -> Result<()> {
    match required_resume_authority(config) {
        ResumeAuthority::Permissionless => Ok(()),
        ResumeAuthority::Guardian if signer == &config.guardian => Ok(()),
        ResumeAuthority::Guardian => bail!(
            "resume requires guardian {}, but --keypair is {}",
            config.guardian,
            signer
        ),
    }
}

#[cfg(test)]
mod tests {
    use circuit_breaker::state::{BreakerState, ThresholdMode};

    use super::*;

    fn sample_config(guardian: Pubkey, auto_recover: bool) -> BreakerConfig {
        BreakerConfig {
            guardian,
            operator: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            token_mint: Pubkey::new_unique(),
            authority_bump: 255,
            window_seconds: 3600,
            threshold_mode: ThresholdMode::Max,
            max_absolute: 1,
            max_bps: 100,
            cooldown_seconds: 300,
            timelock_floor: 86400,
            auto_recover,
            safe_destination: Pubkey::new_unique(),
            state: BreakerState::Tripped,
            tripped_at: 0,
        }
    }

    #[test]
    fn trip_guardian_passes_wrong_key_fails_with_names() {
        let guardian = Pubkey::new_unique();
        let wrong = Pubkey::new_unique();
        let config = sample_config(guardian, false);

        check_trip_authority(&guardian, &config).unwrap();
        let err = check_trip_authority(&wrong, &config).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("trip requires guardian"));
        assert!(msg.contains(&guardian.to_string()));
        assert!(msg.contains(&wrong.to_string()));
    }

    #[test]
    fn resume_guardian_required_when_auto_recover_disabled() {
        let guardian = Pubkey::new_unique();
        let wrong = Pubkey::new_unique();
        let config = sample_config(guardian, false);

        check_resume_authority(&guardian, &config).unwrap();
        let err = check_resume_authority(&wrong, &config).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("resume requires guardian"));
        assert!(msg.contains(&guardian.to_string()));
    }

    #[test]
    fn resume_permissionless_when_auto_recover_enabled() {
        let guardian = Pubkey::new_unique();
        let anyone = Pubkey::new_unique();
        let config = sample_config(guardian, true);

        check_resume_authority(&anyone, &config).unwrap();
    }
}

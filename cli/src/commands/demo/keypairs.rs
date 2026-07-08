use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, SeedDerivable, Signer};

/// Deterministic demo actors derived from `--seed` (no `Keypair::new()`).
#[derive(Debug)]
pub struct DemoKeypairs {
    pub payer: Keypair,
    pub guardian: Keypair,
    pub operator: Keypair,
    pub vault_authority: Keypair,
    pub drain_owner: Keypair,
}

/// SPL mint + token accounts (also seed-derived).
#[derive(Debug)]
pub struct DemoTokenKeypairs {
    pub mint: Keypair,
    pub vault: Keypair,
    pub safe: Keypair,
    pub drain: Keypair,
}

impl DemoKeypairs {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            payer: derive_keypair(seed, b"solhawk-demo-payer"),
            guardian: derive_keypair(seed, b"solhawk-demo-guardian"),
            operator: derive_keypair(seed, b"solhawk-demo-operator"),
            vault_authority: derive_keypair(seed, b"solhawk-demo-vault-auth"),
            drain_owner: derive_keypair(seed, b"solhawk-demo-drain"),
        }
    }

    pub fn token_accounts_from_seed(seed: u64) -> DemoTokenKeypairs {
        DemoTokenKeypairs {
            mint: derive_keypair(seed, b"solhawk-demo-mint"),
            vault: derive_keypair(seed, b"solhawk-demo-vault"),
            safe: derive_keypair(seed, b"solhawk-demo-safe"),
            drain: derive_keypair(seed, b"solhawk-demo-drain-ta"),
        }
    }

    pub fn pubkeys(&self) -> DemoPubkeys {
        DemoPubkeys {
            payer: self.payer.pubkey(),
            guardian: self.guardian.pubkey(),
            operator: self.operator.pubkey(),
            vault_authority: self.vault_authority.pubkey(),
            drain_owner: self.drain_owner.pubkey(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoPubkeys {
    pub payer: Pubkey,
    pub guardian: Pubkey,
    pub operator: Pubkey,
    pub vault_authority: Pubkey,
    pub drain_owner: Pubkey,
}

fn derive_keypair(master_seed: u64, label: &[u8]) -> Keypair {
    let mut seed = [0u8; 32];
    seed[..8].copy_from_slice(&master_seed.to_le_bytes());
    let copy_len = label.len().min(24);
    seed[8..8 + copy_len].copy_from_slice(&label[..copy_len]);
    Keypair::from_seed(&seed).expect("deterministic demo seed produces valid keypair")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_pubkeys() {
        let a = DemoKeypairs::from_seed(42);
        let b = DemoKeypairs::from_seed(42);
        assert_eq!(a.pubkeys(), b.pubkeys());
    }

    #[test]
    fn different_seeds_produce_different_pubkeys() {
        let a = DemoKeypairs::from_seed(42);
        let b = DemoKeypairs::from_seed(43);
        assert_ne!(a.pubkeys().payer, b.pubkeys().payer);
        assert_ne!(a.pubkeys().operator, b.pubkeys().operator);
    }
}

use std::path::PathBuf;

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use circuit_breaker::accounts::{
    CancelConfigChange, EmergencyRouteToSafe, ExecuteConfigChange, GuardedWithdraw,
    InitializeBreaker, ProposeConfigChange, Resume, Trip,
};
use circuit_breaker::instruction::{
    CancelConfigChange as CancelConfigChangeIx, EmergencyRouteToSafe as EmergencyRouteToSafeIx,
    ExecuteConfigChange as ExecuteConfigChangeIx, GuardedWithdraw as GuardedWithdrawIx,
    InitializeBreaker as InitializeBreakerIx, ProposeConfigChange as ProposeConfigChangeIx,
    Resume as ResumeIx, Trip as TripIx,
};
use circuit_breaker::state::{
    AUTHORITY_SEED, BREAKER_CONFIG_SEED, PENDING_CONFIG_SEED, WINDOW_STATE_SEED, BreakerConfig,
    PendingConfigChange, WindowState,
};
use circuit_breaker::{InitializeBreakerParams, ProposedConfigParams, ID as PROGRAM_ID};
use litesvm::LiteSVM;
use solana_clock::Clock;
use solana_program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccountState, Mint as MintState};

pub struct TestKeypairs {
    pub payer: Keypair,
    pub vault_authority: Keypair,
    pub guardian: Keypair,
    pub operator: Keypair,
    pub safe_destination: Keypair,
}

impl TestKeypairs {
    pub fn new() -> Self {
        Self {
            payer: Keypair::new(),
            vault_authority: Keypair::new(),
            guardian: Keypair::new(),
            operator: Keypair::new(),
            safe_destination: Keypair::new(),
        }
    }
}

pub struct BreakerTestContext {
    pub svm: LiteSVM,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub destination: Pubkey,
}

impl BreakerTestContext {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new()
            .with_sysvars()
            .with_spl_programs()
            .with_sigverify(false)
            .with_blockhash_check(false)
            .with_transaction_history(0);

        let program_bytes = std::fs::read(program_so_path()).expect("program .so");
        svm.add_program(PROGRAM_ID, &program_bytes);

        Self {
            svm,
            mint: Pubkey::default(),
            vault: Pubkey::default(),
            destination: Pubkey::default(),
        }
    }

    pub fn setup_vault(&mut self, keys: &TestKeypairs, amount: u64) {
        for kp in [
            &keys.payer,
            &keys.vault_authority,
            &keys.guardian,
            &keys.operator,
            &keys.safe_destination,
        ] {
            self.airdrop(&kp.pubkey(), 10_000_000_000);
        }

        let mint_authority = keys.vault_authority.pubkey();
        self.mint = self.create_mint(&keys.payer, &mint_authority);
        let mint = self.mint;
        self.vault = self.create_token_account(&keys.payer, &mint, &mint_authority);
        let destination_owner = keys.safe_destination.pubkey();
        self.destination = self.create_token_account(&keys.payer, &mint, &destination_owner);
        let vault = self.vault;
        self.mint_to(&keys.payer, &keys.vault_authority, &vault, amount);
    }

    pub fn initialize_breaker(&mut self, keys: &TestKeypairs, params: InitializeBreakerParams) {
        let (breaker_config, _) = self.derive_breaker_config();
        let (window_state, _) = self.derive_window_state();
        let (breaker_authority, _) = self.derive_authority();

        let accounts = InitializeBreaker {
            payer: keys.payer.pubkey(),
            vault_authority: keys.vault_authority.pubkey(),
            token_mint: self.mint,
            vault: self.vault,
            breaker_config,
            window_state,
            breaker_authority,
            token_program: spl_token::ID,
            system_program: system_program::ID,
        };

        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: InitializeBreakerIx { params }.data(),
        };

        self.send(&[ix], &[&keys.payer, &keys.vault_authority])
            .expect("initialize_breaker");
    }

    pub fn guarded_withdraw(
        &mut self,
        keys: &TestKeypairs,
        amount: u64,
    ) -> Result<(), String> {
        let (breaker_config, _) = self.derive_breaker_config();
        let (window_state, _) = self.derive_window_state();
        let (breaker_authority, _) = self.derive_authority();

        let accounts = GuardedWithdraw {
            operator: keys.operator.pubkey(),
            breaker_config,
            window_state,
            vault: self.vault,
            destination: self.destination,
            breaker_authority,
            token_program: spl_token::ID,
        };

        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: GuardedWithdrawIx {
                amount,
                destination: self.destination,
            }
            .data(),
        };

        self.send(&[ix], &[&keys.operator]).map_err(|e| format!("{e:?}"))
    }

    pub fn trip(&mut self, keys: &TestKeypairs) -> Result<(), String> {
        let (breaker_config, _) = self.derive_breaker_config();
        let accounts = Trip {
            guardian: keys.guardian.pubkey(),
            breaker_config,
        };
        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: TripIx {}.data(),
        };
        self.send(&[ix], &[&keys.guardian]).map_err(|e| format!("{e:?}"))
    }

    pub fn resume(&mut self, keys: &TestKeypairs, as_guardian: bool) -> Result<(), String> {
        let payer = if as_guardian {
            &keys.guardian
        } else {
            &keys.payer
        };
        self.resume_with_payer(payer)
    }

    pub fn resume_with_payer(&mut self, payer: &Keypair) -> Result<(), String> {
        let (breaker_config, _) = self.derive_breaker_config();
        let accounts = Resume {
            payer: payer.pubkey(),
            breaker_config,
        };
        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: ResumeIx {}.data(),
        };
        self.send(&[ix], &[payer]).map_err(|e| format!("{e:?}"))
    }

    pub fn warp_time(&mut self, seconds_forward: i64) {
        let mut clock: Clock = self.svm.get_sysvar();
        clock.unix_timestamp += seconds_forward;
        self.svm.set_sysvar(&clock);
    }

    pub fn get_breaker_config(&self) -> BreakerConfig {
        let (addr, _) = self.derive_breaker_config();
        let account = self.svm.get_account(&addr).expect("breaker_config");
        BreakerConfig::try_deserialize(&mut account.data.as_slice()).expect("deserialize config")
    }

    pub fn get_token_account(&self, address: &Pubkey) -> TokenAccountState {
        let account = self.svm.get_account(address).expect("token account");
        TokenAccountState::unpack(&account.data).expect("unpack token")
    }

    pub fn derive_breaker_config(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[BREAKER_CONFIG_SEED, self.vault.as_ref()], &PROGRAM_ID)
    }

    pub fn derive_window_state(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[WINDOW_STATE_SEED, self.vault.as_ref()], &PROGRAM_ID)
    }

    pub fn derive_authority(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[AUTHORITY_SEED, self.vault.as_ref()], &PROGRAM_ID)
    }

    pub fn derive_pending_config(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[PENDING_CONFIG_SEED, self.vault.as_ref()], &PROGRAM_ID)
    }

    pub fn get_pending_config(&self) -> PendingConfigChange {
        let (addr, _) = self.derive_pending_config();
        let account = self.svm.get_account(&addr).expect("pending_config");
        PendingConfigChange::try_deserialize(&mut account.data.as_slice())
            .expect("deserialize pending")
    }

    pub fn get_window_state(&self) -> WindowState {
        let (addr, _) = self.derive_window_state();
        let account = self.svm.get_account(&addr).expect("window_state");
        WindowState::try_deserialize(&mut account.data.as_slice()).expect("deserialize window")
    }

    pub fn propose_config_change(
        &mut self,
        keys: &TestKeypairs,
        new_params: ProposedConfigParams,
        requested_delay: i64,
    ) -> Result<(), String> {
        let (breaker_config, _) = self.derive_breaker_config();
        let (pending_config, _) = self.derive_pending_config();

        let accounts = ProposeConfigChange {
            guardian: keys.guardian.pubkey(),
            payer: keys.payer.pubkey(),
            breaker_config,
            pending_config,
            system_program: system_program::ID,
        };

        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: ProposeConfigChangeIx {
                new_params,
                requested_delay,
            }
            .data(),
        };

        self.send(&[ix], &[&keys.payer, &keys.guardian])
            .map_err(|e| format!("{e:?}"))
    }

    pub fn execute_config_change(&mut self, keys: &TestKeypairs) -> Result<(), String> {
        let (breaker_config, _) = self.derive_breaker_config();
        let (window_state, _) = self.derive_window_state();
        let (pending_config, _) = self.derive_pending_config();

        let accounts = ExecuteConfigChange {
            guardian: keys.guardian.pubkey(),
            payer: keys.payer.pubkey(),
            breaker_config,
            window_state,
            pending_config,
        };

        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: ExecuteConfigChangeIx {}.data(),
        };

        self.send(&[ix], &[&keys.payer, &keys.guardian])
            .map_err(|e| format!("{e:?}"))
    }

    pub fn cancel_config_change(&mut self, keys: &TestKeypairs) -> Result<(), String> {
        let (breaker_config, _) = self.derive_breaker_config();
        let (pending_config, _) = self.derive_pending_config();

        let accounts = CancelConfigChange {
            guardian: keys.guardian.pubkey(),
            payer: keys.payer.pubkey(),
            breaker_config,
            pending_config,
        };

        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: CancelConfigChangeIx {}.data(),
        };

        self.send(&[ix], &[&keys.payer, &keys.guardian])
            .map_err(|e| format!("{e:?}"))
    }

    pub fn emergency_route_to_safe(
        &mut self,
        keys: &TestKeypairs,
        amount: u64,
        destination: Pubkey,
    ) -> Result<(), String> {
        let (breaker_config, _) = self.derive_breaker_config();
        let (breaker_authority, _) = self.derive_authority();

        let accounts = EmergencyRouteToSafe {
            guardian: keys.guardian.pubkey(),
            breaker_config,
            vault: self.vault,
            safe_destination: destination,
            breaker_authority,
            token_program: spl_token::ID,
        };

        let ix = solana_sdk::instruction::Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: EmergencyRouteToSafeIx { amount }.data(),
        };

        self.send(&[ix], &[&keys.guardian]).map_err(|e| format!("{e:?}"))
    }

    pub fn create_alt_destination(&mut self, keys: &TestKeypairs, owner: &Pubkey) -> Pubkey {
        let mint = self.mint;
        self.create_token_account(&keys.payer, &mint, owner)
    }

    pub fn pending_config_exists(&self) -> bool {
        let (addr, _) = self.derive_pending_config();
        match self.svm.get_account(&addr) {
            Some(account) => account.owner == PROGRAM_ID && account.data.len() >= 8,
            None => false,
        }
    }

    pub fn airdrop(&mut self, pubkey: &Pubkey, lamports: u64) {
        self.svm
            .airdrop(pubkey, lamports)
            .expect("airdrop failed");
    }

    fn send(
        &mut self,
        instructions: &[solana_sdk::instruction::Instruction],
        signers: &[&Keypair],
    ) -> Result<(), litesvm::types::FailedTransactionMetadata> {
        let blockhash = self.svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(&signers[0].pubkey()),
            signers,
            blockhash,
        );
        self.svm.send_transaction(tx).map(|_| ())
    }

    fn create_mint(&mut self, payer: &Keypair, authority: &Pubkey) -> Pubkey {
        let mint = Keypair::new();
        let rent = self.svm.minimum_balance_for_rent_exemption(MintState::LEN);
        self.create_account(&mint, MintState::LEN, rent, &spl_token::ID, payer);

        let ix = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            authority,
            None,
            6,
        )
        .unwrap();

        self.send(&[ix], &[payer]).expect("init mint");
        mint.pubkey()
    }

    pub fn create_token_account(
        &mut self,
        payer: &Keypair,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Pubkey {
        let account = Keypair::new();
        let rent = self
            .svm
            .minimum_balance_for_rent_exemption(TokenAccountState::LEN);
        self.create_account(&account, TokenAccountState::LEN, rent, &spl_token::ID, payer);

        let ix = spl_token::instruction::initialize_account(
            &spl_token::ID,
            &account.pubkey(),
            mint,
            owner,
        )
        .unwrap();

        self.send(&[ix], &[payer]).expect("init token account");
        account.pubkey()
    }

    fn create_account(
        &mut self,
        new_account: &Keypair,
        space: usize,
        lamports: u64,
        owner: &Pubkey,
        payer: &Keypair,
    ) {
        let ix = solana_sdk::system_instruction::create_account(
            &payer.pubkey(),
            &new_account.pubkey(),
            lamports,
            space as u64,
            owner,
        );
        self.send(&[ix], &[payer, new_account])
            .expect("create account");
    }

    fn mint_to(
        &mut self,
        payer: &Keypair,
        mint_authority: &Keypair,
        destination: &Pubkey,
        amount: u64,
    ) {
        let ix = spl_token::instruction::mint_to(
            &spl_token::ID,
            &self.mint,
            destination,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap();
        self.send(&[ix], &[payer, mint_authority])
            .expect("mint_to");
    }
}

fn program_so_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("deploy")
        .join("circuit_breaker.so")
}

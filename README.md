# SolHawk — Solana Circuit Breaker Protocol

Custodial-authority circuit breaker for SPL vaults. Phase 1 implements the core on-chain primitive: bucketed sliding-window outflow accounting, `initialize_breaker`, `guarded_withdraw`, `trip`, and `resume`.

Architecture spec: see `circuit-breaker-architecture.md` (source of truth for threat model, accounts, and invariants).

## Project layout

```
circuit_breaker/
├── programs/circuit_breaker/     # Anchor on-chain program
│   └── src/
│       ├── lib.rs                # Account structs + instruction dispatch
│       ├── state/                # BreakerConfig, WindowState, enums
│       ├── window/accounting.rs  # Pure §6 sliding-window (unit tested)
│       └── instructions/         # Per-instruction handlers
├── integration-tests/            # LiteSVM program tests (WSL2)
├── tests/                        # Anchor TS tests (Phase 3; not used in Phase 1)
└── Anchor.toml
```

## Requirements

- **WSL2 (Ubuntu)** — LiteSVM has no native Windows binaries; build and test inside WSL.
- Anchor CLI 0.31.x (matches `anchor-lang = "0.31.1"`)
- Rust stable (1.86+ recommended)
- Solana/Agave CLI 3.x (`agave-install update` if `anchor build` fails on dependency resolution)

## Build & test (WSL2)

```bash
cd /mnt/c/Users/USAJu/Desktop/SolHawk/circuit_breaker

# Update Agave platform tools if needed
agave-install update

# Build the program + IDL
anchor build

# Tier 1: pure sliding-window property tests (fast, no VM)
cargo test -p circuit_breaker

# Tier 2: LiteSVM integration tests (requires built .so)
cargo test -p circuit-breaker-tests
```

## Phase 1 instructions

| Instruction | Caller | Purpose |
|---|---|---|
| `initialize_breaker` | Payer + vault authority | Create PDAs, set breaker PDA as vault authority, fix immutable params |
| `guarded_withdraw` | Operator | Window check → record outflow → PDA-signed SPL transfer |
| `trip` | Guardian | Immediate emergency stop |
| `resume` | Guardian (or anyone after cooldown if `auto_recover`) | `Tripped → Active` after cooldown |

## Security notes

- **Sole transfer path:** Vault funds move only via `guarded_withdraw`; breaker authority PDA is the only signer on SPL transfers.
- **Window before CPI:** `WindowState` is mutated before `token::transfer`.
- **Velocity trip persistence:** When threshold is exceeded, the breaker trips and returns `Ok(())` without transferring. Returning an error would roll back the trip state under Solana's atomic transaction model.
- **`last_bucket_ts` monotonic:** Advanced by `elapsed * bucket_duration`, never rewound to `now`.
- **`PctOfBalance`:** Reads live `vault.amount` at check time.

## Phase 2 seams (not implemented)

Reserved in code comments / seed constants:

- `PENDING_CONFIG_SEED` — timelocked `PendingConfigChange`
- `propose_config_change` / `execute_config_change`
- `emergency_route_to_safe` — immutable `safe_destination` already stored at init
- Destination allowlist hook in `guarded_withdraw`

## Program ID (localnet)

`7Wuw9J1cW8V2R1KAA6T3y3bRDpBYJ5FkQWMj8gDKb1MW`

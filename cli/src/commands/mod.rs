pub mod demo;
pub mod init;
pub mod resume;
pub mod status;
pub mod trip;

// Phase 2+: watch, config-change, emergency-route

use anyhow::bail;

use crate::cli::{Cli, Commands};

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Status { vault, json } => status::run(&cli.global, vault, json),
        Commands::Init(args) => init::run(&cli.global, args),
        Commands::Trip { vault, yes } => trip::run(&cli.global, vault, yes),
        Commands::Resume { vault, yes } => resume::run(&cli.global, vault, yes),
        Commands::Demo(args) => demo::run(&cli.global, args),
    }
}

pub fn not_implemented(cmd: &str) -> anyhow::Result<()> {
    bail!("`solhawk {cmd}` is not implemented yet — coming in the next phase")
}

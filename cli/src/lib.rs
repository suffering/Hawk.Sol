pub mod branding;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod ix;
pub mod model;
pub mod render;
pub mod rpc;

use anyhow::Result;
use clap::Parser;

use branding::{is_tty_stdout, print_banner, terminal_palette};
use cli::Cli;

pub fn main_entry() -> Result<()> {
    let cli = Cli::parse();

    let show_banner = is_tty_stdout()
        && !cli.global.quiet
        && !wants_json_output(&cli);

    if show_banner {
        print_banner(&terminal_palette(cli.global.quiet));
    }

    commands::run(cli)
}

fn wants_json_output(cli: &Cli) -> bool {
    matches!(
        cli.command,
        cli::Commands::Status { json: true, .. } | cli::Commands::Demo(crate::cli::DemoArgs { json: true, .. })
    )
}

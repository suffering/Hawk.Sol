mod branding;
mod cli;
mod commands;
mod config;
mod error;
mod ix;
mod model;
mod render;
mod rpc;

use anyhow::Result;
use clap::Parser;

use branding::{is_tty_stdout, print_banner, terminal_palette};
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let show_banner = is_tty_stdout()
        && !cli.global.quiet
        && !status_wants_json(&cli);

    if show_banner {
        print_banner(&terminal_palette(cli.global.quiet));
    }

    commands::run(cli)
}

fn status_wants_json(cli: &Cli) -> bool {
    matches!(
        cli.command,
        cli::Commands::Status { json: true, .. }
    )
}

use anyhow::Result;

use crate::cli::{GlobalOpts, InitArgs};

pub fn run(_global: &GlobalOpts, _args: InitArgs) -> Result<()> {
    crate::commands::not_implemented("init")
}

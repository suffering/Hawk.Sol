use anyhow::Result;

use crate::cli::{DemoArgs, GlobalOpts};

pub fn run(_global: &GlobalOpts, _args: DemoArgs) -> Result<()> {
    crate::commands::not_implemented("demo")
}

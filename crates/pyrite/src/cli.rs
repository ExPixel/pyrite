use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
pub struct PyriteCli {
    pub rom: Option<PathBuf>,

    #[arg(short, long = "debugger", default_value_t = false)]
    pub debugger_enabled: bool,
}

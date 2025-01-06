use clap::{command, Parser};

use crate::chip8::Mode;

/// chip8 emulator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliOptions {
    /// The program to run.
    #[arg(short, long)]
    pub program: String,

    /// The mode to run in.
    #[arg(short, long, default_value = "chip48")]
    pub mode: Mode,

    /// Operations to run per second.
    #[arg(short, long, default_value = "700")]
    pub speed: u64,

    // Flag for printing debug information.
    #[arg(short, long)]
    pub debug: bool,
}

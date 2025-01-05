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
    #[arg(short, long)]
    pub mode: Mode,
}

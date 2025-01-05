use clap::{command, Parser};

/// chip8 emulator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliOptions {
    /// The program to run.
    #[arg(short, long)]
    pub program: String,
}

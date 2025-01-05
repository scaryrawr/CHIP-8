use std::{
    io::{stdout, Error},
    time,
};

use chip8::decode;
use clap::Parser;
use cli::CliOptions;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Print,
    terminal, ExecutableCommand, QueueableCommand,
};

mod chip8;
mod cli;

fn draw(display: &[[u8; 64]; 32]) -> Result<(), Error> {
    let mut stdout = stdout();
    for (i, row) in display.iter().enumerate() {
        stdout.queue(cursor::MoveTo(0, i as u16))?;
        for &pixel in row.iter() {
            if pixel == 0 {
                stdout.queue(Print(" "))?;
            } else {
                stdout.queue(Print("█"))?;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let options = CliOptions::parse();

    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    stdout.execute(cursor::Hide)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let mut chip8 = chip8::Chip8::new();
    let program = std::fs::read(&options.program)?;
    chip8.load(&program);

    let mut timer = time::Instant::now();
    loop {
        let start = time::Instant::now();
        let opcode = chip8.fetch();

        let instruction = decode(opcode);
        chip8.execute(&instruction, get_keyboard_state()?)?;

        // Attempt to evaluate around 1000 ops per second
        while time::Instant::now() - start < time::Duration::from_millis(60_000 / 1000) {}

        // Redraw the display
        draw(&chip8.display)?;

        // Update delay and sound timer at 60hz
        if time::Instant::now() - timer > time::Duration::from_millis(1_000 / 60) {
            timer = time::Instant::now();
            if chip8.delay_timer > 0 {
                chip8.delay_timer -= 1;
            }

            if chip8.sound_timer > 0 {
                chip8.sound_timer -= 1;
            }
        }
    }
}

fn get_keyboard_state() -> Result<u8, Error> {
    if crossterm::event::poll(time::Duration::from_millis(60_000 / 1000))? {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                return Err(Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Ctrl+C pressed",
                ))
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('1'),
                ..
            }) => return Ok(0x1),
            Event::Key(KeyEvent {
                code: KeyCode::Char('2'),
                ..
            }) => return Ok(0x2),
            Event::Key(KeyEvent {
                code: KeyCode::Char('3'),
                ..
            }) => return Ok(0x3),
            Event::Key(KeyEvent {
                code: KeyCode::Char('4'),
                ..
            }) => return Ok(0xC),
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => return Ok(0x4),
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                ..
            }) => return Ok(0x5),
            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                ..
            }) => return Ok(0x6),
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                ..
            }) => return Ok(0xD),
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                ..
            }) => return Ok(0x7),
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                ..
            }) => return Ok(0x8),
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                ..
            }) => return Ok(0x9),
            Event::Key(KeyEvent {
                code: KeyCode::Char('f'),
                ..
            }) => return Ok(0xE),
            Event::Key(KeyEvent {
                code: KeyCode::Char('z'),
                ..
            }) => return Ok(0xA),
            Event::Key(KeyEvent {
                code: KeyCode::Char('x'),
                ..
            }) => return Ok(0x0),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                ..
            }) => return Ok(0xB),
            Event::Key(KeyEvent {
                code: KeyCode::Char('v'),
                ..
            }) => return Ok(0xF),
            _ => {}
        }
    }

    Ok(0)
}

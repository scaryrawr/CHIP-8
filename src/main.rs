use std::{
    io::{stdout, Error, Write},
    process::exit,
    time,
};

use chip8::{decode, Chip8, KeyboardState};
use clap::Parser;
use cli::CliOptions;
use crossterm::{
    cursor,
    event::{
        read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
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

    stdout.flush()?;

    Ok(())
}

fn draw_debug(machine: &Chip8, keyboard: &KeyboardState) -> Result<(), Error> {
    const DEBUG_COLUMN: u16 = 66;
    let mut stdout = stdout();
    let info: [String; 8] = [
        format!("PC: {:#06x}", machine.program_counter),
        format!("I: {:#06x}", machine.index_register),
        format!("DT: {:#04x}", machine.delay_timer),
        format!("ST: {:#04x}", machine.sound_timer),
        format!("SP: {:#04x}", machine.stack_pointer),
        format!("Mode: {:?}", machine.mode),
        format!("Key: {:?}", keyboard.pressed_key),
        format!(
            "Pressed: {:?}",
            keyboard
                .keys_pressed
                .iter()
                .enumerate()
                .filter(|(_, &v)| v)
                .map(|(i, _)| format!("{:#x}", i))
                .collect::<Vec<_>>()
        ),
    ];

    for (i, line) in info.iter().enumerate() {
        stdout
            .queue(cursor::MoveTo(DEBUG_COLUMN, i as u16))?
            .queue(Print(line.as_str()))?
            .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
    }

    stdout.flush()?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let options = CliOptions::parse();

    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;

    stdout.execute(cursor::Hide)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let mut chip8 = chip8::Chip8::new(options.mode);
    let program = std::fs::read(&options.program)?;
    chip8.load(&program);

    let mut timer = time::Instant::now();
    let mut keyboard_state = KeyboardState::new();
    loop {
        let start = time::Instant::now();
        let opcode = chip8.fetch();

        let instruction = decode(opcode);
        update_keyboard_state(&mut keyboard_state)?;
        let action = chip8.execute(&instruction, &keyboard_state)?;

        // Attempt to evaluate around 1000 ops per second
        while time::Instant::now() - start < time::Duration::from_millis(1_000 / 700) {}

        // Redraw the display
        match action {
            chip8::Actions::Redraw => {
                draw(&chip8.display)?;
            }
            chip8::Actions::None => {}
        }

        if options.debug {
            draw_debug(&chip8, &keyboard_state)?;
        }

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

fn update_keyboard_state(state: &mut KeyboardState) -> Result<(), Error> {
    const KEYS: [KeyCode; 16] = [
        KeyCode::Char('x'),
        KeyCode::Char('1'),
        KeyCode::Char('2'),
        KeyCode::Char('3'),
        KeyCode::Char('q'),
        KeyCode::Char('w'),
        KeyCode::Char('e'),
        KeyCode::Char('a'),
        KeyCode::Char('s'),
        KeyCode::Char('d'),
        KeyCode::Char('z'),
        KeyCode::Char('c'),
        KeyCode::Char('4'),
        KeyCode::Char('r'),
        KeyCode::Char('f'),
        KeyCode::Char('v'),
    ];

    state.pressed_key = None;
    if crossterm::event::poll(time::Duration::from_millis(1_000 / 7000))? {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                exit(-1);
            }
            Event::Key(KeyEvent { code, kind, .. }) => {
                for (i, &key) in KEYS.iter().enumerate() {
                    if code == key {
                        match kind {
                            KeyEventKind::Press => {
                                state.keys_pressed[i] = true;
                                state.pressed_key = Some(i as u8);
                            }
                            KeyEventKind::Release => {
                                state.keys_pressed[i] = false;
                            }
                            KeyEventKind::Repeat => {
                                state.keys_pressed[i] = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

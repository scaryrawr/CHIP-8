use std::io::Error;

use fontset::FONTSET;

mod fontset;

pub const FONTSET_START_ADDRESS: usize = 0x50;
pub const PROGRAM_START_ADDRESS: usize = 0x200;

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum Mode {
    #[default]
    Chip8,
    Chip48,
}

pub enum Actions {
    None,
    Redraw,
}

pub struct KeyboardState {
    pub keys_pressed: [bool; 16],
    pub pressed_key: Option<u8>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            keys_pressed: [false; 16],
            pressed_key: None,
        }
    }
}

pub struct Chip8 {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub index_register: u16,
    pub program_counter: usize,
    pub stack: [usize; 16],
    pub stack_pointer: i8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: [[u8; 64]; 32],
    pub mode: Mode,
}

impl Chip8 {
    pub fn new(mode: Mode) -> Self {
        let mut machine = Self {
            memory: [0; 4096],
            registers: [0; 16],
            index_register: 0,
            program_counter: PROGRAM_START_ADDRESS,
            stack: [0; 16],
            stack_pointer: -1,
            delay_timer: 0,
            sound_timer: 0,
            display: [[0; 64]; 32],
            mode,
        };

        FONTSET.iter().enumerate().for_each(|(i, &byte)| {
            machine.memory[FONTSET_START_ADDRESS + i] = byte;
        });

        machine
    }

    pub fn load(&mut self, program: &[u8]) {
        program.iter().enumerate().for_each(|(i, &byte)| {
            self.memory[PROGRAM_START_ADDRESS + i] = byte;
        });
    }

    pub fn fetch(&mut self) -> u16 {
        let pc = self.program_counter;
        let byte1 = self.memory[pc] as u16;
        let byte2 = self.memory[pc + 1] as u16;

        self.program_counter += 2;

        byte1 << 8 | byte2
    }

    pub fn execute(
        &mut self,
        operation: &Instruction,
        keyboard_state: &KeyboardState,
    ) -> Result<Actions, Error> {
        match operation.instruction {
            0x00 => match operation.nn {
                0xE0 => {
                    // Clear the display
                    self.display = [[0; 64]; 32];
                    return Ok(Actions::Redraw);
                }
                0xEE => {
                    // Return from a subroutine
                    if self.stack_pointer >= 0 {
                        self.program_counter = self.stack[self.stack_pointer as usize];
                        self.stack_pointer -= 1;
                    }
                }
                _ => {
                    // Calls RCA 1802 program at address NNN
                }
            },
            0x01 => {
                // Jump to address NNN
                self.program_counter = operation.nnn;
            }
            0x02 => {
                // Call subroutine at NNN
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.program_counter = operation.nnn;
            }
            0x03 => {
                // Skip next instruction if Vx = NN
                if self.registers[operation.x] == operation.nn {
                    self.program_counter += 2;
                }
            }
            0x04 => {
                // Skip next instruction if Vx != NN
                if self.registers[operation.x] != operation.nn {
                    self.program_counter += 2;
                }
            }
            0x05 => {
                // Skip next instruction if Vx = Vy
                if self.registers[operation.x] == self.registers[operation.y] {
                    self.program_counter += 2;
                }
            }
            0x06 => {
                // Set Vx = NN
                self.registers[operation.x] = operation.nn;
            }
            0x07 => {
                // Set Vx = Vx + NN
                let (result, _) = self.registers[operation.x].overflowing_add(operation.nn);
                self.registers[operation.x] = result;
            }
            0x08 => match operation.n {
                0x00 => {
                    // Set Vx = Vy
                    self.registers[operation.x] = self.registers[operation.y];
                }
                0x01 => {
                    // Set Vx = Vx OR Vy
                    self.registers[operation.x] |= self.registers[operation.y];
                    match self.mode {
                        Mode::Chip8 => {
                            self.registers[0xf] = 0;
                        }
                        _ => {}
                    }
                }
                0x02 => {
                    // Set Vx = Vx AND Vy
                    self.registers[operation.x] &= self.registers[operation.y];
                    match self.mode {
                        Mode::Chip8 => {
                            self.registers[0xf] = 0;
                        }
                        _ => {}
                    }
                }
                0x03 => {
                    // Set Vx = Vx XOR Vy
                    self.registers[operation.x] ^= self.registers[operation.y];
                    match self.mode {
                        Mode::Chip8 => {
                            self.registers[0xf] = 0;
                        }
                        _ => {}
                    }
                }
                0x04 => {
                    // Set Vx = Vx + Vy, set VF = carry
                    let (result, overflow) =
                        self.registers[operation.x].overflowing_add(self.registers[operation.y]);
                    self.registers[operation.x] = result;
                    self.registers[0xF] = overflow as u8;
                }
                0x05 => {
                    // Set Vx = Vx - Vy, set VF = NOT borrow
                    let (result, overflow) =
                        self.registers[operation.x].overflowing_sub(self.registers[operation.y]);
                    self.registers[operation.x] = result;
                    self.registers[0xF] = !overflow as u8;
                }
                0x06 => {
                    match self.mode {
                        Mode::Chip8 => {
                            // Set Vx = Vy SHR 1
                            self.registers[operation.x] = self.registers[operation.y];
                        }
                        Mode::Chip48 => {}
                    }

                    let carry = self.registers[operation.x] & 1;
                    self.registers[operation.x] >>= 1;
                    self.registers[0xF] = carry;
                }
                0x07 => {
                    // Set Vx = Vy - Vx, set VF = NOT borrow
                    let (result, overflow) =
                        self.registers[operation.y].overflowing_sub(self.registers[operation.x]);
                    self.registers[operation.x] = result;
                    self.registers[0xF] = !overflow as u8;
                }
                0x0E => {
                    match self.mode {
                        Mode::Chip8 => {
                            // Set Vx = Vy SHL 1
                            self.registers[operation.x] = self.registers[operation.y];
                        }
                        Mode::Chip48 => {}
                    }

                    let carry = self.registers[operation.x] >> 7;
                    self.registers[operation.x] <<= 1;
                    self.registers[0xF] = carry;
                }
                _ => {}
            },
            0x09 => {
                // Skip next instruction if Vx != Vy
                if self.registers[operation.x] != self.registers[operation.y] {
                    self.program_counter += 2;
                }
            }
            0x0A => {
                // Set I = NNN
                self.index_register = operation.nnn as u16;
            }
            0x0B => {
                // Jump to location NNN + V0
                match self.mode {
                    Mode::Chip8 => {
                        self.program_counter = operation.nnn + self.registers[0] as usize;
                    }
                    Mode::Chip48 => {
                        self.program_counter = operation.nnn + self.registers[operation.x] as usize;
                    }
                }
            }
            0x0C => {
                // Set Vx = random byte AND NN
                let number: u8 = rand::random();
                self.registers[operation.x] = number & operation.nn;
            }
            0x0D => {
                // Display
                let x = (self.registers[operation.x] & 63) as usize;
                let y = (self.registers[operation.y] & 31) as usize;
                self.registers[0xF] = 0;
                let sprite = &self.memory
                    [self.index_register as usize..self.index_register as usize + operation.n];
                for (j, byte) in sprite.iter().enumerate() {
                    if y + j > 31 {
                        break;
                    }

                    for i in 0..8 {
                        if x + i > 63 {
                            break;
                        }

                        let pixel = (byte >> (7 - i)) & 1;
                        if pixel == 1 {
                            if self.display[y + j][x + i] == 1 {
                                self.registers[0xF] = 1;
                            }

                            self.display[y + j][x + i] ^= 1;
                        }
                    }
                }

                return Ok(Actions::Redraw);
            }
            0x0E => match operation.nn {
                0x9E => {
                    // Skip next instruction if key with the value of Vx is pressed
                    if keyboard_state.keys_pressed[self.registers[operation.x] as usize] {
                        self.program_counter += 2;
                    }
                }
                0xA1 => {
                    // Skip next instruction if key with the value of Vx is not pressed
                    if !keyboard_state.keys_pressed[self.registers[operation.x] as usize] {
                        self.program_counter += 2;
                    }
                }
                _ => {}
            },
            0x0F => match operation.nn {
                0x07 => {
                    // Set Vx = delay timer value
                    self.registers[operation.x] = self.delay_timer;
                }
                0x0A => {
                    // Wait for a key press, store the value of the key in Vx
                    if let Some(key) = keyboard_state.pressed_key {
                        self.registers[operation.x] = key;
                    } else {
                        self.program_counter -= 2;
                    }
                }
                0x15 => {
                    // Set delay timer = Vx
                    self.delay_timer = self.registers[operation.x];
                }
                0x18 => {
                    // Set sound timer = Vx
                    self.sound_timer = self.registers[operation.x];
                }
                0x1E => {
                    // Set I = I + Vx
                    self.index_register += self.registers[operation.x] as u16;
                }
                0x29 => {
                    // Set I = location of sprite for digit Vx
                    self.index_register =
                        FONTSET_START_ADDRESS as u16 + self.registers[operation.x] as u16 * 5;
                }
                0x33 => {
                    // Store BCD representation of Vx in memory locations I, I+1, and I+2
                    let value = self.registers[operation.x];
                    self.memory[self.index_register as usize] = value / 100;
                    self.memory[self.index_register as usize + 1] = (value / 10) % 10;
                    self.memory[self.index_register as usize + 2] = value % 10;
                }
                0x55 => {
                    // Store registers V0 through Vx in memory starting at location I
                    for i in 0..=operation.x {
                        self.memory[self.index_register as usize + i] = self.registers[i];
                    }

                    match self.mode {
                        Mode::Chip8 => {
                            self.index_register += operation.x as u16 + 1;
                        }
                        Mode::Chip48 => {}
                    }
                }
                0x65 => {
                    // Read registers V0 through Vx from memory starting at location I
                    for i in 0..=operation.x {
                        self.registers[i] = self.memory[self.index_register as usize + i];
                    }
                    match self.mode {
                        Mode::Chip8 => {
                            self.index_register += operation.x as u16 + 1;
                        }
                        Mode::Chip48 => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(Actions::None)
    }
}

pub struct Instruction {
    instruction: u8,
    x: usize,
    y: usize,
    n: usize,
    nn: u8,
    nnn: usize,
}

pub fn decode(opcode: u16) -> Instruction {
    let instruction = ((opcode & 0xF000) >> 12) as u8;
    let x = ((opcode & 0x0F00) >> 8) as usize;
    let y = ((opcode & 0x00F0) >> 4) as usize;
    let n = (opcode & 0x000F) as usize;
    let nn = (opcode & 0x00FF) as u8;
    let nnn = (opcode & 0x0FFF) as usize;
    return Instruction {
        instruction,
        x,
        y,
        n,
        nn,
        nnn,
    };
}

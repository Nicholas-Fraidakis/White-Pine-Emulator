/*
    Sources:
        - http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
*/

use macroquad::{
    input::{is_key_down, is_key_pressed, KeyCode},
    time::get_fps,
};

type Chip8InstructionRaw = [u8; 2];

type Register = u8;
#[derive(Debug)]
enum Source {
    Register(Register),
    Immediate(u8),
}
impl Default for Source {
    fn default() -> Self {
        Self::Immediate(0)
    }
}

trait Decodeable {
    fn decode(instruction: u16) -> Self;
}
#[derive(Debug)]
pub struct DebugPrintOperand(String);
#[derive(Debug)]
pub struct AddressOperand(usize);
impl Decodeable for AddressOperand {
    fn decode(instruction: u16) -> Self {
        Self(instruction as usize & 0xfff)
    }
}
#[derive(Debug)]
pub struct RegisterValueOperands {
    register: Register,
    value: Source,
}
impl Decodeable for RegisterValueOperands {
    fn decode(instruction: u16) -> Self {
        Self {
            register: (instruction >> 8 & 0xf) as u8,
            value: match instruction >> 12 {
                8 | 5 | 9 => Source::Register((instruction >> 4 & 0xf) as u8),
                _ => Source::Immediate((instruction & 0xff) as u8),
            },
        }
    }
}

#[derive(Debug)]
pub struct SingleRegisterOperand {
    register: Register,
}
impl Decodeable for SingleRegisterOperand {
    fn decode(instruction: u16) -> Self {
        Self {
            register: (instruction >> 8 & 0xf) as u8,
        }
    }
}

#[derive(Debug)]
pub struct DrawSpriteOperands {
    x_register: Register,
    y_register: Register,
    height: Register,
}
impl Decodeable for DrawSpriteOperands {
    fn decode(instruction: u16) -> Self {
        assert!(
            instruction >> 12 == 0xd,
            "Wrongly Attempted To Decode {instruction} Operands As A DrawSpriteOperands!"
        );
        Self {
            x_register: (instruction >> 8 & 0xf) as u8,
            y_register: (instruction >> 4 & 0xf) as u8,
            height: (instruction & 0xf) as u8,
        }
    }
}

#[derive(Debug)]
pub enum Chip8Instruction {
    Unimplemented,
    Terminate,
    Panic(DebugPrintOperand),
    ClearScreen,
    Ret,
    Jump(AddressOperand),
    JumpPlusR0(AddressOperand),
    Call(AddressOperand),
    SkipIfEqual(RegisterValueOperands),
    SkipIfNotEqual(RegisterValueOperands),
    Store(RegisterValueOperands),
    Add(RegisterValueOperands),
    Sub(RegisterValueOperands),
    SubTo(RegisterValueOperands),

    OR(RegisterValueOperands),
    AND(RegisterValueOperands),
    XOR(RegisterValueOperands),
    BitShiftLeftOnce(RegisterValueOperands),
    BitShiftRightOnce(RegisterValueOperands),

    GetRandom(RegisterValueOperands),

    SaveAddress(AddressOperand),

    DrawSprite(DrawSpriteOperands),

    SkipIfKeyPressed(SingleRegisterOperand),
    SkipIfKeyNotPressed(SingleRegisterOperand),

    GetDelayTimer(SingleRegisterOperand),

    SetDelayTimer(SingleRegisterOperand),
    SetSoundTimer(SingleRegisterOperand),

    WaitForKeyPress(SingleRegisterOperand),

    AddToAddress(SingleRegisterOperand),

    SaveSpriteDigitAddress(SingleRegisterOperand),

    StoreBinaryCodedDecimal(SingleRegisterOperand),

    StoreRegistersIntoMemoryChunk(SingleRegisterOperand),
    StoreMemoryChunkIntoRegisters(SingleRegisterOperand),
}

const HEX_KEYPAD: [char; 16] = [
    'x', '1', '2', '3', '4', 'w', 'e', 'a', 's', 'd', 'z', 'c', '4', 'r', 'f', 'v',
];
/*
0x */
fn char_to_keycode(key: char) -> Option<macroquad::input::KeyCode> {
    let key = key.to_ascii_lowercase();

    match key {
        'a' => Some(KeyCode::A),
        'b' => Some(KeyCode::B),
        'c' => Some(KeyCode::C),
        'd' => Some(KeyCode::D),
        'e' => Some(KeyCode::E),
        'f' => Some(KeyCode::F),
        'g' => Some(KeyCode::G),
        'h' => Some(KeyCode::H),
        'i' => Some(KeyCode::I),
        'j' => Some(KeyCode::J),
        'k' => Some(KeyCode::K),
        'l' => Some(KeyCode::L),
        'm' => Some(KeyCode::M),
        'n' => Some(KeyCode::N),
        'o' => Some(KeyCode::O),
        'p' => Some(KeyCode::P),
        'q' => Some(KeyCode::Q),
        'r' => Some(KeyCode::R),
        's' => Some(KeyCode::S),
        't' => Some(KeyCode::T),
        'u' => Some(KeyCode::U),
        'v' => Some(KeyCode::V),
        'w' => Some(KeyCode::W),
        'x' => Some(KeyCode::X),
        'y' => Some(KeyCode::Y),
        'z' => Some(KeyCode::Z),
        '1' => Some(KeyCode::Key1),
        '2' => Some(KeyCode::Key2),
        '3' => Some(KeyCode::Key3),
        '4' => Some(KeyCode::Key4),
        '5' => Some(KeyCode::Key5),
        '6' => Some(KeyCode::Key6),
        '7' => Some(KeyCode::Key7),
        '8' => Some(KeyCode::Key8),
        '9' => Some(KeyCode::Key9),
        '0' => Some(KeyCode::Key0),
        ' ' => Some(KeyCode::Space),
        _ => None,
    }
}

fn hexpad_wait_keypress() -> u8 {
    loop {
        for i in 0..HEX_KEYPAD.len() {
            if is_key_pressed(char_to_keycode(HEX_KEYPAD[i]).unwrap_or(KeyCode::Key1)) {
                return i as u8;
            }
        }
    }
}

fn hexpad_get_keypress(key: u8) -> bool {
    let keycode = char_to_keycode(HEX_KEYPAD[key as usize]).unwrap_or(KeyCode::Space);
    println!("Key {:?}, down is {}", keycode, is_key_down(keycode));
    is_key_down(keycode)
}

impl Chip8Instruction {
    pub fn decode(raw_instruction: Chip8InstructionRaw) -> Chip8Instruction {
        let instruction: u16 = ((raw_instruction[0] as u16) << 8) + raw_instruction[1] as u16;
        match instruction >> 12 {
            0 => match instruction {
                0x_0000 => Chip8Instruction::Terminate, // Probably encountered end of program so lets just dump the context and terminate
                0x_00E0 => Chip8Instruction::ClearScreen,
                0x_00EE => Chip8Instruction::Ret,
                _ => Chip8Instruction::Panic(DebugPrintOperand("0NNN is NOT ALLOWED".to_owned())),
            },
            1 => Chip8Instruction::Jump(AddressOperand::decode(instruction)),
            2 => Chip8Instruction::Call(AddressOperand::decode(instruction)),
            3 | 5 => Chip8Instruction::SkipIfEqual(RegisterValueOperands::decode(instruction)),
            4 | 9 => Chip8Instruction::SkipIfNotEqual(RegisterValueOperands::decode(instruction)),
            6 => Chip8Instruction::Store(RegisterValueOperands::decode(instruction)),
            7 => Chip8Instruction::Add(RegisterValueOperands::decode(instruction)),

            8 => match instruction & 0xf {
                0 => Chip8Instruction::Store(RegisterValueOperands::decode(instruction)),
                1 => Chip8Instruction::OR(RegisterValueOperands::decode(instruction)),
                2 => Chip8Instruction::AND(RegisterValueOperands::decode(instruction)),
                3 => Chip8Instruction::XOR(RegisterValueOperands::decode(instruction)),
                4 => Chip8Instruction::Add(RegisterValueOperands::decode(instruction)),
                5 => Chip8Instruction::Sub(RegisterValueOperands::decode(instruction)),
                6 => {
                    Chip8Instruction::BitShiftRightOnce(RegisterValueOperands::decode(instruction))
                }
                7 => Chip8Instruction::SubTo(RegisterValueOperands::decode(instruction)),
                0xE => {
                    Chip8Instruction::BitShiftLeftOnce(RegisterValueOperands::decode(instruction))
                }
                _ => {
                    println!(
                        "WARNING: Attempted to decode unrecognized instruction {:x}!",
                        instruction
                    );
                    Chip8Instruction::Unimplemented
                }
            },
            0xA => Chip8Instruction::SaveAddress(AddressOperand::decode(instruction)),
            0xB => Chip8Instruction::JumpPlusR0(AddressOperand::decode(instruction)),
            0xC => Chip8Instruction::GetRandom(RegisterValueOperands::decode(instruction)),
            0xD => Chip8Instruction::DrawSprite(DrawSpriteOperands::decode(instruction)),
            0xE => match instruction & 0xff {
                0x9E => {
                    Chip8Instruction::SkipIfKeyPressed(SingleRegisterOperand::decode(instruction))
                }
                0xA1 => Chip8Instruction::SkipIfKeyNotPressed(SingleRegisterOperand::decode(
                    instruction,
                )),
                _ => {
                    println!(
                        "WARNING: Attempted to decode unrecognized instruction {:x}!",
                        instruction
                    );
                    Chip8Instruction::Unimplemented
                }
            },
            0xF => match instruction & 0xff {
                0x07 => Chip8Instruction::GetDelayTimer(SingleRegisterOperand::decode(instruction)),
                0x0A => {
                    Chip8Instruction::WaitForKeyPress(SingleRegisterOperand::decode(instruction))
                }
                0x15 => Chip8Instruction::SetDelayTimer(SingleRegisterOperand::decode(instruction)),
                0x18 => Chip8Instruction::SetSoundTimer(SingleRegisterOperand::decode(instruction)),
                0x1E => Chip8Instruction::AddToAddress(SingleRegisterOperand::decode(instruction)),
                0x29 => Chip8Instruction::SaveSpriteDigitAddress(SingleRegisterOperand::decode(
                    instruction,
                )),
                0x33 => Chip8Instruction::StoreBinaryCodedDecimal(SingleRegisterOperand::decode(
                    instruction,
                )),
                0x55 => Chip8Instruction::StoreRegistersIntoMemoryChunk(
                    SingleRegisterOperand::decode(instruction),
                ),
                0x65 => Chip8Instruction::StoreMemoryChunkIntoRegisters(
                    SingleRegisterOperand::decode(instruction),
                ),
                _ => {
                    println!(
                        "WARNING: Attempted to decode unrecognized instruction {:x}!",
                        instruction
                    );
                    Chip8Instruction::Unimplemented
                }
            },
            _ => {
                println!(
                    "WARNING: Attempted to decode unrecognized instruction {:x}!",
                    instruction
                );
                Chip8Instruction::Unimplemented
            }
        }
    }
}

#[derive(Debug)]
pub struct CPUContext {
    pub r: [u8; 16],
    pub i: usize,
    pub program_counter: usize,
    pub delay_timer: f32,
    pub sound_timer: u8,
    stack_pointer: usize,
    call_stack: [usize; 16],
}

impl CPUContext {
    pub fn dump(&self) {
        println!("{:?}", *self);
    }

    pub fn update_delay(&mut self) {
        self.delay_timer -= 60.0 / get_fps() as f32;
    }

    pub fn push_return_address(&mut self) {
        if self.stack_pointer >= self.call_stack.len() {
            panic!(
                "Stack Overflow on call_stack! call_stack.len() = 0x{:x}\n",
                self.call_stack.len()
            );
        }
        self.call_stack[self.stack_pointer] = self.program_counter;
        self.stack_pointer += 1;
    }

    pub fn pop_return_address(&mut self) -> usize {
        let returned_address: usize;
        if self.stack_pointer == 0 {
            self.dump();
            panic!(
                "Stack Underflow on call_stack! call_stack.len() = 0x{:x}\n",
                self.call_stack.len()
            );
        }

        returned_address = self.call_stack[self.stack_pointer - 1];
        //println!("Attempting to return to {:x}", returned_address);
        self.stack_pointer -= 1;

        returned_address
    }
}

// Note you index [y][x]

pub struct Chip8Sprite {
    height: usize,
    raw_sprite: [u8; 17],
}

impl Chip8Sprite {
    pub fn new(raw_sprite: &[u8]) -> Self {
        Self {
            height: raw_sprite.len(),
            raw_sprite: {
                let mut buffer: [u8; 17] = [0; 17];
                buffer[..raw_sprite.len()].copy_from_slice(&raw_sprite[..raw_sprite.len()]);
                buffer
            },
        }
    }

    pub fn get_width(&self) -> usize {
        8
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.raw_sprite[0..self.height]
    }
}

pub struct Chip8Display([[u8; 8]; 32]);
impl Chip8Display {
    pub fn new() -> Self {
        Self([[0; 8]; 32])
    }
    pub fn get_resolution(&self) -> (usize, usize) {
        (self.0[0].len(), self.0.len())
    }
    pub fn access_position(&self, x: usize, y: usize) -> bool {
        self.0[y][x / 8] & (1 << (x % 8)) != 0
    }

    pub fn clear(&mut self) {
        self.0 = [[0; 8]; 32];
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &Chip8Sprite) -> u8 {
        let display_resolution: (usize, usize);
        let drawn_outside_screen: u8;
        let sprite_slice: &[u8];

        sprite_slice = sprite.as_slice();
        display_resolution = self.get_resolution();

        match x + sprite.get_width() >= display_resolution.0
            || y + sprite.get_height() >= display_resolution.1
        {
            true => drawn_outside_screen = 1,
            false => drawn_outside_screen = 0,
        };

        for current_y in 0..sprite.get_height() {
            let adjusted_byte: u8 = sprite_slice[current_y].reverse_bits();
            let left_byte: u8;
            let right_byte: u8;

            if y + current_y >= display_resolution.1 {
                break;
            }

            left_byte = adjusted_byte << (x % 8);
            right_byte = (((adjusted_byte as u16) << (x % 8)) >> 8) as u8;

            if x / 8 < display_resolution.0 {
                self.0[y + current_y][x / 8] ^= left_byte;
            }

            if x / 8 + 1 < display_resolution.0 {
                self.0[y + current_y][x / 8 + 1] ^= right_byte;
            }
        }
        drawn_outside_screen
    }
}
pub struct Chip8 {
    pub ctx: CPUContext,
    is_running: bool,
    memory: [u8; 4096],
    display: Chip8Display,
}

impl Chip8 {
    pub fn new(program: &[u8]) -> Self {
        assert!(
            program.len() < 0xdff,
            "Program must be smaller than 0xdff bytes!"
        );
        Self {
            ctx: CPUContext {
                r: [0; 16],
                i: 0,
                program_counter: 0x200,
                stack_pointer: 0,
                call_stack: [0x200; 16],
                delay_timer: 0.0,
                sound_timer: 0,
            },
            is_running: true,
            memory: {
                let mut memory: [u8; 4096] = [0; 4096];

                let hexadecimal_digit_sprites: [u8; 0x50] = [
                    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0,
                    0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0,
                    0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40,
                    0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90,
                    0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0,
                    0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0,
                    0x80, 0x80,
                ];

                memory[..0x50].copy_from_slice(&hexadecimal_digit_sprites);
                memory[0x200..][..program.len()].copy_from_slice(program);

                memory
            },
            display: Chip8Display::new(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
    
    pub fn fetch(&mut self) -> Chip8Instruction {
        let instruction: [u8; 2] = [
            self.memory[self.ctx.program_counter],
            self.memory[self.ctx.program_counter + 1],
        ];
        self.ctx.program_counter += 2;

        Chip8Instruction::decode(instruction)
    }

    pub fn execute(&mut self, instruction: Chip8Instruction) {
        match instruction {
            Chip8Instruction::Unimplemented => {
                panic!("FATAL ERROR: Unimplemented instruction attempted to execute")
            }
            Chip8Instruction::Terminate => {
                self.ctx.dump();
                self.is_running = false;
            }
            Chip8Instruction::Panic(DebugPrintOperand(message)) => {
                self.ctx.dump();
                panic!("\n{}", message)
            }
            Chip8Instruction::Jump(AddressOperand(address)) => self.ctx.program_counter = address,
            Chip8Instruction::JumpPlusR0(AddressOperand(address)) => {
                if address + self.ctx.r[0] as usize >= 0xfff {
                    self.ctx.dump();
                    panic!(
                        "Attempted BNNN, but result would have been greater or equal to 0xfff (0x{:x})",
                        address + self.ctx.r[0] as usize
                    );
                }
                self.ctx.program_counter = address + self.ctx.r[0] as usize
            }
            Chip8Instruction::Call(AddressOperand(address)) => {
                self.ctx.push_return_address();
                self.ctx.program_counter = address;
            }
            Chip8Instruction::Ret => {
                self.ctx.program_counter = self.ctx.pop_return_address();
            }
            Chip8Instruction::SkipIfEqual(RegisterValueOperands { register, value }) => {
                let vals: (u8, u8) = (
                    self.ctx.r[register as usize],
                    match value {
                        Source::Register(n) => self.ctx.r[n as usize],
                        Source::Immediate(v) => v,
                    },
                );
                if vals.0 == vals.1 {
                    self.ctx.program_counter += 2
                }
            }
            Chip8Instruction::SkipIfNotEqual(operands) => {
                let vals: (u8, u8) = (
                    self.ctx.r[operands.register as usize],
                    match operands.value {
                        Source::Register(n) => self.ctx.r[n as usize],
                        Source::Immediate(v) => v,
                    },
                );
                if vals.0 != vals.1 {
                    self.ctx.program_counter += 2
                }
            }
            Chip8Instruction::Store(RegisterValueOperands { register, value }) => {
                self.ctx.r[register as usize] = match value {
                    Source::Register(n) => self.ctx.r[n as usize],
                    Source::Immediate(v) => v,
                };
            }
            Chip8Instruction::Add(RegisterValueOperands { register, value }) => {
                let lhs = self.ctx.r[register as usize];
                let rhs = match value {
                    Source::Register(n) => self.ctx.r[n as usize],
                    Source::Immediate(v) => v,
                };

                self.ctx.r[register as usize] = lhs.wrapping_add(rhs);

                self.ctx.r[0xF] = if lhs as u16 + rhs as u16 > 255 { 1 } else { 0 };
            }
            Chip8Instruction::Sub(RegisterValueOperands { register, value }) => {
                let lhs = self.ctx.r[register as usize];
                let rhs = match value {
                    Source::Register(n) => self.ctx.r[n as usize],
                    Source::Immediate(v) => v,
                };

                self.ctx.r[register as usize] = lhs.wrapping_sub(rhs);

                self.ctx.r[0xF] = (lhs >= rhs).into();
            }
            Chip8Instruction::SubTo(RegisterValueOperands { register, value }) => {
                let lhs = self.ctx.r[register as usize];
                let rhs = match value {
                    Source::Register(n) => self.ctx.r[n as usize],
                    Source::Immediate(v) => v,
                };

                self.ctx.r[register as usize] = rhs.wrapping_sub(lhs);

                self.ctx.r[0xF] = (lhs <= rhs).into();
            }
            Chip8Instruction::OR(RegisterValueOperands { register, value }) => {
                self.ctx.r[register as usize] |= match value {
                    Source::Register(n) => self.ctx.r[n as usize],
                    Source::Immediate(v) => v,
                };
            }
            Chip8Instruction::AND(RegisterValueOperands { register, value }) => {
                self.ctx.r[register as usize] &= match value {
                    Source::Register(n) => self.ctx.r[n as usize],
                    Source::Immediate(v) => v,
                };
            }
            Chip8Instruction::XOR(RegisterValueOperands { register, value }) => {
                self.ctx.r[register as usize] ^= match value {
                    Source::Register(n) => self.ctx.r[n as usize],
                    Source::Immediate(v) => v,
                };
            }
            Chip8Instruction::BitShiftLeftOnce(RegisterValueOperands { register, value: _ }) => {
                let overflow: u8 = self.ctx.r[register as usize] >> 7;

                self.ctx.r[register as usize] <<= 1;

                self.ctx.r[0xf] = overflow;
            }
            Chip8Instruction::BitShiftRightOnce(RegisterValueOperands { register, value: _ }) => {
                let underflow: u8 = self.ctx.r[register as usize] & 1;

                self.ctx.r[register as usize] >>= 1;

                self.ctx.r[0xf] = underflow;
            }
            Chip8Instruction::SaveAddress(AddressOperand(address)) => {
                self.ctx.i = address;
            }
            Chip8Instruction::GetRandom(RegisterValueOperands { register, value }) => {
                self.ctx.r[register as usize] = macroquad::rand::rand() as u8
                    & match value {
                        Source::Register(n) => self.ctx.r[n as usize],
                        Source::Immediate(v) => v,
                    };
            }
            Chip8Instruction::ClearScreen => {
                self.display.clear();
            }
            Chip8Instruction::DrawSprite(DrawSpriteOperands {
                x_register,
                y_register,
                height,
            }) => {
                self.ctx.r[0xf] = self.display.draw_sprite(
                    self.ctx.r[x_register as usize].into(),
                    self.ctx.r[y_register as usize].into(),
                    &Chip8Sprite::new(&self.memory[self.ctx.i..][..height as usize]),
                );
            }
            Chip8Instruction::SkipIfKeyPressed(SingleRegisterOperand { register }) => {
                if hexpad_get_keypress(self.ctx.r[register as usize]) {
                    self.ctx.program_counter += 2;
                }
            }
            Chip8Instruction::SkipIfKeyNotPressed(SingleRegisterOperand { register }) => {
                if !hexpad_get_keypress(self.ctx.r[register as usize]) {
                    self.ctx.program_counter += 2;
                }
            }
            Chip8Instruction::WaitForKeyPress(SingleRegisterOperand { register }) => {
                self.ctx.r[register as usize] = hexpad_wait_keypress();
            }
            Chip8Instruction::GetDelayTimer(SingleRegisterOperand { register }) => {
                if self.ctx.delay_timer.is_sign_negative() {
                    self.ctx.r[register as usize] = 0;
                } else {
                    self.ctx.r[register as usize] = self.ctx.delay_timer as u8;
                }
            }
            Chip8Instruction::SetDelayTimer(SingleRegisterOperand { register }) => {
                self.ctx.delay_timer = self.ctx.r[register as usize] as f32;
            }
            Chip8Instruction::SetSoundTimer(SingleRegisterOperand { register }) => {
                self.ctx.sound_timer = self.ctx.r[register as usize];
            }

            Chip8Instruction::AddToAddress(SingleRegisterOperand { register }) => {
                self.ctx.i = self.ctx.i + self.ctx.r[register as usize] as usize;
            }
            Chip8Instruction::SaveSpriteDigitAddress(SingleRegisterOperand { register }) => {
                self.ctx.i = register as usize * 5;
            }
            Chip8Instruction::StoreBinaryCodedDecimal(SingleRegisterOperand { register }) => {
                let binary_value = self.ctx.r[register as usize];

                let hundreds_digit = binary_value / 100;
                let tens_digit = (binary_value - hundreds_digit * 100) / 10;
                let ones_digit = binary_value - hundreds_digit * 100 - tens_digit * 10;

                self.memory[self.ctx.i] = hundreds_digit;
                self.memory[self.ctx.i + 1] = tens_digit;
                self.memory[self.ctx.i + 2] = ones_digit;
            }
            Chip8Instruction::StoreRegistersIntoMemoryChunk(SingleRegisterOperand { register }) => {
                for n in 0..=register {
                    self.memory[self.ctx.i + n as usize] = self.ctx.r[n as usize];
                }
            }
            Chip8Instruction::StoreMemoryChunkIntoRegisters(SingleRegisterOperand { register }) => {
                for n in 0..=register {
                    self.ctx.r[n as usize] = self.memory[self.ctx.i + n as usize];
                }
            }
            //_ => panic!("Unknown Instruction Encountered! {:?}", instruction),
        }
    }

    pub fn emulation_cycle(&mut self) {
        if self.is_running {
            let instruction: Chip8Instruction = self.fetch();
            if let Chip8Instruction::GetDelayTimer(_) = &instruction {
                println!("{:?}", self.ctx.delay_timer);
                //self.ctx.dump();
            }

            self.execute(instruction);
        }
    }

    pub fn get_display(&self) -> &Chip8Display {
        &self.display
    }
}

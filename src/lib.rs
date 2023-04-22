use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::collections::HashSet;

// The audio code is pretty much lifted 1:1 from the SDL2 crate's audio example code: https://rust-sdl2.github.io/rust-sdl2/sdl2/audio/index.html
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct ChipEight {
    // Chip-8 has access to 4KiB RAM. Most programs start at 0x200, as bytes 0x000 to 0x1FF are reserved for the interpreter.
    memory: [u8; 4096],
    // Chip-8 has a 64x32 monochrome screen. Some later versions had higher resolution displays and color support though.
    screen: [[bool; 64]; 32],
    // Chip-8 has a stack that can store up to 16 addresses that the interpreter should return to when a subroutine has finished executing.
    stack: Vec<u16>,
    // Chip-8 has 16 general-purpose 8-bit registers V0 - VF, although VF is used as a flag by some instructions and should not be used by programs.
    v_registers: [u8; 16],
    // The following are special registers that are separated distinctly from the general-purpose registers
    // The program counter is a 16-bit register that stores the currently executing address
    pc: u16,
    // The stack pointer is an 8-bit register that points to the topmost level of the stack
    sp: u8,
    // The I register stores memory addresses. Since there's only 4KiB (0xFFF) RAM, only the lowest 12 bits are used.
    i_register: u16,
    // When greater than 0, the delay timer will decrement by 1 every cycle
    delay_timer: u8,
    // When greater than 0, the sound timer will decrement by 1 every cycle and play a tone (in this case, a square wave middle C note)
    sound_timer: u8,
}

// For the sake of my sanity and my fingers, I'm typing these as hexadecimal values, but their binary representation shows an 8x5 sprite of the number at the given index (i.e., SPRITES[0x0] is the sprite for the number 0)
// A complete table with corresponding binary and hexadecimal values can be found here: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.4
static SPRITES: [[u8; 5]; 16] = [
    // Zero (0)
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    // One (1)
    [0x20, 0x60, 0x20, 0x20, 0x70],
    // Two (2)
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    // Three (3)
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    // Four (4)
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    // Five (5)
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    // Six (6)
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    // Seven (7)
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    // Eight (8)
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    // Nine (9)
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    // A
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    // B
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    // C
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    // D
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    // E
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    // F
    [0xF0, 0x80, 0xF0, 0x80, 0x80],
];

impl ChipEight {
    pub fn new() -> Self {
        ChipEight {
            memory: Self::init_memory(SPRITES),
            screen: [[false; 64]; 32],
            stack: Vec::<u16>::with_capacity(16),
            v_registers: [0; 16],
            pc: 0x200,
            sp: 0,
            i_register: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
    fn init_memory(sprites: [[u8; 5]; 16]) -> [u8; 4096] {
        let mut memory: [u8; 4096] = [0; 4096];
        for (i, sprite) in sprites.iter().enumerate() {
            for (j, byte) in sprite.iter().enumerate() {
                let current_sprite: usize = i * sprite.len();
                memory[current_sprite + j] = *byte;
            }
        }
        memory
    }
    fn poll_input(pressed: &mut HashSet<u8>, event_pump: &mut sdl2::EventPump) -> i32 {
        use sdl2::{event::Event, keyboard::Keycode};

        let mut last_pressed = -1;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return 0x1B,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => {
                        pressed.insert(0x1);
                        last_pressed = 0x1;
                    }
                    Keycode::Num2 => {
                        pressed.insert(0x2);
                        last_pressed = 0x2;
                    }
                    Keycode::Num3 => {
                        pressed.insert(0x3);
                        last_pressed = 0x3;
                    }
                    Keycode::Num4 => {
                        pressed.insert(0xC);
                        last_pressed = 0xC;
                    }
                    Keycode::Q => {
                        pressed.insert(0x4);
                        last_pressed = 0x4;
                    }
                    Keycode::W => {
                        pressed.insert(0x5);
                        last_pressed = 0x5;
                    }
                    Keycode::E => {
                        pressed.insert(0x6);
                        last_pressed = 0x6;
                    }
                    Keycode::R => {
                        pressed.insert(0xD);
                        last_pressed = 0xD;
                    }
                    Keycode::A => {
                        pressed.insert(0x7);
                        last_pressed = 0x7;
                    }
                    Keycode::S => {
                        pressed.insert(0x8);
                        last_pressed = 0x8;
                    }
                    Keycode::D => {
                        pressed.insert(0x9);
                        last_pressed = 0x9;
                    }
                    Keycode::F => {
                        pressed.insert(0xE);
                        last_pressed = 0xE;
                    }
                    Keycode::Z => {
                        pressed.insert(0xA);
                        last_pressed = 0xA;
                    }
                    Keycode::X => {
                        pressed.insert(0x0);
                        last_pressed = 0x0;
                    }
                    Keycode::C => {
                        pressed.insert(0xB);
                        last_pressed = 0xB;
                    }
                    Keycode::V => {
                        pressed.insert(0xF);
                        last_pressed = 0xF;
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => {
                        pressed.remove(&0x1);
                    }
                    Keycode::Num2 => {
                        pressed.remove(&0x2);
                    }
                    Keycode::Num3 => {
                        pressed.remove(&0x3);
                    }
                    Keycode::Num4 => {
                        pressed.remove(&0xC);
                    }
                    Keycode::Q => {
                        pressed.remove(&0x4);
                    }
                    Keycode::W => {
                        pressed.remove(&0x5);
                    }
                    Keycode::E => {
                        pressed.remove(&0x6);
                    }
                    Keycode::R => {
                        pressed.remove(&0xD);
                    }
                    Keycode::A => {
                        pressed.remove(&0x7);
                    }
                    Keycode::S => {
                        pressed.remove(&0x8);
                    }
                    Keycode::D => {
                        pressed.remove(&0x9);
                    }
                    Keycode::F => {
                        pressed.remove(&0xE);
                    }
                    Keycode::Z => {
                        pressed.remove(&0xA);
                    }
                    Keycode::X => {
                        pressed.remove(&0x0);
                    }
                    Keycode::C => {
                        pressed.remove(&0xB);
                    }
                    Keycode::V => {
                        pressed.remove(&0xF);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        last_pressed
    }
    pub fn load_program(&mut self, program: Vec<u8>) {
        use std::collections::VecDeque;

        let mut prog_queue: VecDeque<u8> = VecDeque::from(program);
        let mut mem_idx: usize = 0x200;
        while prog_queue.len() != 0 {
            let byte: u8 = match prog_queue.pop_front() {
                Some(b) => b,
                None => 0,
            };

            self.memory[mem_idx] = byte;
            mem_idx += 1;
        }
    }
    pub fn run(
        &mut self,
        mut canvas: sdl2::render::Canvas<sdl2::video::Window>,
        sdl_context: sdl2::Sdl,
    ) {
        use sdl2::{pixels::Color, rect::Rect};

        let audio_subsystem = match sdl_context.audio() {
            Ok(audio) => {
                println!("Created sdl audio!");
                audio
            }
            Err(e) => panic!("Error creating sdl audiocontext: {:?}", e),
        };

        // Set up the audio subsystem with 44.1KHz mono playback
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let audio_device =
            match audio_subsystem.open_playback(None, &desired_spec, |spec| SquareWave {
                phase_inc: 261.63 / spec.freq as f32, // middle C note
                phase: 0.0,
                volume: 0.0625,
            }) {
                Ok(audio) => {
                    println!("Initialized audio device with a square wave!");
                    audio
                }
                Err(e) => panic!("Error initializing audio device: {:?}", e),
            };

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        match canvas.set_logical_size(64, 32) {
            Ok(_) => {}
            Err(e) => panic!("Error setting canvas logical size: {:?}", e),
        };
        canvas.clear();
        canvas.present();

        let mut event_pump = match sdl_context.event_pump() {
            Ok(pump) => pump,
            Err(e) => panic!("Error creating sdl context event pump: {:?}", e),
        };

        let mut pressed: HashSet<u8> = HashSet::new();
        'running: loop {
            for (y, row) in self.screen.iter().enumerate() {
                for (x, pixel) in row.iter().enumerate() {
                    let rect = Rect::new(x as i32, y as i32, 1, 1);
                    if *pixel {
                        canvas.set_draw_color(Color::RGB(255, 255, 255));
                    } else {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                    }
                    match canvas.draw_rect(rect) {
                        Ok(_) => {}
                        Err(e) => println!("Error drawing rectangle at ({}, {}): {:?}", x, y, e),
                    };
                }
            }

            let key = Self::poll_input(&mut pressed, &mut event_pump);

            if key == 0x1B {
                break 'running;
            }

            let instruction: u16 = (self.memory[self.pc as usize] as u16) << 8
                | self.memory[(self.pc + 1) as usize] as u16;

            self.sound_timer = if self.sound_timer > 0 {
                audio_device.resume();
                self.sound_timer - 1
            } else {
                audio_device.pause();
                0
            };

            self.delay_timer = if self.delay_timer > 0 {
                self.delay_timer - 1
            } else {
                0
            };

            self.execute(instruction, &mut pressed, &mut event_pump);
            canvas.present();
        }
    }
    fn execute(
        &mut self,
        instruction: u16,
        pressed: &mut HashSet<u8>,
        event_pump: &mut sdl2::EventPump,
    ) {
        let top_nybble: u16 = instruction >> 12;
        // These are usize because the second and third nybbles are pretty much exclusively used to access registers Vx and Vy respectively
        let second_nybble: usize = ((instruction & 0x0F00) >> 8) as usize;
        let third_nybble: usize = ((instruction & 0x00F0) >> 4) as usize;
        let bottom_nybble: u16 = instruction & 0x000F;

        let bottom_byte: u8 = (instruction & 0x00FF) as u8;
        let bottom_three_nybbles: u16 = instruction & 0x0FFF;

        match top_nybble {
            0x0 => match bottom_byte {
                0xE0 => self.clear_screen(),
                0xEE => self.return_from_subroutine(),
                _ => self.jump_to_machine_code(),
            },
            0x1 => self.jump_to_address(bottom_three_nybbles),
            0x2 => self.call_subroutine_at_address(bottom_three_nybbles),
            0x3 => self.skip_if_vx_equals_data(second_nybble, bottom_byte),
            0x4 => self.skip_if_vx_not_equals_data(second_nybble, bottom_byte),
            0x5 => self.skip_if_vx_equals_vy(second_nybble, third_nybble),
            0x6 => self.set_vx_equals_data(second_nybble, bottom_byte),
            0x7 => self.add_assign_data_to_vx(second_nybble, bottom_byte),
            0x8 => match bottom_nybble {
                0x0 => self.set_vx_equals_vy(second_nybble, third_nybble),
                0x1 => self.bitor_assign_vy_to_vx(second_nybble, third_nybble),
                0x2 => self.bitand_assign_vy_to_vx(second_nybble, third_nybble),
                0x3 => self.bitxor_assign_vy_to_vx(second_nybble, third_nybble),
                0x4 => self.add_assign_vy_to_vx(second_nybble, third_nybble),
                0x5 => self.sub_assign_vy_to_vx(second_nybble, third_nybble),
                0x6 => self.shift_right_vx(second_nybble, third_nybble),
                0x7 => self.sub_vx_from_vy(second_nybble, third_nybble),
                0xE => self.shift_left_vx(second_nybble, third_nybble),
                _ => panic!("Invalid instruction {:#04x} encountered.", instruction),
            },
            0x9 => self.skip_if_vx_not_equals_vy(second_nybble, third_nybble),
            0xA => self.set_i_to_address(bottom_three_nybbles),
            0xB => self.jump_to_address_plus_v0(bottom_three_nybbles),
            0xC => self.set_vx_equals_rand(second_nybble, bottom_byte),
            0xD => self.draw_n_bytes_at_xy(second_nybble, third_nybble, bottom_nybble),
            0xE => match bottom_byte {
                0x9E => self.skip_if_vx_pressed(second_nybble, pressed),
                0xA1 => self.skip_if_vx_not_pressed(second_nybble, pressed),
                _ => panic!("Invalid instruction {:#04x} encountered.", instruction),
            },
            0xF => match bottom_byte {
                0x07 => self.set_vx_equals_delay(second_nybble),
                0x0A => self.set_vx_equals_key(second_nybble, pressed, event_pump),
                0x15 => self.set_delay_equals_vx(second_nybble),
                0x18 => self.set_sound_equals_vx(second_nybble),
                0x1E => self.add_assign_vx_to_i(second_nybble),
                0x29 => self.set_i_to_sprite(second_nybble),
                0x33 => self.set_i_to_bcd(second_nybble),
                0x55 => self.store_v_registers(second_nybble),
                0x65 => self.restore_v_registers(second_nybble),
                _ => panic!("Invalid instruction {:#04x} encountered.", instruction),
            },
            _ => unreachable!(
                "Somehow encountered an instruction where the top nybble is greater than 0xF????"
            ),
        }
    }
    // The following functions have very ugly names. They're named after the actual instruction + parameters. Sorry.
    // 0nnn - Jumps to machine code routine at address nnn. Ignored by modern interpreters
    fn jump_to_machine_code(&mut self) {
        // Do nothing
        self.pc += 2;
    }
    // 00E0 - Clears the display
    fn clear_screen(&mut self) {
        self.screen = [[false; 64]; 32];
        self.pc += 2;
    }
    // 00EE - Returns from a subroutine. Sets program counter to address at the top of the stack and subtracts 1 from the stack pointer
    fn return_from_subroutine(&mut self) {
        self.pc = match self.stack.last() {
            Some(val) => *val,
            None => panic!(),
        };
        self.stack.pop();
        self.sp -= 1;
        self.pc += 2;
    }
    // 1nnn - Jumps to address nnn. Sets program counter equal to nnn.
    fn jump_to_address(&mut self, address: u16) {
        self.pc = address;
    }
    // 2nnn - Calls subroutine at nnn. Increments the stack pointer, puts the current program counter on top of the stack, then sets the program counter to nnn.
    fn call_subroutine_at_address(&mut self, address: u16) {
        self.stack.push(self.pc);
        self.sp += 1;
        self.pc = address;
    }
    // 3xkk - Skips the next instruction if Vx == kk. Increments the program counter by 2.
    fn skip_if_vx_equals_data(&mut self, x: usize, data: u8) {
        self.pc += if self.v_registers[x] == data { 4 } else { 2 };
    }
    // 4xkk - Skips the next instruction if Vx != kk. Increments the program counter by 2.
    fn skip_if_vx_not_equals_data(&mut self, x: usize, data: u8) {
        self.pc += if self.v_registers[x] != data { 4 } else { 2 };
    }
    // 5xy0 - Skips the next instruction if Vx == Vy. Increments the program counter by 2.
    fn skip_if_vx_equals_vy(&mut self, x: usize, y: usize) {
        self.pc += if self.v_registers[x] == self.v_registers[y] {
            4
        } else {
            2
        };
    }
    // 6xkk - Sets Vx = kk.
    fn set_vx_equals_data(&mut self, x: usize, data: u8) {
        self.v_registers[x] = data;
        self.pc += 2;
    }
    // 7xkk - Sets Vx = Vx + kk.
    fn add_assign_data_to_vx(&mut self, x: usize, data: u8) {
        self.v_registers[x] += data;
        self.pc += 2;
    }
    // 8xy0 - Sets Vx = Vy.
    fn set_vx_equals_vy(&mut self, x: usize, y: usize) {
        self.v_registers[x] = self.v_registers[y];
        self.pc += 2;
    }
    // 8xy1 - Sets Vx = Vx | Vy.
    fn bitor_assign_vy_to_vx(&mut self, x: usize, y: usize) {
        self.v_registers[x] |= self.v_registers[y];
        self.pc += 2;
    }
    // 8xy2 - Sets Vx = Vx & Vy.
    fn bitand_assign_vy_to_vx(&mut self, x: usize, y: usize) {
        self.v_registers[x] &= self.v_registers[y];
        self.pc += 2;
    }
    // 8xy3 - Sets Vx = Vx ^ Vy.
    fn bitxor_assign_vy_to_vx(&mut self, x: usize, y: usize) {
        self.v_registers[x] ^= self.v_registers[y];
        self.pc += 2;
    }
    // 8xy4 - Sets Vx = Vx + Vy. Also sets VF = 1 if a carry flag is needed.
    fn add_assign_vy_to_vx(&mut self, x: usize, y: usize) {
        let f: usize = 0xF;
        let sum: u16 = self.v_registers[x] as u16 + self.v_registers[y] as u16;

        self.v_registers[f] = if sum > 255 { 1 } else { 0 };
        // We only need the lower byte, so just mask it.
        self.v_registers[x] = (sum & 0x00FF) as u8;
        self.pc += 2;
    }
    // 8xy5 - Sets Vx = Vx - Vy. If Vx > Vy, set VF to 1, otherwise set VF to 0.
    fn sub_assign_vy_to_vx(&mut self, x: usize, y: usize) {
        let f: usize = 0xF;

        self.v_registers[x] -= self.v_registers[y];

        self.v_registers[f] = if self.v_registers[x] > self.v_registers[y] {
            1
        } else {
            0
        };

        self.pc += 2;
    }
    // 8xy6 - Sets Vx = Vx >> 1 (equivalent to Vx / 2). If the least significant bit of Vx == 1, set VF = 1.
    fn shift_right_vx(&mut self, x: usize, _y: usize) {
        let f: usize = 0xF;
        let prev: u8 = self.v_registers[x] & 0x0001;

        self.v_registers[x] >>= 1;

        self.v_registers[f] = if prev == 1 { 1 } else { 0 };

        self.pc += 2;
    }
    // 8xy7 - Sets Vx = Vy - Vx. If Vy > Vx, set VF to 1, otherwise set VF to 0.
    fn sub_vx_from_vy(&mut self, x: usize, y: usize) {
        let f: usize = 0xF;

        self.v_registers[x] = self.v_registers[y] - self.v_registers[x];

        self.v_registers[f] = if self.v_registers[y] > self.v_registers[x] {
            1
        } else {
            0
        };

        self.pc += 2;
    }
    // 8xyE - Sets Vx = Vx << 1 (Equivalent to Vx * 2). If the most significant bit of Vx == 1, set VF = 1.
    fn shift_left_vx(&mut self, x: usize, _y: usize) {
        let f: usize = 0xF;
        let prev: u8 = self.v_registers[x] & 0x80;

        self.v_registers[x] <<= 1;

        self.v_registers[f] = if prev != 0 { 1 } else { 0 };

        self.pc += 2;
    }
    // 9xy0 - Skips the next instruction if Vx != Vy.
    fn skip_if_vx_not_equals_vy(&mut self, x: usize, y: usize) {
        self.pc += if self.v_registers[x] != self.v_registers[y] {
            4
        } else {
            2
        };
    }
    // Annn - Sets register I equal to nnn.
    fn set_i_to_address(&mut self, address: u16) {
        self.i_register = address;
        self.pc += 2;
    }
    // Bnnn - Sets program counter equal to nnn + V0
    fn jump_to_address_plus_v0(&mut self, address: u16) {
        self.pc = address + self.v_registers[0] as u16;
    }
    // Cxkk - Sets Vx = kk & random byte.
    fn set_vx_equals_rand(&mut self, x: usize, data: u8) {
        let rand: u8 = rand::random();

        self.v_registers[x] = data & rand;
        self.pc += 2;
    }
    // This function is particularly ugly. Sorry.
    // Dxyn - Display an n-byte sprite starting at memory location I at coordinate (Vx, Vy) and set VF = collision
    fn draw_n_bytes_at_xy(&mut self, x: usize, y: usize, n: u16) {
        let f: usize = 0xF;
        let mut collision: bool = false;
        let sprite_size: usize = (self.i_register + n) as usize;
        let sprite_slice: &[u8] = &self.memory[self.i_register as usize..sprite_size];
        let mut sprite: Vec<[bool; 8]> = vec![]; // We need to use a vector because the value of n isn't known at compile time

        for &byte in sprite_slice {
            // There is almost certainly a less ugly way to do this.
            // We're just bitmasking all 8 bits and checking to see if the resulting value isn't 0.
            let byte_array: [bool; 8] = [
                (byte & 0b10000000) != 0,
                (byte & 0b01000000) != 0,
                (byte & 0b00100000) != 0,
                (byte & 0b00010000) != 0,
                (byte & 0b00001000) != 0,
                (byte & 0b00000100) != 0,
                (byte & 0b00000010) != 0,
                (byte & 0b00000001) != 0,
            ];
            sprite.push(byte_array);
        }

        for i in 0..sprite.len() {
            // If a sprite's coordinates on screen go past the screen boundaries, the sprite should wrap to the other side.
            let sy: usize = (self.v_registers[y] as usize + i) % 32;
            for j in 0..8 as usize {
                // Make sure to also wrap the x-axis.
                let sx: usize = (self.v_registers[x] as usize + j) % 64;
                let current_pixel: bool = self.screen[sy][sx];
                self.screen[sy][sx] ^= sprite[i][j];
                // If current_pixel is true and self.screen[sy][sx] is false, then a collision occurred.
                if current_pixel && !self.screen[sy][sx] {
                    collision = true;
                }
            }
        }
        self.v_registers[f] = if collision { 1 } else { 0 };
        self.pc += 2;
    }
    // Ex9E - Skip next instruction if key with the value of Vx is pressed.
    fn skip_if_vx_pressed(&mut self, x: usize, pressed: &HashSet<u8>) {
        self.pc += if pressed.contains(&self.v_registers[x]) {
            4
        } else {
            2
        };
    }
    // ExA1 - Skip next instruction if key with the value of Vx is not pressed.
    fn skip_if_vx_not_pressed(&mut self, x: usize, pressed: &HashSet<u8>) {
        self.pc += if !pressed.contains(&self.v_registers[x]) {
            4
        } else {
            2
        };
    }
    // Fx07 - Set Vx = delay_timer.
    fn set_vx_equals_delay(&mut self, x: usize) {
        self.v_registers[x] = self.delay_timer;
        self.pc += 2;
    }
    // Fx0A - Wait for a key press, then store the value of the key in Vx.
    fn set_vx_equals_key(
        &mut self,
        x: usize,
        pressed: &mut HashSet<u8>,
        event_pump: &mut sdl2::EventPump,
    ) {
        let key = loop {
            let key = Self::poll_input(pressed, event_pump);

            if key == 0x1B {
                // This probably isn't the best idea but oh well ¯\_(ツ)_/¯
                std::process::exit(0);
            } else if !(key == -1) {
                break key;
            }
        };

        self.v_registers[x] = key as u8;
        self.pc += 2;
    }
    // Fx15 - Set delay_timer = Vx.
    fn set_delay_equals_vx(&mut self, x: usize) {
        self.delay_timer = self.v_registers[x];
        self.pc += 2;
    }
    // Fx18 - Set sound_timer = Vx.
    fn set_sound_equals_vx(&mut self, x: usize) {
        self.sound_timer = self.v_registers[x];
        self.pc += 2;
    }
    // Fx1E - Set I = I + Vx.
    fn add_assign_vx_to_i(&mut self, x: usize) {
        self.i_register += self.v_registers[x] as u16;
        self.pc += 2;
    }
    // Fx29 - Set I to the location of the hexadecimal sprite corresponding to the value of Vx.
    fn set_i_to_sprite(&mut self, x: usize) {
        // The hexadecimal sprites are 8x5, so we multiply the value of Vx by 5 to get the index of the sprite
        let i: u16 = (self.v_registers[x] * 5) as u16;

        self.i_register = i;
        self.pc += 2;
    }
    // Fx33 - Store the BCD representation of Vx in I, I+1, and I+2. The hundreds place is stored in I, tens in I+1, and ones in I+2.
    fn set_i_to_bcd(&mut self, x: usize) {
        let hundreds: u8 = self.v_registers[x] / 100;
        let tens: u8 = (self.v_registers[x] / 10) % 10;
        let ones: u8 = self.v_registers[x] % 10;
        let idx: usize = self.i_register as usize;

        self.memory[idx] = hundreds;
        self.memory[idx + 1] = tens;
        self.memory[idx + 2] = ones;
        self.pc += 2;
    }
    // Fx55 - Store the values in registers V0 - Vx in memory starting at location I.
    fn store_v_registers(&mut self, x: usize) {
        let idx: usize = self.i_register as usize;

        for i in 0..=x {
            self.memory[idx + i] = self.v_registers[i];
        }
        self.pc += 2;
    }
    // Fx65 - Read values from memory starting at location I and store them in registers V0 - Vx.
    fn restore_v_registers(&mut self, x: usize) {
        let idx: usize = self.i_register as usize;

        for i in 0..=x {
            self.v_registers[i] = self.memory[idx + i];
        }
        self.pc += 2;
    }
}

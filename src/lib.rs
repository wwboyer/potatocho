pub struct ChipEight {
    // Chip-8 has access to 4KiB RAM. Most programs start at 0x200, as bytes 0x000 to 0x1FF are reserved for the interpreter.
    memory: [u8; 4096],
    // Chip-8 has a 64x32 monochrome screen. Some later versions had higher resolution displays and color support though.
    screen: [[bool; 64]; 32],
    // Chip-8 has a stack that can store up to 16 addresses that the interpreter should return to when a subroutine has finished executing.
    stack: Vec<u16>,
    // Chip-8 has 16 general-purpose 8-bit registers V0 - VF, although VF is used as a flag by some instructions and should not be used by programs.
    gp_registers: [u8; 16],
    // The following are special registers that are separated distinctly from the general-purpose registers
    // The program counter is a 16-bit register that stores the currently executing address
    pc: u16,
    // The stack pointer is an 8-bit register that points to the topmost level of the stack
    sp: u8,
    // The I register stores memory addresses. Since there's only 4KiB (0xFFF) RAM, only the lowest 12 bits are used.
    i_register: u16,
    delay_timer: u8,
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
            gp_registers: [0; 16],
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
    pub fn run(&mut self, mut canvas: sdl2::render::Canvas<sdl2::video::Window>, sdl_context: sdl2::Sdl) {
        use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        match canvas.set_logical_size(64, 32) {
            Ok(_) => {},
            Err(e) => panic!("Error setting canvas logical size: {:?}", e),
        };
        canvas.clear();
        canvas.present();

        let mut event_pump = match sdl_context.event_pump() {
            Ok(pump) => pump,
            Err(e) => panic!("Error creating sdl context event pump: {:?}", e),
        };

        let mut pressed: u8 = 0x10;
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
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => match keycode {
                        Keycode::Num1 => {
                            pressed = 0x1;
                        }
                        Keycode::Num2 => {
                            pressed = 0x2;
                        }
                        Keycode::Num3 => {
                            pressed = 0x3;
                        }
                        Keycode::Num4 => {
                            pressed = 0xC;
                        }
                        Keycode::Q => {
                            pressed = 0x4;
                        }
                        Keycode::W => {
                            pressed = 0x5;
                        }
                        Keycode::E => {
                            pressed = 0x6;
                        }
                        Keycode::R => {
                            pressed = 0xD;
                        }
                        Keycode::A => {
                            pressed = 0x7;
                        }
                        Keycode::S => {
                            pressed = 0x8;
                        }
                        Keycode::D => {
                            pressed = 0x9;
                        }
                        Keycode::F => {
                            pressed = 0xE;
                        }
                        Keycode::Z => {
                            pressed = 0xA;
                        }
                        Keycode::X => {
                            pressed = 0x0;
                        }
                        Keycode::C => {
                            pressed = 0xB;
                        }
                        Keycode::V => {
                            pressed = 0xF;
                        }
                        _ => {}
                    },
                    Event::KeyUp {..} => {
                        pressed = 0x10;
                    }
                    _ => {}
                }
            }

            let instruction: u16 = (self.memory[self.pc as usize] as u16) << 8
                | self.memory[(self.pc + 1) as usize] as u16;

            self.execute(instruction, pressed);
            canvas.present();
        }
    }
    fn execute(&mut self, instruction: u16, pressed: u8) {
        let top_nybble: u16 = instruction >> 12;
        let bottom_byte: u16 = instruction & 0x00FF;
        let bottom_nybble: u16 = instruction & 0x000F;

        // println!("Current instruction: {:#04x}", instruction);
        // println!("Current pc: {:#04x}", self.pc);

        match top_nybble {
            0x0 => match bottom_byte {
                0xE0 => self.cls(instruction),
                0xEE => self.ret(instruction),
                _ => self.sys(instruction),
            },
            0x1 => self.jmp(instruction),
            0x2 => self.call(instruction),
            0x3 => self.se_vx_kk(instruction),
            0x4 => self.sne_vx_kk(instruction),
            0x5 => self.se_vx_vy(instruction),
            0x6 => self.ld_vx_kk(instruction),
            0x7 => self.add_vx_kk(instruction),
            0x8 => match bottom_nybble {
                0x0 => self.ld_vx_vy(instruction),
                0x1 => self.or_vx_vy(instruction),
                0x2 => self.and_vx_vy(instruction),
                0x3 => self.xor_vx_vy(instruction),
                0x4 => self.add_vx_vy(instruction),
                0x5 => self.sub_vx_vy(instruction),
                0x6 => self.shr_vx_vy(instruction),
                0x7 => self.subn_vx_vy(instruction),
                0xE => self.shl_vx_vy(instruction),
                _ => panic!("Invalid instruction {:#04x} encountered.", instruction),
            },
            0x9 => self.sne_vx_vy(instruction),
            0xA => self.ld_i_nnn(instruction),
            0xB => self.jmp_v0_nnn(instruction),
            0xC => self.rnd_vx_kk(instruction),
            0xD => self.drw_vx_vy_n(instruction),
            0xE => match bottom_byte {
                0x9E => self.skp_vx(instruction, pressed),
                0xA1 => self.sknp_vx(instruction, pressed),
                _ => panic!("Invalid instruction {:#04x} encountered.", instruction),
            },
            0xF => match bottom_byte {
                0x07 => self.ld_vx_dt(instruction),
                0x0A => self.ld_vx_key(instruction, pressed),
                0x15 => self.ld_dt_vx(instruction),
                0x18 => self.ld_st_vx(instruction),
                0x1E => self.add_i_vx(instruction),
                0x29 => self.ld_f_vx(instruction),
                0x33 => self.ld_b_vx(instruction),
                0x55 => self.ld_i_vx(instruction),
                0x65 => self.ld_vx_i(instruction),
                _ => panic!("Invalid instruction {:#04x} encountered.", instruction),
            },
            _ => unreachable!(
                "Somehow encountered an instruction where the top nybble is greater than 0xF????"
            ),
        }
    }
    // The following functions have very ugly names. They're named after the actual instruction + parameters. Sorry.
    // 0nnn - Jumps to machine code routine at address nnn. Ignored by modern interpreters
    fn sys(&mut self, _instruction: u16) {
        // Do nothing
        self.pc += 2;
    }
    // 00E0 - Clears the display
    fn cls(&mut self, _instruction: u16) {
        self.screen = [[false; 64]; 32];
        self.pc += 2;
    }
    // 00EE - Returns from a subroutine. Sets program counter to address at the top of the stack and subtracts 1 from the stack pointer
    fn ret(&mut self, _instruction: u16) {
        self.pc = match self.stack.last() {
            Some(val) => *val,
            None => panic!(),
        };
        self.stack.pop();
        self.sp -= 1;
        self.pc += 2;
    }
    // 1nnn - Jumps to address nnn. Sets program counter equal to nnn.
    fn jmp(&mut self, instruction: u16) {
        self.pc = instruction & 0x0FFF;
    }
    // 2nnn - Calls subroutine at nnn. Increments the stack pointer, puts the current program counter on top of the stack, then sets the program counter to nnn.
    fn call(&mut self, instruction: u16) {
        let nnn = instruction & 0x0FFF;
        self.stack.push(self.pc);
        self.sp += 1;
        self.pc = nnn;
    }
    // 3xkk - Skips the next instruction if Vx == kk. Increments the program counter by 2.
    fn se_vx_kk(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let kk: u8 = (instruction & 0x00FF) as u8;

        if self.gp_registers[x] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
    // 4xkk - Skips the next instruction if Vx != kk. Increments the program counter by 2.
    fn sne_vx_kk(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let kk: u8 = (instruction & 0x00FF) as u8;

        if self.gp_registers[x] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
    // 5xy0 - Skips the next instruction if Vx == Vy. Increments the program counter by 2.
    fn se_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;

        if self.gp_registers[x] == self.gp_registers[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
    // 6xkk - Sets Vx = kk.
    fn ld_vx_kk(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let kk: u8 = (instruction & 0x00FF) as u8;

        self.gp_registers[x] = kk;
        self.pc += 2;
    }
    // 7xkk - Sets Vx = Vx + kk.
    fn add_vx_kk(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let kk: u8 = (instruction & 0x00FF) as u8;

        self.gp_registers[x] += kk;
        self.pc += 2;
    }
    // 8xy0 - Sets Vx = Vy.
    fn ld_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;

        self.gp_registers[x] = self.gp_registers[y];
        self.pc += 2;
    }
    // 8xy1 - Sets Vx = Vx | Vy.
    fn or_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;

        self.gp_registers[x] |= self.gp_registers[y];
        self.pc += 2;
    }
    // 8xy2 - Sets Vx = Vx & Vy.
    fn and_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;

        self.gp_registers[x] &= self.gp_registers[y];
        self.pc += 2;
    }
    // 8xy3 - Sets Vx = Vx ^ Vy.
    fn xor_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;

        self.gp_registers[x] ^= self.gp_registers[y];
        self.pc += 2;
    }
    // 8xy4 - Sets Vx = Vx + Vy. Also sets VF = 1 if a carry flag is needed.
    fn add_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;
        let f: usize = 0xF;

        let sum: u16 = self.gp_registers[x] as u16 + self.gp_registers[y] as u16;

        if sum > 255 {
            self.gp_registers[f] = 1;
        } else {
            self.gp_registers[f] = 0;
        }
        // We only need the lower byte, so just mask it.
        self.gp_registers[x] = (sum & 0x00FF) as u8;
        self.pc += 2;
    }
    // 8xy5 - Sets Vx = Vx - Vy. If Vx > Vy, set VF to 1, otherwise set VF to 0.
    fn sub_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;
        let f: usize = 0xF;

        self.gp_registers[x] -= self.gp_registers[y];

        if self.gp_registers[x] > self.gp_registers[y] {
            self.gp_registers[f] = 1;
        } else {
            self.gp_registers[f] = 0;
        }

        self.pc += 2;
    }
    // 8xy6 - Sets Vx = Vx >> 1 (equivalent to Vx / 2). If the least significant bit of Vx == 1, set VF = 1.
    fn shr_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let _y: usize = ((instruction & 0x00F0) >> 4) as usize; // Specified in the documentation but I'm pretty sure this is unused
        let f: usize = 0xF;
        let prev: u8 = self.gp_registers[x];

        self.gp_registers[x] >>= 1;

        if prev & 0x0001 == 1 {
            self.gp_registers[f] = 1;
        } else {
            self.gp_registers[f] = 0;
        }

        self.pc += 2;
    }
    // 8xy7 - Sets Vx = Vy - Vx. If Vy > Vx, set VF to 1, otherwise set VF to 0.
    fn subn_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;
        let f: usize = 0xF;

        self.gp_registers[x] = self.gp_registers[y] - self.gp_registers[x];

        if self.gp_registers[y] > self.gp_registers[x] {
            self.gp_registers[f] = 1;
        } else {
            self.gp_registers[f] = 0;
        }

        self.pc += 2;
    }
    // 8xyE - Sets Vx = Vx << 1 (Equivalent to Vx * 2). If the most significant bit of Vx == 1, set VF = 1.
    fn shl_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let _y: usize = ((instruction & 0x00F0) >> 4) as usize; // Again, the third nybble is specified but I'm pretty sure it's unused
        let f: usize = 0xF;
        let prev: u8 = self.gp_registers[x];

        self.gp_registers[x] <<= 1;

        if prev & 0x80 != 0 {
            self.gp_registers[f] = 1;
        } else {
            self.gp_registers[f] = 0;
        }

        self.pc += 2;
    }
    // 9xy0 - Skips the next instruction if Vx != Vy.
    fn sne_vx_vy(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;

        if self.gp_registers[x] != self.gp_registers[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
    // Annn - Sets register I equal to nnn.
    fn ld_i_nnn(&mut self, instruction: u16) {
        let nnn: u16 = instruction & 0x0FFF;

        self.pc += 2;
        self.i_register = nnn;
    }
    // Bnnn - Sets program counter equal to nnn + V0
    fn jmp_v0_nnn(&mut self, instruction: u16) {
        let nnn = instruction & 0x0FFF;
        self.pc = nnn + self.gp_registers[0] as u16;
    }
    // Cxkk - Sets Vx = kk & random byte.
    fn rnd_vx_kk(&mut self, instruction: u16) {
        let rand: u8 = rand::random();
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let kk: u8 = (instruction & 0x00FF) as u8;

        self.gp_registers[x] = kk & rand;
        self.pc += 2;
    }
    // This function is particularly ugly. Sorry.
    // Dxyn - Display an n-byte sprite starting at memory location I at coordinate (Vx, Vy) and set VF = collision
    fn drw_vx_vy_n(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let y: usize = ((instruction & 0x00F0) >> 4) as usize;
        let f: usize = 0xF;
        let n: u16 = instruction & 0x000F;
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
            let sy: usize = (self.gp_registers[y] as usize + i) % 32;
            for j in 0..8 as usize {
                // Make sure to also wrap the x-axis.
                let sx: usize = (self.gp_registers[x] as usize + j) % 64;
                let current_pixel: bool = self.screen[sy][sx];
                self.screen[sy][sx] ^= sprite[i][j];
                // If current_pixel is true and self.screen[sy][sx] is false, then a collision occurred.
                if current_pixel && !self.screen[sy][sx] {
                    collision = true;
                }
            }
        }
        self.gp_registers[f] = if collision { 1 } else { 0 };
        self.pc += 2;
    }
    // Ex9E - Skip next instruction if key with the value of Vx is pressed.
    fn skp_vx(&mut self, instruction: u16, pressed: u8) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;

        //println!("pressed = {}", pressed);
        if self.gp_registers[x] == pressed {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
    // ExA1 - Skip next instruction if key with the value of Vx is not pressed.
    fn sknp_vx(&mut self, instruction: u16, pressed: u8) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;

        if self.gp_registers[x] != pressed {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
    // Fx07 - Set Vx = delay_timer.
    fn ld_vx_dt(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;

        self.gp_registers[x] = self.delay_timer;
        self.pc += 2;
    }
    // Fx0A - Wait for a key press, then store the value of the key in Vx.
    fn ld_vx_key(&mut self, instruction: u16, pressed: u8) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;

        if pressed != 0x10 {
            self.gp_registers[x] = pressed;
            self.pc += 2;
        }
    }
    // Fx15 - Set delay_timer = Vx.
    fn ld_dt_vx(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;

        self.delay_timer = self.gp_registers[x];
        self.pc += 2;
    }
    // Fx18 - Set sound_timer = Vx.
    fn ld_st_vx(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;

        self.sound_timer = self.gp_registers[x];
        self.pc += 2;
    }
    // Fx1E - Set I = I + Vx.
    fn add_i_vx(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;

        self.i_register += self.gp_registers[x] as u16;
        self.pc += 2;
    }
    // Fx29 - Set I to the location of the hexadecimal sprite corresponding to the value of Vx.
    fn ld_f_vx(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        // The hexadecimal sprites are 8x5, so we multiply the value of Vx by 5 to get the index of the sprite
        let i: u16 = (self.gp_registers[x] * 5) as u16;

        self.i_register = i;
        self.pc += 2;
    }
    // Fx33 - Store the BCD representation of Vx in I, I+1, and I+2. The hundreds place is stored in I, tens in I+1, and ones in I+2.
    fn ld_b_vx(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let hundreds: u8 = self.gp_registers[x] / 100;
        let tens: u8 = (self.gp_registers[x] / 10) % 10;
        let ones: u8 = self.gp_registers[x] % 10;
        let idx: usize = self.i_register as usize;

        self.memory[idx] = hundreds;
        self.memory[idx + 1] = tens;
        self.memory[idx + 2] = ones;
        self.pc += 2;
    }
    // Fx55 - Store the values in registers V0 - Vx in memory starting at location I.
    fn ld_i_vx(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let idx: usize = self.i_register as usize;

        for i in 0..=x {
            self.memory[idx + i] = self.gp_registers[i];
        }
        self.pc += 2;
    }
    // Fx65 - Read values from memory starting at location I and store them in registers V0 - Vx.
    fn ld_vx_i(&mut self, instruction: u16) {
        let x: usize = ((instruction & 0x0F00) >> 8) as usize;
        let idx: usize = self.i_register as usize;

        for i in 0..=x {
            self.gp_registers[i] = self.memory[idx + i];
        }
        self.pc += 2;
    }
}

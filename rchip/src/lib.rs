mod display;
mod keypad;
use display::Display;
use keypad::Keypad;

pub struct Cpu {
    memory: [u8; 4096],
    registers: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    display: Display,
    keypad: Keypad,
}

impl Cpu {
    pub fn new() -> Cpu {
        Self {
            memory: [0u8; 4096],
            registers: [0u8; 16],
            i: 0,
            pc: 0x200,
            stack: [0u16; 16],
            sp: 0u8,
            delay_timer: 0u8,
            sound_timer: 0u8,
            display: Display::new(),
            keypad: Keypad::new(),
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        //Variable that defines the first free starting memory address: 0x200
        let program_start = 0x200;

        for byte_index in 0..program.len() {
            self.memory[program_start + byte_index] = program[byte_index];
        }
    }

    fn fetch(&mut self) -> u16 {
        let high_byte: u8 = self.memory[self.pc as usize];
        let low_byte: u8 = self.memory[(self.pc + 1) as usize];
        self.pc += 2;

        ((high_byte as u16) << 8) | (low_byte as u16)
    }

    /// Returns register index from opcode
    fn register_index(opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
    }

    fn second_register_index(opcode: u16) -> usize {
        ((opcode & 0x00F0) >> 4) as usize
    }

    /// Returns immediate byte from opcode
    fn immediate_byte(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    /// Returns instruction type in hexadecimal from opcode
    fn instruction_type(opcode: u16) -> u8 {
        ((opcode & 0xF000) >> 12) as u8
    }

    /// Returns immediate 12_bit data from opcode, can be used for address values and data
    fn immediate_12bit_data(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    fn execute(&mut self, opcode: u16) {
        let instruction_type = Self::instruction_type(opcode);

        match opcode {
            // Clear Display Instruction
            0x00E0 => self.display.clear(),
            // Return from Call (branch) instruction)
            0x00EE => {
                // Decrement stack pointer
                self.sp -= 1;
                // Pop address from stack into program counter
                self.pc = self.stack[self.sp as usize];
                return;
            }
            _ => (),
        }

        match instruction_type {
            // Jump Instruction
            0x1 => {
                let data = Self::immediate_12bit_data(opcode);

                self.pc = data;
            }
            // Call (branch) Instruction
            0x2 => {
                let address = Self::immediate_12bit_data(opcode);
                // First store current program counter address into stack
                self.stack[self.sp as usize] = self.pc;
                // Increment stack pointer
                self.sp += 1;
                // Change program counter to new call (branch) address
                self.pc = address;
            }
            // Conditional if register == byte then skip instruction
            0x3 => {
                let register = Self::register_index(opcode);
                let byte = Self::immediate_byte(opcode);

                if self.registers[register] == byte {
                    self.pc += 2;
                }
            }
            // Conditional if register != byte then skip instruction
            0x4 => {
                let register = Self::register_index(opcode);
                let byte = Self::immediate_byte(opcode);

                if self.registers[register] != byte {
                    self.pc += 2;
                }
            }
            // Conditional if register_x == register_y then skip instruction
            0x5 => {
                if (opcode & 0x000F) != 0 {
                    return;
                }
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                if self.registers[register_x] == self.registers[register_y] {
                    self.pc += 2;
                }
            }
            // Load Immediate Instruction
            0x6 => {
                let register = Self::register_index(opcode);
                let byte = Self::immediate_byte(opcode);
                self.registers[register] = byte
            }
            // Add Immediate Instruction
            0x7 => {
                let register = Self::register_index(opcode);
                let byte = Self::immediate_byte(opcode);
                self.registers[register] = self.registers[register].wrapping_add(byte)
            }
            // Logical Operations and Arithmetic Instructions
            0x8 => Self::logic_and_arithmetic_functions(self, opcode),
            // Conditional if register_x != register_y then skip instruction
            0x9 => {
                if (opcode & 0x000F) != 0 {
                    return;
                }
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                if self.registers[register_x] != self.registers[register_y] {
                    self.pc += 2;
                }
            }
            // Load value into I register
            0xA => {
                let data = Self::immediate_12bit_data(opcode);

                self.i = data;
            }
            // Jump instruction (NNN + V0)
            0xB => {
                let data = Self::immediate_12bit_data(opcode);
                let final_address = data.wrapping_add(self.registers[0] as u16);

                self.pc = final_address;
            }
            // Random and with byte into register
            0xC => {
                let register = Self::register_index(opcode);
                let byte = Self::immediate_byte(opcode);

                let random_byte: u8 = rand::random();

                self.registers[register] = random_byte & byte;
            }
            // Draw Instruction
            0xD => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);
                let x_coord = self.registers[register_x] as usize;
                let y_coord = self.registers[register_y] as usize;
                let number_of_rows = opcode & 0x000F;

                let sprite_data: &[u8] =
                    &self.memory[(self.i as usize)..(self.i as usize + number_of_rows as usize)];

                self.registers[0xF] = if self.display.draw_sprite(x_coord, y_coord, sprite_data) {
                    1
                } else {
                    0
                };
            }
            // Input instructions
            0xE => self.keyboard_functions(opcode),
            // Timer/Utility related instructions
            0xF => self.timer_and_utility_functions(opcode),
            _ => (),
        }
    }

    fn timer_and_utility_functions(&mut self, opcode: u16) {
        let function = opcode & 0x00FF;

        match function {
            0x07 => {
                let register = Self::register_index(opcode);

                self.registers[register] = self.delay_timer;
            }
            0x15 => {
                let register = Self::register_index(opcode);

                self.delay_timer = self.registers[register];
            }
            0x18 => {
                let register = Self::register_index(opcode);

                self.sound_timer = self.registers[register];
            }
            0x1E => {
                let register = Self::register_index(opcode);
                let data = self.registers[register];

                self.i = self.i.wrapping_add(data as u16);
            }
            0x29 => {
                let register = Self::register_index(opcode);
                let data = self.registers[register];

                self.i = 0x200;
            }
            _ => (),
        }
    }

    fn keyboard_functions(&mut self, opcode: u16) {
        let function = opcode & 0x00FF;

        match function {
            0x9E => {
                let key = self.registers[Self::register_index(opcode)];
                if self.keypad.is_pressed(key as usize) {
                    self.pc += 2;
                }
            }
            0xA1 => {
                let key = self.registers[Self::register_index(opcode)];
                if !self.keypad.is_pressed(key as usize) {
                    self.pc += 2;
                }
            }
            _ => (),
        }
    }

    fn logic_and_arithmetic_functions(&mut self, opcode: u16) {
        let function = opcode & 0x000F;

        match function {
            // VX = VY
            0x0 => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                self.registers[register_x] = self.registers[register_y]
            }
            // VX = (VX OR VY)
            0x1 => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                self.registers[register_x] |= self.registers[register_y]
            }
            // VX = (VX AND VY)
            0x2 => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                self.registers[register_x] &= self.registers[register_y]
            }
            // VX = (VX XOR VY)
            0x3 => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                self.registers[register_x] ^= self.registers[register_y]
            }
            // VX = (VX + VY)
            0x4 => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                let (result, carry) =
                    self.registers[register_x].overflowing_add(self.registers[register_y]);

                self.registers[register_x] = result;
                self.registers[0xF] = if carry { 1 } else { 0 };
            }
            // VX = (VX - VY)
            0x5 => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                let (result, carry) =
                    self.registers[register_x].overflowing_sub(self.registers[register_y]);

                self.registers[register_x] = result;
                self.registers[0xF] = if carry { 0 } else { 1 };
            }
            // VX = (VX >> 1)
            0x6 => {
                let register_x = Self::register_index(opcode);

                self.registers[0xF] = self.registers[register_x] & 0x01;
                self.registers[register_x] >>= 1;
            }
            // VX = (VY - VX)
            0x7 => {
                let register_x = Self::register_index(opcode);
                let register_y = Self::second_register_index(opcode);

                let (result, carry) =
                    self.registers[register_y].overflowing_sub(self.registers[register_x]);

                self.registers[register_x] = result;
                self.registers[0xF] = if carry { 0 } else { 1 };
            }
            // VX = (VX << 1)
            0xE => {
                let register_x = Self::register_index(opcode);

                self.registers[0xF] = (self.registers[register_x] & 0x80) >> 7;
                self.registers[register_x] <<= 1;
            }
            _ => (),
        }
    }

    fn cycle(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_combines_two_bytes() {
        let mut cpu = Cpu::new();
        cpu.memory[0x200] = 0x60;
        cpu.memory[0x201] = 0x0A;

        let opcode = cpu.fetch();

        assert_eq!(opcode, 0x600A);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn correct_register_indexing_and_immediate_byte() {
        assert_eq!(Cpu::register_index(0x6A42), 0xA);
        assert_eq!(Cpu::immediate_byte(0x6A42), 0x42);
    }

    #[test]
    fn correct_instruction_type() {
        assert_eq!(Cpu::instruction_type(0x6A42), 0x6);
    }

    #[test]
    fn load_immediate_into_register() {
        let mut cpu = Cpu::new();

        cpu.execute(0x6A42);

        assert_eq!(cpu.registers[10], 0x42);
    }

    #[test]
    fn add_immediate_into_register() {
        let mut cpu = Cpu::new();

        cpu.execute(0x7A03);
        cpu.execute(0x7A07);

        assert_eq!(cpu.registers[10], 0xA);
    }

    #[test]
    fn add_immediate_into_register_overflow() {
        let mut cpu = Cpu::new();

        cpu.execute(0x7AFF);
        cpu.execute(0x7A07);

        assert_eq!(cpu.registers[10], 0x6);
    }

    #[test]
    fn cycle_fetches_and_executes() {
        let mut cpu = Cpu::new();

        cpu.memory[0x200] = 0x6A;
        cpu.memory[0x201] = 0x42;

        cpu.cycle();

        assert_eq!(cpu.registers[10], 0x42);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn load_small_program() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[0x6A, 0x42, 0x7A, 0x05]);

        assert_eq!(cpu.memory[0x200], 0x6A);
        assert_eq!(cpu.memory[0x201], 0x42);
        assert_eq!(cpu.memory[0x202], 0x7A);
        assert_eq!(cpu.memory[0x203], 0x05);
    }

    #[test]
    fn execute_small_program() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[0x6A, 0x42, 0x7A, 0x05]);

        cpu.cycle();
        cpu.cycle();

        assert_eq!(cpu.registers[10], 0x47);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn call_address() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[0x22, 0x04, 0x7A, 0x05, 0x6A, 0x42]);

        cpu.cycle();

        assert_eq!(cpu.stack[0], 0x202);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn call_address_and_return() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[0x22, 0x04, 0x6A, 0x42, 0x7A, 0x05, 0x00, 0xEE]);

        // The following program should initially add 0x5 into Register 10
        // But since the return brings it back to an immediate load, the final
        // result should be a 0x42 not a 0x47.
        // 0x204
        cpu.cycle();
        // 0x206
        cpu.cycle();
        // 0x202
        cpu.cycle();
        //0x204
        cpu.cycle();

        assert_eq!(cpu.registers[10], 0x42);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn conditional_execution_if_equal_skip() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[0x6A, 0x42, 0x3A, 0x42, 0x6A, 0x45, 0x3A, 0x45]);

        cpu.cycle();
        cpu.cycle();

        assert_eq!(cpu.pc, 0x206);

        // If not equal does not skip

        cpu.cycle();

        assert_eq!(cpu.registers[10], 0x42);
        assert_eq!(cpu.pc, 0x208);
    }

    #[test]
    fn conditional_execution_if_not_equal_skip() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[0x6A, 0x42, 0x4A, 0x42, 0x6A, 0x45, 0x4A, 0x47]);

        cpu.cycle();
        cpu.cycle();

        assert_eq!(cpu.pc, 0x204);

        // If not equal does skip

        cpu.cycle();
        cpu.cycle();

        assert_eq!(cpu.registers[10], 0x45);
        assert_eq!(cpu.pc, 0x20A);
    }

    #[test]
    fn conditional_execution_if_registers_equal_skip() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[
            0x6A, 0x42, 0x6B, 0x42, 0x5A, 0xB0, 0x69, 0x40, 0x6A, 0x45, 0x5A, 0xB0, 0x6B, 0x32,
        ]);

        // Load 42 in registers A and B
        cpu.cycle();
        cpu.cycle();

        // Execute conditional register execution should skip to 0x6A, 0x45
        cpu.cycle();

        assert_eq!(cpu.pc, 0x208);

        // Move 45 into register A then compare register A (45) and B (42)
        cpu.cycle();
        cpu.cycle();

        // Shouldn't skip so register B should have 32 loaded into it
        cpu.cycle();

        assert_eq!(cpu.registers[11], 0x32);
        assert_eq!(cpu.pc, 0x20E);
    }

    #[test]
    fn conditional_execution_if_registers_not_equal_skip() {
        let mut cpu = Cpu::new();

        cpu.load_program(&[
            0x6A, 0x42, 0x6B, 0x42, 0x9A, 0xB0, 0x69, 0x40, 0x6A, 0x45, 0x9A, 0xB0, 0x6B, 0x32,
        ]);

        // Load 42 in registers A and B
        cpu.cycle();
        cpu.cycle();

        // Execute conditional register execution should not  skip to 0x6A, 0x45
        cpu.cycle();

        assert_eq!(cpu.pc, 0x206);

        // Then Register 9 holds the value of 40
        // and Register A holds 45 now
        cpu.cycle();
        cpu.cycle();

        // Execute conditional register execution, should skip so register B still has the value of 42.
        cpu.cycle();

        assert_eq!(cpu.registers[9], 0x40);
        assert_eq!(cpu.registers[11], 0x42);
        assert_eq!(cpu.pc, 0x20E);
    }

    #[test]
    fn arithmetic_add() {
        let mut cpu = Cpu::new();

        // 200 into register A
        cpu.execute(0x6AC8);
        // 100 into register B
        cpu.execute(0x6B64);
        // Add reg A and B
        cpu.execute(0x8AB4);

        // Result should be 44 and carry
        assert_eq!(cpu.registers[0xF], 1);
        assert_eq!(cpu.registers[0xA], 44);

        // 100 into register A
        cpu.execute(0x6A64);
        // 50 into register B
        cpu.execute(0x6B32);
        // Add reg A and B
        cpu.execute(0x8AB4);

        // Result should be 150 without carry
        assert_eq!(cpu.registers[0xF], 0);
        assert_eq!(cpu.registers[0xA], 150);
    }

    #[test]
    fn arithmetic_sub() {
        let mut cpu = Cpu::new();

        // 10 into register A
        cpu.execute(0x6A0A);
        // 3 into register B
        cpu.execute(0x6B03);
        // Subtract reg a by reg b
        cpu.execute(0x8AB5);

        // Result should be 7 and flag is set
        assert_eq!(cpu.registers[0xF], 1);
        assert_eq!(cpu.registers[0xA], 7);

        // 3 into register A
        cpu.execute(0x6A03);
        // 10 into register B
        cpu.execute(0x6B0A);
        // Subtract reg a by reg b
        cpu.execute(0x8AB5);

        // Result should be 249 and flag is cleared
        assert_eq!(cpu.registers[0xF], 0);
        assert_eq!(cpu.registers[0xA], 249);
    }

    #[test]
    fn arithmetic_reverse_sub() {
        let mut cpu = Cpu::new();

        // 10 into register A
        cpu.execute(0x6A0A);
        // 3 into register B
        cpu.execute(0x6B03);
        // Subtract reg B by A
        cpu.execute(0x8AB7);

        // Result should be 249 and flag is cleared
        assert_eq!(cpu.registers[0xF], 0);
        assert_eq!(cpu.registers[0xA], 249);

        // 3 into register A
        cpu.execute(0x6A03);
        // 10 into register B
        cpu.execute(0x6B0A);
        // Subtract reg B by A
        cpu.execute(0x8AB7);

        // Result should be 7 and flag is set
        assert_eq!(cpu.registers[0xF], 1);
        assert_eq!(cpu.registers[0xA], 7);
    }

    #[test]
    fn logical_shift_right() {
        let mut cpu = Cpu::new();

        // Load 1011_0111 into reg A
        cpu.execute(0x6AB7);
        // Load 1011_0110 into reg B
        cpu.execute(0x6BB6);

        // Shift A right by 1
        cpu.execute(0x8AB6);
        assert_eq!(cpu.registers[0xA], 0x5B);
        assert_eq!(cpu.registers[0xF], 1);

        // Shift B right by 1
        cpu.execute(0x8BA6);
        assert_eq!(cpu.registers[0xB], 0x5B);
        assert_eq!(cpu.registers[0xF], 0);
    }

    #[test]
    fn logical_shift_left() {
        let mut cpu = Cpu::new();

        // Load 1011_0111 into reg A
        cpu.execute(0x6AB7);
        // Load 0011_0110 into reg B
        cpu.execute(0x6B37);

        // Shift A left by 1
        cpu.execute(0x8ABE);
        assert_eq!(cpu.registers[0xA], 0x6E);
        assert_eq!(cpu.registers[0xF], 1);

        // Shift B left by 1
        cpu.execute(0x8BAE);
        assert_eq!(cpu.registers[0xB], 0x6E);
        assert_eq!(cpu.registers[0xF], 0);
    }

    #[test]
    fn load_immediate_into_i() {
        let mut cpu = Cpu::new();

        cpu.execute(0xA300);

        assert_eq!(cpu.i, 0x300);
    }

    #[test]
    fn jump_to_address() {
        let mut cpu = Cpu::new();

        cpu.memory[0x200] = 0x13;
        cpu.memory[0x201] = 0x00;
        cpu.memory[0x300] = 0x6A;
        cpu.memory[0x301] = 0x42;

        cpu.cycle();
        cpu.cycle();

        assert_eq!(cpu.registers[0xA], 0x42);
        assert_eq!(cpu.pc, 0x302);
    }

    #[test]
    fn jump_to_address_plus_v0() {
        let mut cpu = Cpu::new();

        cpu.registers[0] = 0x10;

        cpu.memory[0x200] = 0xB3;
        cpu.memory[0x201] = 0x00;
        cpu.memory[0x300] = 0x6A;
        cpu.memory[0x301] = 0x42;
        cpu.memory[0x310] = 0x6A;
        cpu.memory[0x311] = 0x40;

        cpu.cycle();
        cpu.cycle();

        assert_eq!(cpu.registers[0xA], 0x40);
        assert_eq!(cpu.pc, 0x312);
    }

    #[test]
    // This is pointless lol
    fn random_and_with_immediate() {
        let mut cpu = Cpu::new();

        cpu.execute(0xCAF0);

        assert_eq!(cpu.registers[0xA] & 0x0F, 0);
    }

    #[test]
    fn draw_sprite_instruction() {
        let mut cpu = Cpu::new();

        // Fake sprite data
        cpu.memory[0x50] = 0x50;
        cpu.memory[0x51] = 0x41;
        cpu.memory[0x52] = 0x30;

        // I register point at the sprite data start
        cpu.i = 0x50;

        // X and Y coordinates
        cpu.registers[0xA] = 0x8;
        cpu.registers[0xB] = 0x5;

        // Draw the sprite data
        cpu.execute(0xDAB3);

        assert!(cpu.display.get_pixel(9, 5));
        assert!(cpu.display.get_pixel(11, 5));
        assert!(cpu.display.get_pixel(9, 6));
        assert!(cpu.display.get_pixel(15, 6));
        assert!(cpu.display.get_pixel(10, 7));
        assert!(cpu.display.get_pixel(11, 7));

        assert_eq!(cpu.registers[0xF], 0);

        // Erase sprite data
        cpu.execute(0xDAB3);

        assert!(!cpu.display.get_pixel(9, 5));
        assert!(!cpu.display.get_pixel(11, 5));
        assert!(!cpu.display.get_pixel(9, 6));
        assert!(!cpu.display.get_pixel(15, 6));
        assert!(!cpu.display.get_pixel(10, 7));
        assert!(!cpu.display.get_pixel(11, 7));

        assert_eq!(cpu.registers[0xF], 1);
    }

    #[test]
    fn clear_display() {
        let mut cpu = Cpu::new();

        cpu.memory[0x50] = 0x50;

        cpu.i = 0x50;

        cpu.registers[0xA] = 0x8;
        cpu.registers[0xB] = 0x5;

        cpu.execute(0xDAB3);
        assert!(cpu.display.get_pixel(9, 5));
        assert!(cpu.display.get_pixel(11, 5));

        cpu.execute(0x00E0);
        assert!(!cpu.display.get_pixel(9, 5));
        assert!(!cpu.display.get_pixel(11, 5));
    }

    #[test]
    fn input_branch() {
        let mut cpu = Cpu::new();

        cpu.registers[0] = 0xA;
        cpu.keypad.set_key(0xA, true);

        assert!(cpu.keypad.is_pressed(0xA));

        cpu.load_program(&[
            0x6A, 0x42, 0x6B, 0x42, 0xE0, 0x9E, 0x69, 0x40, 0x6A, 0x45, 0xE0, 0xA1, 0x6B, 0x32,
        ]);

        // Load 42 in registers A and B
        cpu.cycle();
        cpu.cycle();

        // Execute conditional register execution should skip to 0x6A, 0x45
        cpu.cycle();

        assert_eq!(cpu.pc, 0x208);

        // Move 45 into register A then compare register A (45) and B (42)
        cpu.cycle();
        cpu.cycle();

        // Shouldn't skip so register B should have 32 loaded into it
        cpu.cycle();

        assert_eq!(cpu.registers[11], 0x32);
        assert_eq!(cpu.pc, 0x20E);
    }

    #[test]
    fn set_register_to_delay_timer() {
        let mut cpu = Cpu::new();

        cpu.delay_timer = 120;

        cpu.execute(0xF207);

        assert_eq!(cpu.registers[0x2], 120);
    }

    #[test]
    fn set_delay_timer_to_register_value() {
        let mut cpu = Cpu::new();

        cpu.registers[0x2] = 120;

        cpu.execute(0xF215);

        assert_eq!(cpu.delay_timer, 120);
    }

    #[test]
    fn set_sound_timer_to_register_value() {
        let mut cpu = Cpu::new();

        cpu.registers[0x2] = 120;

        cpu.execute(0xF218);

        assert_eq!(cpu.sound_timer, 120);
    }

    #[test]
    fn add_register_into_i() {
        let mut cpu = Cpu::new();

        cpu.registers[0x2] = 0x20;
        cpu.i = 0x300;

        cpu.execute(0xF21E);

        assert_eq!(cpu.i, 0x320);

        cpu.registers[0x2] = 0x1;
        cpu.i = 0xFFFF;

        cpu.execute(0xF21E);

        assert_eq!(cpu.i, 0);
    }
}

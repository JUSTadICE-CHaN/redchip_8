pub struct Cpu {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Self {
            memory: [0u8; 4096],
            registers: [0u8; 16],
            pc: 0x200,
            stack: [0u16; 16],
            sp: 0u8,
            delay_timer: 0u8,
            sound_timer: 0u8,
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

    /// Returns immediate byte from opcode
    fn immediate_byte(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    /// Returns instruction type in hexadecimal from opcode
    fn instruction_type(opcode: u16) -> u8 {
        ((opcode & 0xF000) >> 12) as u8
    }

    /// Returns immediate address from opcode
    fn immediate_address(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    fn execute(&mut self, opcode: u16) {
        let instruction_type = Self::instruction_type(opcode);

        match opcode {
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
            // Call (branch) Instruction
            0x2 => {
                let address = Self::immediate_address(opcode);
                // First store current program counter address into stack
                self.stack[self.sp as usize] = self.pc;
                // Increment stack pointer
                self.sp += 1;
                // Change program counter to new call (branch) address
                self.pc = address;
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
}

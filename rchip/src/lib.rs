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

    fn fetch(&mut self) -> u16 {
        let high_byte: u8 = self.memory[self.pc as usize];
        let low_byte: u8 = self.memory[(self.pc + 1) as usize];
        self.pc += 2;

        ((high_byte as u16) << 8) | (low_byte as u16)
    }

    fn register_index(opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
    }

    fn immediate_byte(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    fn execute_load(&mut self, opcode: u16) {
        let register = Self::register_index(opcode);
        let byte = Self::immediate_byte(opcode);

        self.registers[register] = byte;
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
    fn load_immediate_into_register() {
        let mut cpu = Cpu::new();

        cpu.execute_load(0x6A42);

        assert_eq!(cpu.registers[10], 0x42);
    }
}

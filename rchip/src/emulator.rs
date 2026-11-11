use std::time::Duration;

use crate::{
    Cpu,
    display::Display,
    scheduler::{ScheduledEvent, Scheduler},
};

pub struct Emulator {
    cpu: Cpu,
    scheduler: Scheduler,
    frequency: u16,
    program: Vec<u8>,
}

impl Emulator {
    pub fn new(freq: u16) -> Self {
        let cpu = Cpu::new();
        let scheduler = Scheduler::new(freq);
        let frequency = freq;

        Self {
            cpu,
            scheduler,
            frequency,
            program: Vec::new(),
        }
    }

    pub fn advance(&mut self, elapsed: Duration) -> Result<(), String> {
        let events = self.scheduler.advance(elapsed);

        for event in events {
            match event {
                ScheduledEvent::CpuInstruction => self.cpu.cycle()?,
                ScheduledEvent::TimerTick => self.cpu.timer_tick(),
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
        self.scheduler = Scheduler::new(self.frequency);

        if !self.program.is_empty() {
            self.cpu.load_program(&self.program).unwrap();
        }
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<(), String> {
        if program.is_empty() {
            return Err("ROM file is empty".to_string());
        }

        let mut new_cpu = Cpu::new();
        new_cpu.load_program(program)?;

        self.cpu = new_cpu;
        self.scheduler = Scheduler::new(self.frequency);
        self.program = program.to_vec();

        Ok(())
    }

    pub fn pixels(&self) -> &[bool; 2048] {
        self.cpu.display.pixels()
    }

    pub fn registers(&self) -> &[u8; 16] {
        self.cpu.registers()
    }

    pub fn pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn i_register(&self) -> u16 {
        self.cpu.i
    }

    pub fn delay_timer(&self) -> u8 {
        self.cpu.delay_timer
    }

    pub fn sound_timer(&self) -> u8 {
        self.cpu.sound_timer
    }

    pub fn set_key(&mut self, key: u8, pressed: bool) {
        if key > 0xF {
            return;
        }
        self.cpu.keypad.set_key(key as usize, pressed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proper_advance() {
        let mut emulator = Emulator::new(60);

        emulator
            .load_program(&[0x6A, 0x42, 0x7A, 0x05, 0xAA, 0x00])
            .unwrap();

        emulator.cpu.delay_timer = 5;
        emulator.cpu.sound_timer = 3;

        assert_eq!(emulator.cpu.memory[0x200], 0x6A);
        assert_eq!(emulator.cpu.memory[0x201], 0x42);
        assert_eq!(emulator.cpu.memory[0x202], 0x7A);
        assert_eq!(emulator.cpu.memory[0x203], 0x05);

        emulator
            .advance(Duration::from_secs_f64(1.0 / 60.0))
            .unwrap();

        assert_eq!(emulator.cpu.pc, 0x204);
        assert_eq!(emulator.cpu.delay_timer, 4);
        assert_eq!(emulator.cpu.sound_timer, 2);

        emulator
            .advance(Duration::from_secs_f64(1.0 / 60.0))
            .unwrap();

        assert_eq!(emulator.cpu.pc, 0x206);
        assert_eq!(emulator.cpu.delay_timer, 3);
        assert_eq!(emulator.cpu.sound_timer, 1);
    }
}

use crate::bus::Bus;
use crate::cartridge::RomError;
use crate::cpu::Cpu;

#[derive(Default, Debug)]
pub struct Emulator {
    bus: Bus,
    cpu: Cpu
}

impl Emulator {
    pub fn new() -> Self {
        Self { bus: Bus::new(), cpu: Cpu::new() }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) -> Result<(), RomError> {
        self.bus.load_cartridge(rom_data)
    }

    // Returns number of cycles taken by the instruction
    pub fn step(&self) -> u8 {
        // Get current instruction addr

        // Get opcode at that addr

        // execute the instruction

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_creation() {
        let _emulator = Emulator::new();
    }

    #[test]
    fn test_emulator_step() {
        let emulator = Emulator::new();
        emulator.step();
    }
}

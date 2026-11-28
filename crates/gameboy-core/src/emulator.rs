use crate::bus::Bus;
use crate::cartridge::RomError;
use crate::cpu::Cpu;

#[derive(Default, Debug)]
pub struct Emulator {
    bus: Bus,
    cpu: Cpu,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            bus: Bus::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) -> Result<(), RomError> {
        self.bus.load_cartridge(rom_data)
    }

    // Returns number of cycles taken by the instruction
    pub fn step(&mut self) -> u8 {
        self.cpu.step(&mut self.bus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom_with_program(program: &[u8]) -> Vec<u8> {
        let mut rom = vec![0; 0x8000];
        rom[0x0100..0x0100 + program.len()].copy_from_slice(program);
        rom[0x0134..=0x0143].copy_from_slice(b"TEST ROM\0\0\0\0\0\0\0\0");
        rom[0x014D] = rom[0x0134..=0x014C].iter().fold(0_u8, |checksum, &byte| {
            checksum.wrapping_sub(byte).wrapping_sub(1)
        });
        rom
    }

    #[test]
    fn test_emulator_creation() {
        let _emulator = Emulator::new();
    }

    #[test]
    fn test_emulator_step() {
        let mut emulator = Emulator::new();
        emulator.load_rom(rom_with_program(&[0x00])).unwrap();

        assert_eq!(emulator.step(), 4);
    }
}

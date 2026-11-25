
use crate::bus::Bus;
use crate::cartridge::RomError;

#[derive(Default, Debug)]
pub struct Emulator {
    bus: Bus,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            bus: Bus::new(),
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) -> Result<(), RomError> {
        self.bus.load_cartridge(rom_data)
    }

    pub fn step(&self) {
        println!("Stepping emulator");
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

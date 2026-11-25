use gameboy_core::Emulator;
use std::{env, fs};

fn main() {
    // read in test file .gb from test/fixtures/roms/dmg-acid2.gb
    let rom_path = env::args()
        .nth(1)
        .expect("Usage: gameboy-frontend <path-to-rom.gb>");
    let rom_data =
        fs::read(&rom_path).unwrap_or_else(|err| panic!("Failed to read ROM '{rom_path}': {err}"));

    let mut _emulator = Emulator::new();
    _emulator.load_rom(rom_data).unwrap();
}

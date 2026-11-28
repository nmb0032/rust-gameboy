use gameboy_core::Emulator;

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
fn emulator_can_be_created() {
    let mut emulator = Emulator::new();
    emulator.load_rom(rom_with_program(&[0x00])).unwrap();

    assert_eq!(emulator.step(), 4);
}

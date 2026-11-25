use gameboy_core::Emulator;

#[test]
fn emulator_can_be_created() {
    let emulator = Emulator::new();
    emulator.step();
}

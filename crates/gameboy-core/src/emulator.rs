#[derive(Default)]
pub struct Emulator;

impl Emulator {
    pub fn new() -> Self {
        Self
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

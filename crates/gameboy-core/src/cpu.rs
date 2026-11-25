use crate::registers::Registers;

#[derive(Default, Debug)]
struct Cpu {
    registers: Registers,
}

impl Cpu {
    fn new() -> Self {
        Self::default()
    }
}

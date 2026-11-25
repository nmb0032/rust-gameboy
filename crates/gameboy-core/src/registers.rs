use crate::flags::Flags;

#[derive(Debug)]
pub(crate) struct Registers {
    a: u8,
    f: Flags,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0x01,
            f: Flags::from_value(0xB0),
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }
}

impl Registers {
    pub(crate) fn a(&self) -> u8 {
        self.a
    }

    pub(crate) fn set_a(&mut self, value: u8) {
        self.a = value;
    }

    pub(crate) fn flags(&self) -> &Flags {
        &self.f
    }

    pub(crate) fn flags_mut(&mut self) -> &mut Flags {
        &mut self.f
    }

    pub(crate) fn b(&self) -> u8 {
        self.b
    }

    pub(crate) fn set_b(&mut self, value: u8) {
        self.b = value;
    }

    pub(crate) fn c(&self) -> u8 {
        self.c
    }

    pub(crate) fn set_c(&mut self, value: u8) {
        self.c = value;
    }

    pub(crate) fn d(&self) -> u8 {
        self.d
    }

    pub(crate) fn set_d(&mut self, value: u8) {
        self.d = value;
    }

    pub(crate) fn e(&self) -> u8 {
        self.e
    }

    pub(crate) fn set_e(&mut self, value: u8) {
        self.e = value;
    }

    pub(crate) fn h(&self) -> u8 {
        self.h
    }

    pub(crate) fn set_h(&mut self, value: u8) {
        self.h = value;
    }

    pub(crate) fn l(&self) -> u8 {
        self.l
    }

    pub(crate) fn set_l(&mut self, value: u8) {
        self.l = value;
    }

    pub(crate) fn sp(&self) -> u16 {
        self.sp
    }

    pub(crate) fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    pub(crate) fn pc(&self) -> u16 {
        self.pc
    }

    pub(crate) fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub(crate) fn af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.f.value()])
    }

    pub(crate) fn set_af(&mut self, value: u16) {
        let [a, f] = value.to_be_bytes();
        self.a = a;
        self.f.set_value(f);
    }

    pub(crate) fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    pub(crate) fn set_bc(&mut self, value: u16) {
        [self.b, self.c] = value.to_be_bytes();
    }

    pub(crate) fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    pub(crate) fn set_de(&mut self, value: u16) {
        [self.d, self.e] = value.to_be_bytes();
    }

    pub(crate) fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    pub(crate) fn set_hl(&mut self, value: u16) {
        [self.h, self.l] = value.to_be_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_to_dmg_post_boot_values() {
        let registers = Registers::default();

        assert_eq!(registers.af(), 0x01B0);
        assert_eq!(registers.bc(), 0x0013);
        assert_eq!(registers.de(), 0x00D8);
        assert_eq!(registers.hl(), 0x014D);
        assert_eq!(registers.sp(), 0xFFFE);
        assert_eq!(registers.pc(), 0x0100);
    }

    #[test]
    fn reads_and_writes_individual_registers() {
        let mut registers = Registers::default();

        registers.set_a(0x12);
        registers.set_b(0x34);
        registers.set_c(0x56);
        registers.set_d(0x78);
        registers.set_e(0x9A);
        registers.set_h(0xBC);
        registers.set_l(0xDE);
        registers.set_sp(0xFF80);
        registers.set_pc(0x0100);
        registers.flags_mut().set_zero(true);

        assert_eq!(registers.a(), 0x12);
        assert_eq!(registers.b(), 0x34);
        assert_eq!(registers.c(), 0x56);
        assert_eq!(registers.d(), 0x78);
        assert_eq!(registers.e(), 0x9A);
        assert_eq!(registers.h(), 0xBC);
        assert_eq!(registers.l(), 0xDE);
        assert_eq!(registers.sp(), 0xFF80);
        assert_eq!(registers.pc(), 0x0100);
        assert!(registers.flags().zero());
    }

    #[test]
    fn writes_and_reads_register_pairs() {
        let mut registers = Registers::default();

        registers.set_af(0x123F);
        registers.set_bc(0x4567);
        registers.set_de(0x89AB);
        registers.set_hl(0xCDEF);

        assert_eq!(registers.af(), 0x1230);
        assert_eq!(registers.bc(), 0x4567);
        assert_eq!(registers.de(), 0x89AB);
        assert_eq!(registers.hl(), 0xCDEF);
    }
}

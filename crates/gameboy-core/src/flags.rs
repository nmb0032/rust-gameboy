const ZERO: u8 = 0b1000_0000;
const SUBTRACTION: u8 = 0b0100_0000;
const HALF_CARRY: u8 = 0b0010_0000;
const CARRY: u8 = 0b0001_0000;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Flags {
    value: u8,
}

impl Flags {
    pub(crate) fn from_value(value: u8) -> Self {
        let mut flags = Self::default();
        flags.set_value(value);
        flags
    }

    pub(crate) fn value(&self) -> u8 {
        self.value
    }

    pub(crate) fn set_value(&mut self, value: u8) {
        self.value = value & 0xF0;
    }

    pub(crate) fn set_zero(&mut self, value: bool) {
        self.set(ZERO, value);
    }

    pub(crate) fn zero(&self) -> bool {
        self.is_set(ZERO)
    }

    pub(crate) fn set_subtraction(&mut self, value: bool) {
        self.set(SUBTRACTION, value);
    }

    pub(crate) fn subtraction(&self) -> bool {
        self.is_set(SUBTRACTION)
    }

    pub(crate) fn set_half_carry(&mut self, value: bool) {
        self.set(HALF_CARRY, value);
    }

    pub(crate) fn half_carry(&self) -> bool {
        self.is_set(HALF_CARRY)
    }

    pub(crate) fn set_carry(&mut self, value: bool) {
        self.set(CARRY, value);
    }

    pub(crate) fn carry(&self) -> bool {
        self.is_set(CARRY)
    }

    fn set(&mut self, mask: u8, value: bool) {
        if value {
            self.value |= mask;
        } else {
            self.value &= !mask;
        }
    }

    fn is_set(&self, mask: u8) -> bool {
        self.value & mask != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_nibble_is_always_discarded() {
        assert_eq!(Flags::from_value(0xBF).value(), 0xB0);
    }

    #[test]
    fn flags_are_set_and_cleared_independently() {
        let mut flags = Flags::default();

        flags.set_zero(true);
        flags.set_subtraction(true);
        flags.set_half_carry(true);
        flags.set_carry(true);
        flags.set_half_carry(false);

        assert!(flags.zero());
        assert!(flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
        assert_eq!(flags.value(), 0xD0);
    }
}

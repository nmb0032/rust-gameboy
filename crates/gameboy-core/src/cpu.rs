use crate::bus::Bus;
use crate::registers::Registers;

#[derive(Debug)]
enum AluOp {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
}

#[derive(Debug)]
enum JumpOp {
    Jp,
    Jr,
    Call,
    Ret,
    Rst,
}

#[derive(Debug)]
enum Condition {
    Always,
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

#[derive(Debug)]
enum Operand8 {
    B,
    C,
    D,
    E,
    H,
    L,
    MemoryHl,
    A,
    Immediate(u8),
}

#[derive(Debug)]
enum Register16 {
    Bc,
    De,
    Hl,
    Sp,
}

#[derive(Debug)]
enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug)]
enum Target8 {
    Register(Register8),
    MemoryHl,
}

#[derive(Debug)]
enum Instruction {
    Nop,

    Halt,

    Alu { operation: AluOp, operand: Operand8 },

    Jp { condition: Condition, address: u16 },

    Jr { condition: Condition, offset: i8 },

    Ld16 { register: Register16, value: u16 },

    Ld8 { register: Target8, value: u8 },

    Call { condition: Condition, address: u16 },

    Ret { condition: Condition },

    Rst(u16),
}

#[derive(Default, Debug)]
pub(crate) struct Cpu {
    is_halted: bool,
    registers: Registers,
}

impl Cpu {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn step(&mut self, bus: &mut Bus) -> u8 {
        // Get current instruction addr
        let instruction_address = self.registers.pc();
        let opcode = self.fetch_byte(bus);
        let instruction = self.decode(opcode, instruction_address, bus);
        // decode and execute the instruction
        self.execute(instruction, bus)
    }

    fn execute(&mut self, instruction: Instruction, bus: &mut Bus) -> u8 {
        match instruction {
            Instruction::Nop => 4,
            Instruction::Halt => {
                self.is_halted = true;
                4
            }
            Instruction::Ld16 { register, value } => {
                match register {
                    Register16::Bc => self.registers.set_bc(value),
                    Register16::De => self.registers.set_de(value),
                    Register16::Hl => self.registers.set_hl(value),
                    Register16::Sp => self.registers.set_sp(value),
                }
                12 // 12 cycles for all 16 bit LD
            }
            Instruction::Ld8 { register, value } => {
                match register {
                    Target8::Register(reg) => match reg {
                        Register8::A => self.registers.set_a(value),
                        Register8::B => self.registers.set_b(value),
                        Register8::C => self.registers.set_c(value),
                        Register8::D => self.registers.set_d(value),
                        Register8::E => self.registers.set_e(value),
                        Register8::H => self.registers.set_h(value),
                        Register8::L => self.registers.set_l(value),
                    },
                    Target8::MemoryHl => {
                        let hl = self.registers.hl();
                        bus.write(hl, value);
                        return 12; // 12 cycle MHL
                    }
                }
                8 //8 cycles for the rest
            }
            Instruction::Jp { condition, address } => {
                match condition {
                    Condition::Always => {
                        self.registers.set_pc(address);
                        return 16; // 16 cycles for JP always
                    }
                    Condition::NotZero => {
                        if !self.registers.flags().zero() {
                            self.registers.set_pc(address);
                            return 16; // 16 cycles for JP NZ taken
                        }
                    }
                    Condition::Zero => {
                        if self.registers.flags().zero() {
                            self.registers.set_pc(address);
                            return 16; // 16 cycles for JP Z taken
                        }
                    }
                    Condition::Carry => {
                        if self.registers.flags().carry() {
                            self.registers.set_pc(address);
                            return 16; // 16 cycles for JP C taken
                        }
                    }
                    Condition::NotCarry => {
                        if !self.registers.flags().carry() {
                            self.registers.set_pc(address);
                            return 16; // 16 cycles for JP NC taken
                        }
                    }
                }
                12 // 12 cycles for JP not taken
            }
            _ => unimplemented!("Instruction {:?} not implemented", instruction),
        }
    }

    fn decode(&mut self, opcode: u8, instruction_address: u16, bus: &mut Bus) -> Instruction {
        // Match opcode to instruction
        match opcode {
            0b0000_0000 => Instruction::Nop,
            0b0111_0110 => Instruction::Halt,
            // ld r16, imm16
            opcode if opcode & 0b1100_1111 == 0b0000_0001 => Instruction::Ld16 {
                register: match opcode & 0b0011_0000 {
                    0b0000_0000 => Register16::Bc,
                    0b0001_0000 => Register16::De,
                    0b0010_0000 => Register16::Hl,
                    0b0011_0000 => Register16::Sp,
                    _ => unreachable!(),
                },
                value: self.fetch_word(bus),
            },
            // ld r, n
            opcode if opcode & 0b1100_0111 == 0b0000_0110 => Instruction::Ld8 {
                register: match opcode & 0b0011_1000 {
                    0b0000_0000 => Target8::Register(Register8::B),
                    0b0000_1000 => Target8::Register(Register8::C),
                    0b0001_0000 => Target8::Register(Register8::D),
                    0b0001_1000 => Target8::Register(Register8::E),
                    0b0010_0000 => Target8::Register(Register8::H),
                    0b0010_1000 => Target8::Register(Register8::L),
                    0b0011_0000 => Target8::MemoryHl,
                    0b0011_1000 => Target8::Register(Register8::A),
                    _ => unreachable!(),
                },
                value: self.fetch_byte(bus),
            },
            // jp cond, imm16
            opcode if opcode & 0b1110_0111 == 0b1100_0010 => Instruction::Jp {
                condition: match opcode & 0b0001_1000 {
                    0b0000_0000 => Condition::NotZero,
                    0b0000_1000 => Condition::Zero,
                    0b0001_0000 => Condition::NotCarry,
                    0b0001_1000 => Condition::Carry,
                    _ => unreachable!(),
                },
                address: self.fetch_word(bus),
            },
            // jp imm16
            opcode if opcode == 0b1100_0011 => Instruction::Jp {
                condition: Condition::Always,
                address: self.fetch_word(bus),
            },
            _ => unimplemented!(
                "Opcode {:08b} at address {:#06X} not implemented",
                opcode,
                instruction_address
            ),
        }
    }

    // Fetches the next byte from memory and increments the program counter
    fn fetch_byte(&mut self, bus: &mut Bus) -> u8 {
        let pc = self.registers.pc();
        let byte = bus.read(pc);
        self.registers.set_pc(pc.wrapping_add(1));
        byte
    }

    // The next two operands are fetched from the pc little endian notation low, high
    fn fetch_word(&mut self, bus: &mut Bus) -> u16 {
        let low = self.fetch_byte(bus);
        let high = self.fetch_byte(bus);
        u16::from_le_bytes([low, high])
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

    fn cpu_with_program(program: &[u8]) -> (Cpu, Bus) {
        let mut bus = Bus::new();
        bus.load_cartridge(rom_with_program(program)).unwrap();
        (Cpu::new(), bus)
    }

    #[test]
    fn executes_nop() {
        let (mut cpu, mut bus) = cpu_with_program(&[0x00]);

        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.pc(), 0x0101);
    }

    #[test]
    fn executes_halt() {
        let (mut cpu, mut bus) = cpu_with_program(&[0x76]);

        assert_eq!(cpu.step(&mut bus), 4);
        assert!(cpu.is_halted);
        assert_eq!(cpu.registers.pc(), 0x0101);
    }

    #[test]
    fn loads_each_8_bit_register_from_immediate() {
        let cases = [
            (0x06, 0xB0_u8),
            (0x0E, 0xC0_u8),
            (0x16, 0xD0_u8),
            (0x1E, 0xE0_u8),
            (0x26, 0x50_u8),
            (0x2E, 0x60_u8),
            (0x3E, 0xA0_u8),
        ];

        for (opcode, value) in cases {
            let (mut cpu, mut bus) = cpu_with_program(&[opcode, value]);

            assert_eq!(cpu.step(&mut bus), 8);
            let actual = match opcode {
                0x06 => cpu.registers.b(),
                0x0E => cpu.registers.c(),
                0x16 => cpu.registers.d(),
                0x1E => cpu.registers.e(),
                0x26 => cpu.registers.h(),
                0x2E => cpu.registers.l(),
                0x3E => cpu.registers.a(),
                _ => unreachable!(),
            };
            assert_eq!(actual, value);
            assert_eq!(cpu.registers.pc(), 0x0102);
        }
    }

    #[test]
    fn loads_memory_at_hl_from_immediate() {
        let (mut cpu, mut bus) = cpu_with_program(&[0x36, 0xAB]);
        cpu.registers.set_hl(0xC123);

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(bus.read(0xC123), 0xAB);
        assert_eq!(cpu.registers.hl(), 0xC123);
    }

    #[test]
    fn loads_each_16_bit_register_from_immediate() {
        let cases = [
            (0x01_u8, 0x1234_u16),
            (0x11_u8, 0x5678_u16),
            (0x21_u8, 0x9ABC_u16),
            (0x31_u8, 0xDEF0_u16),
        ];

        for (opcode, value) in cases {
            let value_bytes = value.to_le_bytes();
            let (mut cpu, mut bus) = cpu_with_program(&[opcode, value_bytes[0], value_bytes[1]]);

            assert_eq!(cpu.step(&mut bus), 12);
            let actual = match opcode {
                0x01 => cpu.registers.bc(),
                0x11 => cpu.registers.de(),
                0x21 => cpu.registers.hl(),
                0x31 => cpu.registers.sp(),
                _ => unreachable!(),
            };
            assert_eq!(actual, value);
            assert_eq!(cpu.registers.pc(), 0x0103);
        }
    }

    #[test]
    fn jumps_to_immediate_address() {
        let (mut cpu, mut bus) = cpu_with_program(&[0xC3, 0x34, 0x12]);

        assert_eq!(cpu.step(&mut bus), 16);
        assert_eq!(cpu.registers.pc(), 0x1234);
    }

    #[test]
    fn conditionally_jumps_when_condition_matches() {
        let cases = [
            (0xC2, Condition::NotZero),
            (0xCA, Condition::Zero),
            (0xD2, Condition::NotCarry),
            (0xDA, Condition::Carry),
        ];

        for (opcode, condition) in cases {
            let (mut cpu, mut bus) = cpu_with_program(&[opcode, 0x34, 0x12]);
            match condition {
                Condition::NotZero => cpu.registers.flags_mut().set_zero(false),
                Condition::Zero => cpu.registers.flags_mut().set_zero(true),
                Condition::NotCarry => cpu.registers.flags_mut().set_carry(false),
                Condition::Carry => cpu.registers.flags_mut().set_carry(true),
                Condition::Always => unreachable!(),
            }

            assert_eq!(cpu.step(&mut bus), 16);
            assert_eq!(cpu.registers.pc(), 0x1234);
        }
    }

    #[test]
    fn conditionally_does_not_jump_when_condition_does_not_match() {
        let cases = [
            (0xC2, Condition::NotZero),
            (0xCA, Condition::Zero),
            (0xD2, Condition::NotCarry),
            (0xDA, Condition::Carry),
        ];

        for (opcode, condition) in cases {
            let (mut cpu, mut bus) = cpu_with_program(&[opcode, 0x34, 0x12]);
            match condition {
                Condition::NotZero => cpu.registers.flags_mut().set_zero(true),
                Condition::Zero => cpu.registers.flags_mut().set_zero(false),
                Condition::NotCarry => cpu.registers.flags_mut().set_carry(true),
                Condition::Carry => cpu.registers.flags_mut().set_carry(false),
                Condition::Always => unreachable!(),
            }

            assert_eq!(cpu.step(&mut bus), 12);
            assert_eq!(cpu.registers.pc(), 0x0103);
        }
    }

    #[test]
    #[should_panic(expected = "Opcode 11010011 at address 0x0100 not implemented")]
    fn unknown_opcode_panic_includes_instruction_address() {
        let (mut cpu, mut bus) = cpu_with_program(&[0xD3]);

        cpu.step(&mut bus);
    }
}

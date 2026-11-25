use crate::registers::Registers;
use crate::bus::Bus;

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
    B, C, D, E, H, L,
    MemoryHl,
    A,
    Immediate(u8),
}

#[derive(Debug)]
enum Register16 {
    Bc,
    De,
    Hl,
    Sp
}

#[derive(Debug)]
enum Instruction {
    Nop,

    Halt,

    Alu {
        operation: AluOp,
        operand: Operand8,
    },

    Jp {
        condition: Condition,
        address: u16,
    },

    Jr {
        condition: Condition,
        offset: i8,
    },

    Ld {
        register: Register16,
        value: u16
    },

    Call {
        condition: Condition,
        address: u16,
    },

    Ret {
        condition: Condition,
    },

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
        let opcode = self.fetch_byte(bus);
        let instruction = self.decode(opcode);
        // decode and execute the instruction
        self.execute(instruction, bus)
    }

    fn execute(&mut self, instruction: Instruction, bus: &mut Bus) -> u8 {
        match instruction {
            Instruction::Nop => {
                4
            },
            Instruction::Halt => {
                self.is_halted = true;
                4
            },
            Instruction::Ld { register, value } => {},
            Instruction::Jp { condition, address } => {},
            _ => unimplemented!("Instruction {:?} not implemented", instruction),
        }
    }

    fn decode(&mut self, opcode: u8, bus: &mut Bus) -> Instruction {
        // Match opcode to instruction
        match opcode {
            0b0000_0000 => Instruction::Nop,
            0b0111_0110 => Instruction::Halt,
            // ld r16, imm16
            opcode if opcode & 0b1100_1111 == 0b0000_0001 => Instruction::Ld {
                register: match opcode & 0b0011_0000 {
                    0b0000_0000 => Register16::Bc,
                    0b0001_0000 => Register16::De,
                    0b0010_0000 => Register16::Hl,
                    0b0011_0000 => Register16::Sp,
                    _ => unreachable!(),
                },
                value: self.fetch_word(bus),
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
                address: self.fetch_word(bus)
            },
            _ => unimplemented!("Opcode {:08b} not implemented", opcode),
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

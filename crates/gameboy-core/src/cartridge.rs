#[derive(Debug)]
pub(crate) struct Cartridge {
    rom: Vec<u8>,
}

#[derive(Debug)]
pub enum RomError {
    TooShort,
    HeaderChecksumMismatch { calculated: u8, expected: u8 },
    UnsupportedCartridgeType(u8),
}

impl std::fmt::Display for RomError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(formatter, "ROM is too short to contain a cartridge header"),
            Self::HeaderChecksumMismatch {
                calculated,
                expected,
            } => write!(
                formatter,
                "header checksum mismatch: calculated {calculated:#04X}, expected {expected:#04X}"
            ),
            Self::UnsupportedCartridgeType(cartridge_type) => {
                write!(formatter, "unsupported cartridge type: {cartridge_type:#04X}")
            }
        }
    }
}

impl std::error::Error for RomError {}

impl Cartridge {
    pub(crate) fn new(rom: Vec<u8>) -> Result<Self, RomError> {
        if rom.len() < 0x0150 {
            return Err(RomError::TooShort);
        }

        let title = &rom[0x0134..=0x0143]
            .iter()
            .map(|&c| c as char)
            .collect::<String>();
        let cartridge_type = &rom[0x0147];
        let rom_size = &rom[0x0148];
        let ram_size = &rom[0x0149];
        let header_checksum = &rom[0x014D];

        let expected_checksum = rom[0x0134..=0x014C]
            .iter()
            .fold(0_u8, |checksum, &byte| {
                checksum.wrapping_sub(byte).wrapping_sub(1)
            });
        if expected_checksum != *header_checksum {
            return Err(RomError::HeaderChecksumMismatch {
                calculated: expected_checksum,
                expected: *header_checksum,
            });
        }

        if *cartridge_type != 0x00 {
            return Err(RomError::UnsupportedCartridgeType(*cartridge_type));
        }

        let _ = (title, rom_size, ram_size);
        Ok(Self { rom })
    }

    pub(crate) fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
}

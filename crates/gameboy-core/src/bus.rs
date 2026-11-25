//! Game Boy Memory map quick reference (16-bit address space):
//! 
//! 0000-3FFF: ROM Bank 0   (16kib) (Cartidge owned)
//! 4000-7FFF: ROM Bank 01-NN (16kib) (Cartidge owned)
//! 8000-9FFF: Video RAM (8kib)
//! A000-BFFF: External RAM (8kib)
//! C000-DFFF: Work RAM (8kib)
//! E000-FDFF: Echo RAM (7.5kib) (Mirrors Work Ram)
//! FE00-FE9F: OAM (Sprite Attribute Table) (160b)
//! FEA0-FEFF: Not Usable
//! FF00-FF7F: I/O Registers (128b)
//! FF80-FFFE: High RAM (127b)
//! FFFF: Interrupt Enable Register (1b)

#[derive(Debug)]
pub struct Bus {
    cartridge: Option<Cartridge>,
    vram : [u8; 0x2000],
    work_ram: [u8; 0x2000],
    oam: [u8; 160],
    io_registers: [u8; 128],
    high_ram: [u8; 127],
    interrupt_enable: u8,
}

#[derive(Debug)]
struct Cartridge {
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

impl Default for Bus {
    fn default() -> Self {
        Self {
            cartridge: None,
            vram: [0; 0x2000],
            work_ram: [0; 0x2000],
            oam: [0; 160],
            io_registers: [0; 128],
            high_ram: [0; 127],
            interrupt_enable: 0,
        }
    }
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_cartridge(&mut self, rom: Vec<u8>) -> Result<(), RomError> {
        self.cartridge = Some(Cartridge::new(rom)?);
        Ok(())
    }

    pub fn read(&self, addr: u16) -> u8 {
        // Match address to correct memory region
        match addr {
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xC000..=0xDFFF => self.work_ram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => self.work_ram[(addr - 0xE000) as usize], // Echo RAM
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF00..=0xFF7F => self.io_registers[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize],
            0xFFFF => self.interrupt_enable,
            
            // Normally mapped to cartridge ROM
            0x0000..=0x7FFF => self
                .cartridge
                .as_ref()
                .map_or(0xFF, |cartridge| cartridge.read(addr)),

            // No backing memory for this range (FEA0-FEFF)
            0xFEA0..=0xFEFF => 0xFF, // Not usable
            _ => 0, // Default for unmapped addresses
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,
            0xC000..=0xDFFF => self.work_ram[(addr - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.work_ram[(addr - 0xE000) as usize] = value, // Echo RAM
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,
            0xFF00..=0xFF7F => self.io_registers[(addr - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize] = value,
            0xFFFF => self.interrupt_enable = value,

            0x0000..=0x7FFF => {}, // Cartridge ROM (not implemented)
            0xA000..=0xBFFF => {}, // Cartridge RAM (not implemented)
            0xFEA0..=0xFEFF => {}, // Not usable
        }
    }
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Result<Self, RomError> {
        if rom.len() < 0x0150 {
            return Err(RomError::TooShort);
        }

        let title = &rom[0x0134..=0x0143].iter().map(|&c| c as char).collect::<String>();
        let cartridge_type = &rom[0x0147];
        let rom_size = &rom[0x0148];
        let ram_size = &rom[0x0149];
        let header_checksum = &rom[0x014D];

        // check ROM header checksum
        let expected_checksum = rom[0x0134..=0x014C].iter().fold(0_u8, |checksum, &byte| checksum.wrapping_sub(byte).wrapping_sub(1));
        if expected_checksum != *header_checksum {
            return Err(RomError::HeaderChecksumMismatch { calculated: expected_checksum, expected: *header_checksum });
        }

        // Support only ROM cartridges for now
        if *cartridge_type != 0x00 {
            return Err(RomError::UnsupportedCartridgeType(*cartridge_type));
        }

        Ok(Self { rom })
    }

    pub fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_rom() -> Vec<u8> {
        let mut rom = vec![0; 0x8000];
        rom[0x0134..=0x0143].copy_from_slice(b"TEST ROM\0\0\0\0\0\0\0\0");
        rom[0x0147] = 0x00;
        rom[0x0148] = 0x00;
        rom[0x0149] = 0x00;
        rom[0x014D] = rom[0x0134..=0x014C]
            .iter()
            .fold(0_u8, |checksum, &byte| {
                checksum.wrapping_sub(byte).wrapping_sub(1)
            });
        rom
    }

    #[test]
    fn test_load_valid_cartridge() {
        let mut bus = Bus::new();
        let mut rom = valid_rom();
        rom[0x0000] = 0x12;
        rom[0x7FFF] = 0x34;

        assert!(bus.load_cartridge(rom).is_ok());
        assert_eq!(bus.read(0x0000), 0x12);
        assert_eq!(bus.read(0x7FFF), 0x34);
    }

    #[test]
    fn test_rejects_short_cartridge() {
        let mut bus = Bus::new();

        assert!(matches!(
            bus.load_cartridge(vec![0; 0x014F]),
            Err(RomError::TooShort)
        ));
    }

    #[test]
    fn test_rejects_invalid_header_checksum() {
        let mut bus = Bus::new();
        let mut rom = valid_rom();
        rom[0x014D] ^= 0xFF;

        assert!(matches!(
            bus.load_cartridge(rom),
            Err(RomError::HeaderChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_rejects_unsupported_cartridge_type() {
        let mut bus = Bus::new();
        let mut rom = valid_rom();
        rom[0x0147] = 0x01;
        rom[0x014D] = rom[0x0134..=0x014C]
            .iter()
            .fold(0_u8, |checksum, &byte| {
                checksum.wrapping_sub(byte).wrapping_sub(1)
            });

        assert!(matches!(
            bus.load_cartridge(rom),
            Err(RomError::UnsupportedCartridgeType(0x01))
        ));
    }

    #[test]
    fn test_vram_read() {
        let mut bus = Bus::new();
        bus.vram[0] = 0x42;
        assert_eq!(bus.read(0x8000), 0x42);
    }

    #[test]
    fn test_vram_write() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0x42);
        assert_eq!(bus.vram[0], 0x42);
    }

    #[test]
    fn test_vram_read_write() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0x42);
        assert_eq!(bus.read(0x8000), 0x42);
    }

    #[test]
    fn test_echo_ram_mirror() {
        let mut bus = Bus::new();
        bus.write(0xE000, 0x42);
        assert_eq!(bus.read(0xE000), 0x42);
        assert_eq!(bus.read(0xC000), 0x42); // Echo RAM mirrors Work RAM
    }

    #[test]
    fn test_echo_ram_write() {
        let mut bus = Bus::new();
        bus.write(0xE000, 0x42);
        assert_eq!(bus.work_ram[0], 0x42); // Echo RAM write should affect Work RAM
        assert_eq!(bus.read(0xC000), 0x42); // Work RAM reflects the change
    }

    #[test]
    fn test_oam_read_write() {
        let mut bus = Bus::new();
        bus.write(0xFE00, 0x42);
        assert_eq!(bus.oam[0], 0x42);
        assert_eq!(bus.read(0xFE00), 0x42);
    }

    #[test]
    fn test_interrupt_enable_read_write() {
        let mut bus = Bus::new();
        bus.write(0xFFFF, 0x42);
        assert_eq!(bus.interrupt_enable, 0x42);
        assert_eq!(bus.read(0xFFFF), 0x42);
    }

    #[test]
    fn test_high_ram_read_write() {
        let mut bus = Bus::new();
        bus.write(0xFF80, 0x42);
        assert_eq!(bus.high_ram[0], 0x42);
        assert_eq!(bus.read(0xFF80), 0x42);
    }

    #[test]
    fn test_io_registers_read_write() {
        let mut bus = Bus::new();
        bus.write(0xFF00, 0x42);
        assert_eq!(bus.io_registers[0], 0x42);
        assert_eq!(bus.read(0xFF00), 0x42);
    }

    #[test]
    fn test_memory_region_boundaries() {
        let mut bus = Bus::new();
        let regions = [
            (0x8000, 0x9FFF, 0x11),
            (0xC000, 0xDFFF, 0x22),
            (0xFE00, 0xFE9F, 0x33),
            (0xFF00, 0xFF7F, 0x44),
            (0xFF80, 0xFFFE, 0x55),
        ];

        for (start, end, value) in regions {
            bus.write(start, value);
            bus.write(end, value.wrapping_add(1));

            assert_eq!(bus.read(start), value);
            assert_eq!(bus.read(end), value.wrapping_add(1));
        }
    }

    #[test]
    fn test_echo_ram_boundaries() {
        let mut bus = Bus::new();

        bus.write(0xE000, 0x11);
        assert_eq!(bus.read(0xC000), 0x11);

        bus.write(0xFDFF, 0x22);
        assert_eq!(bus.read(0xDDFF), 0x22);

        bus.write(0xC000, 0x33);
        assert_eq!(bus.read(0xE000), 0x33);

        bus.write(0xDDFF, 0x44);
        assert_eq!(bus.read(0xFDFF), 0x44);
    }
}

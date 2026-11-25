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

struct Bus {
    //TODO: Add cartridge
    vram : [u8; 0x2000],
    work_ram: [u8; 0x2000],
    oam: [u8; 160],
    io_registers: [u8; 128],
    high_ram: [u8; 127],
    interrupt_enable: u8,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
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
            
            0x0000..=0x7FFF => 0, // Cartridge ROM (not implemented)

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
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

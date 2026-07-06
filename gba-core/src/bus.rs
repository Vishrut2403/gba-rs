const BIOS_SIZE: usize = 16 * 1024;
const EWRAM_SIZE: usize = 256 * 1024;
const IWRAM_SIZE: usize = 32 * 1024;
const IO_SIZE: usize = 1024;
const PALETTE_SIZE: usize = 1024;
const VRAM_SIZE: usize = 96 * 1024;
const OAM_SIZE: usize = 1024;

pub struct Bus {
    bios: [u8; BIOS_SIZE],
    ewram: [u8; EWRAM_SIZE],
    iwram: [u8; IWRAM_SIZE],
    io: [u8; IO_SIZE],
    palette: [u8; PALETTE_SIZE],
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    rom: Vec<u8>,
    sram: Vec<u8>,
}

impl Bus {
    pub fn new(rom: Vec<u8>, sram: Vec<u8>) -> Self {
        Self {
            bios: [0; BIOS_SIZE],
            ewram: [0; EWRAM_SIZE],
            iwram: [0; IWRAM_SIZE],
            io: [0; IO_SIZE],
            palette: [0; PALETTE_SIZE],
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            rom,
            sram,
        }
    }

    /// Single source of truth for checking if an address is in the SRAM region
    fn is_sram(addr: u32) -> bool {
        let top = addr >> 24;
        top == 0x0E || top == 0x0F
    }

    fn get_vram_offset(addr: u32) -> usize {
        let offset = (addr as usize) & 0x1FFFF;
        if offset >= 0x18000 {
            offset - 0x8000
        } else {
            offset
        }
    }

    pub fn read8(&self, addr: u32) -> u8 {
        match addr >> 24 {
            0x00 => self.bios[(addr as usize) & (BIOS_SIZE - 1)],
            0x02 => self.ewram[(addr as usize) & (EWRAM_SIZE - 1)],
            0x03 => self.iwram[(addr as usize) & (IWRAM_SIZE - 1)],
            0x04 => self.io[(addr as usize) & (IO_SIZE - 1)],
            0x05 => self.palette[(addr as usize) & (PALETTE_SIZE - 1)],
            0x06 => self.vram[Self::get_vram_offset(addr)],
            0x07 => self.oam[(addr as usize) & (OAM_SIZE - 1)],
            0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D => {
                if self.rom.is_empty() {
                    return 0;
                }
                let offset = (addr as usize) & 0x01FFFFFF;
                if offset < self.rom.len() {
                    self.rom[offset]
                } else {
                    0
                }
            }
            0x0E | 0x0F => {
                if self.sram.is_empty() {
                    return 0;
                }
                self.sram[(addr as usize) & (self.sram.len() - 1)]
            }
            _ => 0,
        }
    }

    pub fn read16(&self, addr: u32) -> u16 {
        if Self::is_sram(addr) {
            return (self.read8(addr) as u16) * 0x0101;
        }

        let lo = self.read8(addr);
        let hi = self.read8(addr + 1);
        u16::from_le_bytes([lo, hi])
    }

    pub fn read32(&self, addr: u32) -> u32 {
        if Self::is_sram(addr) {
            return (self.read8(addr) as u32) * 0x01010101;
        }

        let b0 = self.read8(addr);
        let b1 = self.read8(addr + 1);
        let b2 = self.read8(addr + 2);
        let b3 = self.read8(addr + 3);
        u32::from_le_bytes([b0, b1, b2, b3])
    }

    /// Internal helper: performs plain write to memory with no side effects.
    fn raw_write8(&mut self, addr: u32, val: u8) {
        match addr >> 24 {
            0x00 => {} // BIOS is read-only
            0x02 => self.ewram[(addr as usize) & (EWRAM_SIZE - 1)] = val,
            0x03 => self.iwram[(addr as usize) & (IWRAM_SIZE - 1)] = val,
            0x04 => self.io[(addr as usize) & (IO_SIZE - 1)] = val,
            0x05 => self.palette[(addr as usize) & (PALETTE_SIZE - 1)] = val,
            0x06 => self.vram[Self::get_vram_offset(addr)] = val,
            0x07 => self.oam[(addr as usize) & (OAM_SIZE - 1)] = val,
            0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D => {} // ROM is read-only
            0x0E | 0x0F => {
                if !self.sram.is_empty() {
                    let mask = self.sram.len() - 1;
                    self.sram[(addr as usize) & mask] = val;
                }
            }
            _ => {}
        }
    }

    pub fn write8(&mut self, addr: u32, val: u8) {
        match addr >> 24 {
            0x05 => {
                // Palette: 8-bit duplication
                let aligned = addr & !1;
                self.raw_write8(aligned, val);
                self.raw_write8(aligned | 1, val);
            }
            0x06 => {
                // VRAM: 8-bit BG/OBJ split
                let offset = Self::get_vram_offset(addr);
                if offset < 0x10000 {
                    // Background
                    let aligned = addr & !1;
                    self.raw_write8(aligned, val);
                    self.raw_write8(aligned | 1, val);
                }
            }
            0x07 => {} // OAM: 8-bit write ignored
            _ => self.raw_write8(addr, val),
        }
    }

    pub fn write16(&mut self, addr: u32, val: u16) {
        if Self::is_sram(addr) {
            if !self.sram.is_empty() {
                let mask = self.sram.len() - 1;
                let byte_index = (addr & 1) as usize;
                self.sram[(addr as usize) & mask] = val.to_le_bytes()[byte_index];
            }
            return;
        }
        let bytes = val.to_le_bytes();
        self.raw_write8(addr, bytes[0]);
        self.raw_write8(addr + 1, bytes[1]);
    }

    pub fn write32(&mut self, addr: u32, val: u32) {
        if Self::is_sram(addr) {
            if !self.sram.is_empty() {
                let mask = self.sram.len() - 1;
                let byte_index = (addr & 3) as usize;
                self.sram[(addr as usize) & mask] = val.to_le_bytes()[byte_index];
            }
            return;
        }
        let bytes = val.to_le_bytes();
        self.raw_write8(addr, bytes[0]);
        self.raw_write8(addr + 1, bytes[1]);
        self.raw_write8(addr + 2, bytes[2]);
        self.raw_write8(addr + 3, bytes[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bus() -> Bus {
        let rom = vec![0; 1024]; // 1KB dummy ROM
        let sram = vec![0; 64 * 1024]; // 64KB dummy SRAM
        Bus::new(rom, sram)
    }

    #[test]
    fn test_bios_read() {
        let mut bus = create_test_bus();
        bus.bios[0x0005] = 0xAB;
        assert_eq!(bus.read8(0x0000_0005), 0xAB);
    }

    #[test]
    fn test_ewram_mirroring() {
        let mut bus = create_test_bus();
        bus.ewram[0] = 0x42;
        assert_eq!(bus.read8(0x0204_0000), 0x42);
    }

    #[test]
    fn test_vram_complex_mirroring() {
        let mut bus = create_test_bus();
        bus.vram[0x10000] = 0x99;
        assert_eq!(bus.read8(0x0601_8000), 0x99);

        bus.vram[0] = 0x77;
        assert_eq!(bus.read8(0x0602_0000), 0x77);
    }

    #[test]
    fn test_sram_replication() {
        let mut bus = create_test_bus();
        bus.sram[0] = 0xAA;

        assert_eq!(bus.read8(0x0E00_0000), 0xAA);
        assert_eq!(bus.read16(0x0E00_0000), 0xAAAA);
        assert_eq!(bus.read32(0x0E00_0000), 0xAAAAAAAA);
    }

    #[test]
    fn test_write_read_symmetry() {
        let mut bus = create_test_bus();
        bus.write16(0x0200_0000, 0x1234);
        assert_eq!(bus.read16(0x0200_0000), 0x1234);
    }

    #[test]
    fn test_readonly_noops() {
        let mut bus = create_test_bus();
        bus.write8(0x0000_0000, 0xAA);
        bus.write8(0x0800_0000, 0xBB);
        assert_eq!(bus.read8(0x0000_0000), 0x00);
        assert_eq!(bus.read8(0x0800_0000), 0x00);
    }

    #[test]
    fn test_oam_vram_write_quirks() {
        let mut bus = create_test_bus();
        bus.write8(0x0700_0000, 0xAA);
        assert_eq!(bus.read8(0x0700_0000), 0x00);
        bus.write16(0x0700_0000, 0x1234);
        assert_eq!(bus.read16(0x0700_0000), 0x1234);

        bus.write8(0x0600_0000, 0xCC);
        assert_eq!(bus.read16(0x0600_0000), 0xCCCC);
    }

    #[test]
    fn test_sram_rotation() {
        let mut bus = create_test_bus();
        bus.write16(0x0E00_0001, 0x1234);
        assert_eq!(bus.read8(0x0E00_0001), 0x12);

        bus.write32(0x0E00_0002, 0x12345678);
        assert_eq!(bus.read8(0x0E00_0002), 0x34);
    }
}
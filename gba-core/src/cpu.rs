#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    User = 0x10,
    Fiq = 0x11,
    Irq = 0x12,
    Supervisor = 0x13,
    Abort = 0x17,
    Undefined = 0x1B,
    System = 0x1F,
}

pub fn mode_from_bits(bits: u32) -> Option<Mode> {
    match bits & 0x1F {
        0x10 => Some(Mode::User),
        0x11 => Some(Mode::Fiq),
        0x12 => Some(Mode::Irq),
        0x13 => Some(Mode::Supervisor),
        0x17 => Some(Mode::Abort),
        0x1B => Some(Mode::Undefined),
        0x1F => Some(Mode::System),
        _ => None,
    }
}

pub struct Cpu {
    r: [u32; 16],
    cpsr: u32,
    r8_12_fiq: [u32; 5],
    r13_banked: [u32; 5],
    r14_banked: [u32; 5],
    spsr_banked: [u32; 5],
}

impl Mode {
    pub fn bank_index(self) -> Option<usize> {
        match self {
            Self::User | Self::System => None,
            Self::Fiq => Some(0),
            Self::Irq => Some(1),
            Self::Supervisor => Some(2),
            Self::Abort => Some(3),
            Self::Undefined => Some(4),
        }
    }
}

impl Cpu {
    /// Initializes the CPU to the hardware reset state:
    /// Supervisor mode, interrupts disabled, PC at 0.
    pub fn new() -> Self {
        Self {
            r: [0; 16],
            // Mode 0x13 (Supervisor) | I-bit (1<<7) | F-bit (1<<6)
            cpsr: 0x13 | (1 << 7) | (1 << 6),
            r8_12_fiq: [0; 5],
            r13_banked: [0; 5],
            r14_banked: [0; 5],
            spsr_banked: [0; 5],
        }
    }

    fn get_mode(&self) -> Mode {
        mode_from_bits(self.cpsr).expect("CPSR mode bits corrupted")
    }

    pub fn read_reg(&self, index: u32) -> u32 {
        let idx = (index & 0xF) as usize;
        let mode = self.get_mode();

        if idx == 15 {
            return self.r[15];
        }

        if (8..=12).contains(&idx) && mode == Mode::Fiq {
            return self.r8_12_fiq[idx - 8];
        }

        if idx == 13 || idx == 14 {
            if let Some(bank) = mode.bank_index() {
                return if idx == 13 { self.r13_banked[bank] } else { self.r14_banked[bank] };
            }
        }

        self.r[idx]
    }

    pub fn write_reg(&mut self, index: u32, value: u32) {
        let idx = (index & 0xF) as usize;
        let mode = self.get_mode();

        if idx == 15 {
            self.r[15] = value;
            return;
        }

        if (8..=12).contains(&idx) && mode == Mode::Fiq {
            self.r8_12_fiq[idx - 8] = value;
            return;
        }

        if idx == 13 || idx == 14 {
            if let Some(bank) = mode.bank_index() {
                if idx == 13 { self.r13_banked[bank] = value; } else { self.r14_banked[bank] = value; }
                return;
            }
        }

        self.r[idx] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_from_bits_valid() {
        assert_eq!(mode_from_bits(0x10), Some(Mode::User));
        assert_eq!(mode_from_bits(0x13), Some(Mode::Supervisor));
        assert_eq!(mode_from_bits(0x1F), Some(Mode::System));
    }

    #[test]
    fn test_mode_from_bits_invalid() {
        assert_eq!(mode_from_bits(0x00), None);
        assert_eq!(mode_from_bits(0x14), None); 
    }

    #[test]
    fn test_mode_from_bits_masking() {
        assert_eq!(mode_from_bits(0xFF_FF_FF_12), Some(Mode::Irq));
    }

    #[test]
    fn test_bank_index() {
        assert_eq!(Mode::User.bank_index(), None);
        assert_eq!(Mode::System.bank_index(), None);
        assert_eq!(Mode::Fiq.bank_index(), Some(0));
        assert_eq!(Mode::Undefined.bank_index(), Some(4));
    }

    #[test]
    fn test_register_access_scenarios() {
        let mut cpu = Cpu::new();

        // 1. Write to r0 in User mode (Unbanked)
        cpu.cpsr = Mode::User as u32;
        cpu.write_reg(0, 0xABC);
        assert_eq!(cpu.read_reg(0), 0xABC);

        // 2. Write to r8 in User vs FIQ (Banked)
        cpu.cpsr = Mode::User as u32;
        cpu.write_reg(8, 0x111);
        cpu.cpsr = Mode::Fiq as u32;
        assert_eq!(cpu.read_reg(8), 0, "FIQ r8 should be fresh");
        cpu.write_reg(8, 0x222);
        assert_eq!(cpu.read_reg(8), 0x222);
        cpu.cpsr = Mode::User as u32;
        assert_eq!(cpu.read_reg(8), 0x111);

        // 3. Write r13 in SVC vs ABT (Banked)
        cpu.cpsr = Mode::Supervisor as u32;
        cpu.write_reg(13, 0x333);
        cpu.cpsr = Mode::Abort as u32;
        assert_eq!(cpu.read_reg(13), 0, "Abort r13 should be fresh");
        cpu.write_reg(13, 0x444);
        assert_eq!(cpu.read_reg(13), 0x444);

        // 4. PC is never banked
        cpu.cpsr = Mode::User as u32;
        cpu.write_reg(15, 0x555);
        cpu.cpsr = Mode::Fiq as u32;
        assert_eq!(cpu.read_reg(15), 0x555, "PC should not be banked in FIQ");
    }
}
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
    // We only care about the bottom 5 bits (M4-M0)
    match bits & 0x1F {
        0x10 => Some(Mode::User),
        0x11 => Some(Mode::Fiq),
        0x12 => Some(Mode::Irq),
        0x13 => Some(Mode::Supervisor),
        0x17 => Some(Mode::Abort),
        0x1B => Some(Mode::Undefined),
        0x1F => Some(Mode::System),
        // Writing any other value into mode bits is not allowed
        _ => None, 
    }
}

pub struct Cpu {
    pub r: [u32; 16],
    pub cpsr: u32,
    pub r8_12_fiq: [u32;5],
    pub r13_banked: [u32; 5],
    pub r14_banked: [u32; 5],
    pub spsr_banked: [u32; 5],
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
        // Test a bit pattern that shouldn't match any valid mode
        assert_eq!(mode_from_bits(0x00), None);
        assert_eq!(mode_from_bits(0x14), None); 
    }

    #[test]
    fn test_mode_from_bits_masking() {
        // Test that upper bits are correctly ignored
        // 0xFF_FF_FF_12 should mask down to 0x12 (Irq)
        assert_eq!(mode_from_bits(0xFF_FF_FF_12), Some(Mode::Irq));
    }

    #[test]
    fn test_bank_index() {
        assert_eq!(Mode::User.bank_index(), None);
        assert_eq!(Mode::System.bank_index(), None);
        assert_eq!(Mode::Fiq.bank_index(), Some(0));
        assert_eq!(Mode::Undefined.bank_index(), Some(4));
    }
}
use std::convert::TryFrom;
use crate::error::DecodeError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Condition {
    Eq = 0x0,
    Ne = 0x1,
    Cs = 0x2,
    Cc = 0x3,
    Mi = 0x4,
    Pl = 0x5,
    Vs = 0x6,
    Vc = 0x7,
    Hi = 0x8,
    Ls = 0x9,
    Ge = 0xA,
    Lt = 0xB,
    Gt = 0xC,
    Le = 0xD,
    Al = 0xE,
}

impl TryFrom<u8> for Condition {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Condition::Eq),
            0x1 => Ok(Condition::Ne),
            0x2 => Ok(Condition::Cs),
            0x3 => Ok(Condition::Cc),
            0x4 => Ok(Condition::Mi),
            0x5 => Ok(Condition::Pl),
            0x6 => Ok(Condition::Vs),
            0x7 => Ok(Condition::Vc),
            0x8 => Ok(Condition::Hi),
            0x9 => Ok(Condition::Ls),
            0xA => Ok(Condition::Ge),
            0xB => Ok(Condition::Lt),
            0xC => Ok(Condition::Gt),
            0xD => Ok(Condition::Le),
            0xE => Ok(Condition::Al),
            // Catch 0xF AND anything larger than 15 without silently masking
            _ => Err(DecodeError::InvalidCondition(value)),
        }
    }
}

pub fn evaluate_cond(cond: Condition, n: bool, z: bool, c: bool, v: bool) -> bool {
    match cond {
        Condition::Eq => z,
        Condition::Ne => !z,
        Condition::Cs => c,
        Condition::Cc => !c,
        Condition::Mi => n,
        Condition::Pl => !n,
        Condition::Vs => v,
        Condition::Vc => !v,
        Condition::Hi => c && !z,
        Condition::Ls => !c || z,
        Condition::Ge => n == v,
        Condition::Lt => n != v,
        Condition::Gt => !z && (n == v),
        Condition::Le => z || (n != v),
        Condition::Al => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_try_from() {
        assert_eq!(Condition::try_from(0x0), Ok(Condition::Eq));
        assert_eq!(Condition::try_from(0xE), Ok(Condition::Al));

        // 0xF is an invalid condition
        assert_eq!(Condition::try_from(0xF), Err(DecodeError::InvalidCondition(0xF)));

        // Unmasked garbage values should also fail, not silently truncate!
        assert_eq!(Condition::try_from(0xF0), Err(DecodeError::InvalidCondition(0xF0)));
    }

    #[test]
    fn test_evaluate_simple_flags() {
        assert!(evaluate_cond(Condition::Eq, false, true, false, false));
        assert!(!evaluate_cond(Condition::Eq, false, false, false, false));
        assert!(evaluate_cond(Condition::Ne, false, false, false, false));

        assert!(evaluate_cond(Condition::Cs, false, false, true, false));
        assert!(evaluate_cond(Condition::Cc, false, false, false, false));

        assert!(evaluate_cond(Condition::Mi, true, false, false, false));
        assert!(evaluate_cond(Condition::Pl, false, false, false, false));

        assert!(evaluate_cond(Condition::Vs, false, false, false, true));
        assert!(evaluate_cond(Condition::Vc, false, false, false, false));
    }

    #[test]
    fn test_evaluate_compound_flags() {
        // HI: C=1 and Z=0
        assert!(evaluate_cond(Condition::Hi, false, false, true, false));
        assert!(!evaluate_cond(Condition::Hi, false, true, true, false));

        // LS: C=0 or Z=1
        assert!(evaluate_cond(Condition::Ls, false, true, false, false)); // Z=1
        assert!(evaluate_cond(Condition::Ls, false, false, false, false)); // C=0
        assert!(!evaluate_cond(Condition::Ls, false, false, true, false)); // C=1, Z=0

        // GE: N == V
        assert!(evaluate_cond(Condition::Ge, true, false, false, true));
        assert!(evaluate_cond(Condition::Ge, false, false, false, false));
        assert!(!evaluate_cond(Condition::Ge, true, false, false, false));

        // LT: N != V
        assert!(evaluate_cond(Condition::Lt, true, false, false, false)); // N=1, V=0
        assert!(!evaluate_cond(Condition::Lt, true, false, false, true)); // N=1, V=1

        // GT: Z=0 and N == V
        assert!(evaluate_cond(Condition::Gt, true, false, false, true));
        assert!(!evaluate_cond(Condition::Gt, true, true, false, true));

        // LE: Z=1 or N != V
        assert!(evaluate_cond(Condition::Le, false, true, false, false));
        assert!(evaluate_cond(Condition::Le, true, false, false, false));
        assert!(!evaluate_cond(Condition::Le, true, false, false, true));

        // AL: Always true
        assert!(evaluate_cond(Condition::Al, false, false, false, false));
        assert!(evaluate_cond(Condition::Al, true, true, true, true));
    }
}
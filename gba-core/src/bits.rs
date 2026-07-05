pub fn bit(value: u32, pos: u32) -> bool {
    (value >> pos) & 1 != 0
}

pub fn bits(value: u32, start: u32, end: u32) -> u32 {
    let width = end - start + 1;
    let mask = (1u32 << width) - 1;
    (value >> start) & mask
}

pub fn sign_extend(value: u32, num_bits: u32) -> i32 {
    let shift = 32 - num_bits;
    ((value << shift) as i32) >> shift
}

// Assertion tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit() {
        assert_eq!(bit(0b1010,1),true);
        assert_eq!(bit(0b1010, 2), false);
        assert_eq!(bit(0x8000_0000, 31), true);
    }

    #[test]
    fn test_bits(){
        assert_eq!(bits(0b110101, 2, 4), 5);
        assert_eq!(bits(0xFF, 0, 3), 15);
    }

    #[test]
    fn test_sign_extend(){
        assert_eq!(sign_extend(0b111, 3), -1);
        assert_eq!(sign_extend(0b0101, 4), 5);
        assert_eq!(sign_extend(0x800, 12), -2048);
    }
}
pub trait SignExtend {
    fn sign_extend(self, bit_count: u8) -> u16;
}

impl SignExtend for u16 {
    fn sign_extend(self, bit_count: u8) -> u16 {
        if ((self >> (bit_count - 1)) & 1) == 1 {
            self | (0xFFFF << bit_count)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extend_positive_number() {
        assert_eq!(0b01010.sign_extend(5), 0b0000_0000_0000_1010);
    }

    #[test]
    fn sign_extend_negative_number() {
        assert_eq!(0b10101.sign_extend(5), 0b1111_1111_1111_0101);
    }
}

pub(crate) fn sign_extend(mut value: u16, bit_count: u8) -> u16 {
    if ((value >> (bit_count - 1)) & 1) == 1 {
        value |= 0xFFFF << bit_count;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extend_positive_number() {
        assert_eq!(sign_extend(0b01010, 5), 0b0000_0000_0000_1010);
    }

    #[test]
    fn sign_extend_negative_number() {
        assert_eq!(sign_extend(0b10101, 5), 0b1111_1111_1111_0101);
    }
}

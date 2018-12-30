pub(crate) fn sign_extend(mut value: u16, bit_count: u8) -> u16 {
    if ((value >> (bit_count - 1)) & 1) == 1 {
        value |= 0xFFFF << bit_count;
    }
    value
}

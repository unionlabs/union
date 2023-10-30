/// Returns the four most significant bytes
#[allow(dead_code)] // only used in Wasm builds
#[inline]
pub fn from_high_half(data: u64) -> u32 {
    (data >> 32).try_into().unwrap()
}

/// Returns the four least significant bytes
#[allow(dead_code)] // only used in Wasm builds
#[inline]
pub fn from_low_half(data: u64) -> u32 {
    (data & 0xFFFFFFFF).try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_high_half_works() {
        assert_eq!(from_high_half(0), 0);
        assert_eq!(from_high_half(0x1122334455667788), 0x11223344);
    }

    #[test]
    fn from_low_haf_works() {
        assert_eq!(from_low_half(0), 0);
        assert_eq!(from_low_half(0x1122334455667788), 0x55667788);
    }
}

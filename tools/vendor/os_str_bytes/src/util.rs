pub(super) const BYTE_SHIFT: u8 = 6;

pub(super) const CONT_MASK: u8 = (1 << BYTE_SHIFT) - 1;

pub(super) const CONT_TAG: u8 = 0b1000_0000;

pub(super) const fn is_continuation(byte: u8) -> bool {
    byte & !CONT_MASK == CONT_TAG
}

use super::{Algorithm, Crc, Digest};
use crate::table::crc8_table;

impl Crc<u8> {
    pub const fn new(algorithm: &'static Algorithm<u8>) -> Self {
        let table = crc8_table(algorithm.width, algorithm.poly, algorithm.refin);
        Self { algorithm, table }
    }

    pub const fn checksum(&self, bytes: &[u8]) -> u8 {
        let mut crc = self.init(self.algorithm.init);
        crc = self.update(crc, bytes);
        self.finalize(crc)
    }

    const fn init(&self, initial: u8) -> u8 {
        if self.algorithm.refin {
            initial.reverse_bits() >> (8u8 - self.algorithm.width)
        } else {
            initial << (8u8 - self.algorithm.width)
        }
    }

    const fn table_entry(&self, index: u8) -> u8 {
        self.table[index as usize]
    }

    const fn update(&self, mut crc: u8, bytes: &[u8]) -> u8 {
        let mut i = 0;

        while i < bytes.len() {
            crc = self.table_entry(crc ^ bytes[i]);
            i += 1;
        }

        crc
    }

    const fn finalize(&self, mut crc: u8) -> u8 {
        if self.algorithm.refin ^ self.algorithm.refout {
            crc = crc.reverse_bits();
        }
        if !self.algorithm.refout {
            crc >>= 8u8 - self.algorithm.width;
        }
        crc ^ self.algorithm.xorout
    }

    pub const fn digest(&self) -> Digest<u8> {
        self.digest_with_initial(self.algorithm.init)
    }

    /// Construct a `Digest` with a given initial value.
    ///
    /// This overrides the initial value specified by the algorithm.
    /// The effects of the algorithm's properties `refin` and `width`
    /// are applied to the custom initial value.
    pub const fn digest_with_initial(&self, initial: u8) -> Digest<u8> {
        let value = self.init(initial);
        Digest::new(self, value)
    }
}

impl<'a> Digest<'a, u8> {
    const fn new(crc: &'a Crc<u8>, value: u8) -> Self {
        Digest { crc, value }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.value = self.crc.update(self.value, bytes);
    }

    pub const fn finalize(self) -> u8 {
        self.crc.finalize(self.value)
    }
}

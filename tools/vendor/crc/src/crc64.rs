use super::{Algorithm, Crc, Digest};
use crate::table::crc64_table;

impl Crc<u64> {
    pub const fn new(algorithm: &'static Algorithm<u64>) -> Self {
        let table = crc64_table(algorithm.width, algorithm.poly, algorithm.refin);
        Self { algorithm, table }
    }

    pub const fn checksum(&self, bytes: &[u8]) -> u64 {
        let mut crc = self.init(self.algorithm.init);
        crc = self.update(crc, bytes);
        self.finalize(crc)
    }

    const fn init(&self, initial: u64) -> u64 {
        if self.algorithm.refin {
            initial.reverse_bits() >> (64u8 - self.algorithm.width)
        } else {
            initial << (64u8 - self.algorithm.width)
        }
    }

    const fn table_entry(&self, index: u64) -> u64 {
        self.table[(index & 0xFF) as usize]
    }

    const fn update(&self, mut crc: u64, bytes: &[u8]) -> u64 {
        let mut i = 0;
        if self.algorithm.refin {
            while i < bytes.len() {
                let table_index = crc ^ bytes[i] as u64;
                crc = self.table_entry(table_index) ^ (crc >> 8);
                i += 1;
            }
        } else {
            while i < bytes.len() {
                let table_index = (crc >> 56) ^ bytes[i] as u64;
                crc = self.table_entry(table_index) ^ (crc << 8);
                i += 1;
            }
        }
        crc
    }

    const fn finalize(&self, mut crc: u64) -> u64 {
        if self.algorithm.refin ^ self.algorithm.refout {
            crc = crc.reverse_bits();
        }
        if !self.algorithm.refout {
            crc >>= 64u8 - self.algorithm.width;
        }
        crc ^ self.algorithm.xorout
    }

    pub const fn digest(&self) -> Digest<u64> {
        self.digest_with_initial(self.algorithm.init)
    }

    /// Construct a `Digest` with a given initial value.
    ///
    /// This overrides the initial value specified by the algorithm.
    /// The effects of the algorithm's properties `refin` and `width`
    /// are applied to the custom initial value.
    pub const fn digest_with_initial(&self, initial: u64) -> Digest<u64> {
        let value = self.init(initial);
        Digest::new(self, value)
    }
}

impl<'a> Digest<'a, u64> {
    const fn new(crc: &'a Crc<u64>, value: u64) -> Self {
        Digest { crc, value }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.value = self.crc.update(self.value, bytes);
    }

    pub const fn finalize(self) -> u64 {
        self.crc.finalize(self.value)
    }
}

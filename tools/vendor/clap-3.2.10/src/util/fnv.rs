use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

const MAGIC_INIT: u64 = 0x811C_9DC5;

// TODO: Docs
pub trait Key: Hash + Display {
    fn key(&self) -> u64;
}

impl<T> Key for T
where
    T: Hash + Display,
{
    fn key(&self) -> u64 {
        let mut hasher = FnvHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

pub(crate) struct FnvHasher(u64);

impl FnvHasher {
    pub(crate) fn new() -> Self {
        FnvHasher(MAGIC_INIT)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        let FnvHasher(mut hash) = *self;

        for byte in bytes.iter() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }

        *self = FnvHasher(hash);
    }
}

mod mmap;
mod owned;

pub use crate::{
    mmap::MmapError,
    owned::{OwnedBuffer, OwnedIntoIter},
};

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        hash::Hash,
        ops::Deref,
        path::{Path, PathBuf},
    };

    use bytes::{Buf, Bytes};

    use super::*;

    #[test]
    fn owned_api() {
        fn static_assert<'a, T>()
        where
            // Conversions
            T: TryFrom<&'a Path, Error = MmapError>,
            T: TryFrom<&'a PathBuf, Error = MmapError>,
            T: TryFrom<PathBuf, Error = MmapError>,
            T: TryFrom<&'a File, Error = MmapError>,
            T: TryFrom<File, Error = MmapError>,
            T: From<&'a Vec<u8>>,
            T: From<&'a [u8]>,
            T: From<Bytes>,
            // PartialEq
            T: PartialEq<[u8]>,
            T: PartialEq<Vec<u8>>,
            T: PartialEq<&'a Vec<u8>>,
            T: PartialEq<&'a [u8]>,
            T: PartialEq<[u8; 42]>,
            [u8]: PartialEq<T>,
            Vec<u8>: PartialEq<T>,
            &'a Vec<u8>: PartialEq<T>,
            &'a [u8]: PartialEq<T>,
            [u8; 42]: PartialEq<T>,
            // Commonly needed for collections
            T: Eq + Ord + Hash,
            // Misc
            T: Deref<Target = [u8]>,
            T: AsRef<[u8]>,
            T: Buf,
            &'a T: IntoIterator<Item = &'a u8> + 'a,
        {
        }

        static_assert::<OwnedBuffer>();
    }
}

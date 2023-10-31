#[cfg(feature = "v3")]
pub(crate) fn hash(ns: &[u8], src: &[u8]) -> [u8; 16] {
    use private_md_5::{Md5, Digest};

    let mut hasher = Md5::new();

    hasher.update(ns);
    hasher.update(src);

    let mut bytes = [0; 16];
    bytes.copy_from_slice(&hasher.finalize()[..16]);

    bytes
}

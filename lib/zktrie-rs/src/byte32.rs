use crate::{fr_from_big_endian, Fr, HashScheme, HASH_DOMAIN_BYTE32};

#[derive(Debug, PartialEq, Eq, Clone, Default, Ord, PartialOrd, Copy)]
#[repr(transparent)]
pub struct Byte32([u8; 32]);

impl From<[u8; 32]> for Byte32 {
    fn from(val: [u8; 32]) -> Self {
        Self(val)
    }
}

impl Byte32 {
    #[must_use]
    pub fn from_bytes_padding(mut b: &[u8]) -> Self {
        let mut bytes = [0_u8; 32];
        if b.len() > bytes.len() {
            b = &b[..bytes.len()];
        }
        let dst = if b.len() > bytes.len() {
            &mut bytes[..]
        } else {
            &mut bytes[..b.len()]
        };
        dst.copy_from_slice(b);
        bytes.into()
    }

    #[must_use]
    pub fn from_bytes(mut bytes: &[u8]) -> Self {
        let mut out = Self::default();
        if bytes.len() > 32 {
            let len = bytes.len();
            bytes = &bytes[len - 32..];
        }
        out.0[32 - bytes.len()..].copy_from_slice(bytes);
        out
    }

    pub fn hash<H: HashScheme>(&self) -> Result<Fr, String> {
        let first16 = fr_from_big_endian(&self.0[0..16])?;
        let last16 = fr_from_big_endian(&self.0[16..32])?;
        let domain = HASH_DOMAIN_BYTE32.into();
        Ok(H::hash_scheme(&[first16, last16], &domain))
    }

    pub fn fr(&self) -> Result<Fr, String> {
        fr_from_big_endian(&self.0)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn from_vec_bytes(data: &[u8]) -> Vec<Byte32> {
        let mut len = data.len() / 32;
        if data.len() % 32 != 0 {
            len += 1;
        }
        let mut out = vec![0_u8; len * 32];
        out[len * 32 - data.len()..].copy_from_slice(data);
        let ptr = out.as_ptr().cast::<Byte32>();
        unsafe { std::slice::from_raw_parts(ptr, len) }.to_owned()
    }
}

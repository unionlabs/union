use crate::{Error, HashScheme, Node};

lazy_static::lazy_static! {
    pub static ref MAGIC_HASH: &'static [u8] = b"THIS IS THE MAGIC INDEX FOR ZKTRIE";
    pub static ref MAGIC_SMT_BYTES: &'static [u8] = b"THIS IS SOME MAGIC BYTES FOR SMT m1rRXgP2xpDI";
}

pub fn decode_smt_proofs<H: HashScheme>(buf: &[u8]) -> Result<Option<Node<H>>, Error> {
    if MAGIC_SMT_BYTES.eq(buf) {
        return Ok(None);
    }
    Ok(Some(<Node<H>>::from_bytes(buf)?))
}

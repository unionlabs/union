use crate::{Error, HashScheme, Node};

pub const MAGIC_SMT_BYTES: &[u8] = b"THIS IS SOME MAGIC BYTES FOR SMT m1rRXgP2xpDI";

pub fn decode_smt_proofs<H: HashScheme>(buf: &[u8]) -> Result<Option<Node<H>>, Error> {
    if MAGIC_SMT_BYTES.eq(buf) {
        return Ok(None);
    }
    Ok(Some(<Node<H>>::from_bytes(buf)?))
}

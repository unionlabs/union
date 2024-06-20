use sha2::Digest;
use unionlabs::cosmos::ics23::hash_op::HashOp;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum HashError {
    #[error(
        "supported hash ops are ([Sha256, Sha512, Ripemd160, Bitcoin, Sha512256]) but found ({0})"
    )]
    UnsupportedOp(HashOp),
}

pub fn do_hash(hash_op: HashOp, preimage: &[u8]) -> Result<Vec<u8>, HashError> {
    let hash = match hash_op {
        HashOp::Sha256 => sha2::Sha256::new()
            .chain_update(preimage)
            .finalize()
            .to_vec(),
        HashOp::Sha512 => sha2::Sha512::new()
            .chain_update(preimage)
            .finalize()
            .to_vec(),
        HashOp::Ripemd160 => ripemd::Ripemd160::new()
            .chain_update(preimage)
            .finalize()
            .to_vec(),
        HashOp::Bitcoin => ripemd::Ripemd160::new()
            .chain_update(sha2::Sha256::new().chain_update(preimage).finalize())
            .finalize()
            .to_vec(),
        HashOp::Sha512256 => sha2::Sha512_256::new()
            .chain_update(preimage)
            .finalize()
            .to_vec(),
        op => return Err(HashError::UnsupportedOp(op)),
    };

    Ok(hash)
}

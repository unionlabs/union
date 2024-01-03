use sha2::Digest;
use unionlabs::cosmos::ics23::hash_op::HashOp;

pub fn do_hash_or_noop(hash_op: HashOp, preimage: &[u8]) -> Vec<u8> {
    if hash_op == HashOp::NoHash {
        return preimage.into();
    }

    do_hash(hash_op, preimage)
}

pub fn do_hash(hash_op: HashOp, preimage: &[u8]) -> Vec<u8> {
    match hash_op {
        // TODO(aeryz): why not keccak
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
            .chain_update(
                sha2::Sha256::new()
                    .chain_update(preimage)
                    .finalize()
                    .to_vec(),
            )
            .finalize()
            .to_vec(),
        HashOp::Sha512256 => sha2::Sha512_256::new()
            .chain_update(preimage)
            .finalize()
            .to_vec(),
        _ => panic!("unsupported"),
    }
}

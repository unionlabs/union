use crate::errors::Error;
use ethereum_consensus::primitives::Root;
use rlp::Rlp;
use tiny_keccak::{Hasher, Keccak};

/// decode rlp format `List<List>` to `Vec<List>`
pub fn decode_eip1184_rlp_proof(proof: Vec<u8>) -> Result<Vec<Vec<u8>>, Error> {
    let r = Rlp::new(&proof);
    let mut proofs: Vec<Vec<u8>> = Vec::new();
    if r.is_list() {
        for r_item in r.into_iter() {
            let proof: Vec<Vec<u8>> = r_item
                .as_list()
                .map_err(|_| Error::decode("cannot get proofs from rlp in `decode_eip1184`"))?;
            proofs.push(rlp::encode_list::<Vec<u8>, Vec<u8>>(&proof).into())
        }
        Ok(proofs)
    } else {
        Err(Error::InvalidProofFormat)
    }
}

pub fn extract_storage_root_from_account(account_rlp: &[u8]) -> Result<Root, Error> {
    let r = Rlp::new(account_rlp);
    if !r.is_list() {
        let items: Vec<Vec<u8>> = r.as_list().map_err(|_| {
            Error::decode("cannot get items from rlp in `extract_storage_root_from_account`")
        })?;
        if items.len() != 4 {
            Err(Error::InvalidProofFormat)
        } else {
            Ok(Root::try_from(items.get(2).ok_or(Error::InvalidProofFormat)?.as_slice()).unwrap())
        }
    } else {
        Err(Error::InvalidProofFormat)
    }
}

fn keccak_256(input: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut k = Keccak::v256();
    k.update(input);
    k.finalize(&mut out);
    out
}

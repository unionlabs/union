use serde::{Deserialize, Serialize};

use crate::{hash::H256, uint::U256};

/*
(The following scheme assumes the big-endian encoding)
[0:32] (bytes in big-endian)
[0:16] Reserved with all 0
[16:24] CodeSize, uint64 in big-endian
[24:32] Nonce, uint64 in big-endian
[32:64] Balance
[64:96] StorageRoot
[96:128] KeccakCodeHash
[128:160] PoseidonCodehash
(total 160 bytes)
 */
// https://github.com/scroll-tech/zktrie/blob/a12f2f262ad3e82301e39ecdf9bfe235befc7074/docs/zktrie.md
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Account {
    pub code_size: u64,
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: H256,
    pub keccak_code_hash: H256,
    pub poseidon_code_hash: H256,
}

impl TryFrom<&[u8]> for Account {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value = <[u8; 160]>::try_from(value).map_err(|_| ())?;
        Ok(Account {
            code_size: u64::from_be_bytes(value[16..24].try_into().expect("impossible")),
            nonce: u64::from_be_bytes(value[24..32].try_into().expect("impossible")),
            balance: U256::from_big_endian(value[32..64].try_into().expect("impossible")),
            storage_root: H256(value[64..96].try_into().expect("impossible")),
            keccak_code_hash: H256(value[96..128].try_into().expect("impossible")),
            poseidon_code_hash: H256(value[128..160].try_into().expect("impossible")),
        })
    }
}

use crate::{
    errors::{ExpectedLength, InvalidLength},
    hash::H256,
    uint::U256,
    ByteArrayExt,
};

// (The following scheme assumes the big-endian encoding)
// [0:32] (bytes in big-endian)
// [0:16] Reserved with all 0
// [16:24] CodeSize, uint64 in big-endian
// [24:32] Nonce, uint64 in big-endian
// [32:64] Balance
// [64:96] StorageRoot
// [96:128] KeccakCodeHash
// [128:160] PoseidonCodehash
// (total 160 bytes)
// https://github.com/scroll-tech/zktrie/blob/a12f2f262ad3e82301e39ecdf9bfe235befc7074/docs/zktrie.md
pub struct Account {
    pub code_size: u64,
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: H256,
    pub keccak_code_hash: H256,
    pub poseidon_code_hash: H256,
}

impl Account {
    pub fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength> {
        let value = value.as_ref();
        let value = <[u8; 160]>::try_from(value).map_err(|_| InvalidLength {
            expected: ExpectedLength::Exact(160),
            found: value.len(),
        })?;
        Ok(Account {
            code_size: u64::from_be_bytes(value.array_slice::<16, 8>()),
            nonce: u64::from_be_bytes(value.array_slice::<24, 8>()),
            balance: U256::from_be_bytes(value.array_slice::<32, 32>()),
            storage_root: H256(value.array_slice::<64, 32>()),
            keccak_code_hash: H256(value.array_slice::<96, 32>()),
            poseidon_code_hash: H256(value.array_slice::<128, 32>()),
        })
    }
}

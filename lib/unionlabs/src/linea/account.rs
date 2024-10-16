use crate::{
    errors::{ExpectedLength, InvalidLength},
    hash::H256,
    uint::U256,
    ByteArrayExt,
};

pub const ZKACCOUNT_BYTES_LEN: usize = 32 * 6;

// https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/core/src/main/java/net/consensys/shomei/ZkAccount.java
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ZkAccount {
    pub nonce: U256,
    pub balance: U256,
    pub storage_root: H256,
    pub mimc_code_hash: H256,
    pub keccak_code_hash: MimcSafeBytes,
    pub code_size: U256,
}

impl ZkAccount {
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.into()
    }

    pub fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength> {
        let value = value.as_ref();
        let value = <[u8; ZKACCOUNT_BYTES_LEN]>::try_from(value).map_err(|_| InvalidLength {
            expected: ExpectedLength::Exact(ZKACCOUNT_BYTES_LEN),
            found: value.len(),
        })?;
        Ok(ZkAccount {
            nonce: U256::from_be_bytes(value.array_slice::<0, 32>()),
            balance: U256::from_be_bytes(value.array_slice::<32, 32>()),
            storage_root: value.array_slice::<64, 32>().into(),
            mimc_code_hash: value.array_slice::<96, 32>().into(),
            keccak_code_hash: value.array_slice::<128, 32>().into(),
            code_size: U256::from_be_bytes(value.array_slice::<160, 32>()),
        })
    }
}

impl From<ZkAccount> for Vec<u8> {
    fn from(value: ZkAccount) -> Vec<u8> {
        [
            value.nonce.to_be_bytes().as_ref(),
            value.balance.to_be_bytes().as_ref(),
            &value.storage_root.into_bytes(),
            &value.mimc_code_hash.into_bytes(),
            &value.keccak_code_hash.into_bytes(),
            value.code_size.to_be_bytes().as_ref(),
        ]
        .concat()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct MimcSafeBytes {
    pub lsb: H256,
    pub msb: H256,
}

impl MimcSafeBytes {
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        [*self.lsb.get(), *self.msb.get()].concat()
    }
}

impl From<[u8; 32]> for MimcSafeBytes {
    fn from(value: [u8; 32]) -> Self {
        let mut lsb = [0u8; 32];
        let mut msb = [0u8; 32];
        lsb[16..32].copy_from_slice(&value.array_slice::<16, 16>());
        msb[16..32].copy_from_slice(&value.array_slice::<0, 16>());
        Self {
            lsb: lsb.into(),
            msb: msb.into(),
        }
    }
}

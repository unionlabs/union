use unionlabs::primitives::{Bytes, U256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct StorageProof {
    pub key: U256,
    pub value: U256,
    pub proof: Vec<Bytes>,
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{Bincode, Json},
        primitives::U256,
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_storage_proof() -> StorageProof {
        StorageProof {
            key: U256::from(123u64),
            value: U256::from(123u64),
            proof: vec![b"proof".into()],
        }
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_storage_proof());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_storage_proof());
    }
}

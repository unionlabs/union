use macros::model;

use crate::cosmos::ics23::commitment_proof::{CommitmentProof, TryFromCommitmentProofError};

#[model(proto(raw(protos::ibc::core::commitment::v1::MerkleProof), into, from))]
pub struct MerkleProof {
    pub proofs: Vec<CommitmentProof>,
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromMerkleProofError {
    #[error("invalid proofs")]
    Proofs(#[from] TryFromCommitmentProofError),
}

impl TryFrom<protos::ibc::core::commitment::v1::MerkleProof> for MerkleProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::ibc::core::commitment::v1::MerkleProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proofs: value
                .proofs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromMerkleProofError::Proofs)?,
        })
    }
}

impl From<MerkleProof> for protos::ibc::core::commitment::v1::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_json_roundtrip;

    #[test]
    fn json() {
        assert_json_roundtrip::<MerkleProof>(&serde_json::from_str(r#"{"proofs":[{"@type":"exist","@value":{"key":"0x636f6e6e656374696f6e732f636f6e6e656374696f6e2d34","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002fe14","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204fe1420ef76e6b775ddb12b9266a1c6be8f8a0465334edc5d63c4d50ec08acd3222b87120","suffix":"0x0"},{"hash":"sha256","prefix":"0x0408fe14205f20a52bf4eaf1f74bc2fe181f8ffde4007e02431bb098de85f583e7fd2ffab220","suffix":"0x0"},{"hash":"sha256","prefix":"0x060cfe1420d4c306fc76462b92c3888a22f78962e742022a4fa4b272947acf199993820a0720","suffix":"0x0"},{"hash":"sha256","prefix":"0x0814fe1420","suffix":"0x205da3a959acc11540519acc75cddcb1a953daa64ae13c029b0acacb7d3bf42f48"},{"hash":"sha256","prefix":"0x0a26fe14207288844aa9b92616031aa88ac045762ffca46300497d57256576f9e4481a162520","suffix":"0x0"}],"value":"0x0a0930382d7761736d2d3112140a0131120f4f524445525f554e4f524445524544180122130a0a636f6d6574626c732d301a050a03696263"}},{"@type":"exist","@value":{"key":"0x696263","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x01","suffix":"0x2cd8b50700950546180ad979135a8708c2ea2098fff6ade31b7e40eb5dcf7c05"},{"hash":"sha256","prefix":"0x012cf3feea58fcdb48b73c2cdd1b018c90c4078f924385675a0e9457168cd47ff1","suffix":"0x0"},{"hash":"sha256","prefix":"0x01668a26eca4c2d85af70ee4cec96ab6fd1dc8c30f6e77964b6ed44a33b5a16d8c","suffix":"0x0"},{"hash":"sha256","prefix":"0x01074d5d4ae17c3219a53737ff36d1528bcd934b9ac7d9a7fc75257464bfbc8b8c","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0x5b70af779b3bfd3c770d0ba038f05691478946ec8336226dc7ca5b6d3c08d9e3"}],"value":"0x3befa6eab5974ef0ba3fca174a9718704a98204f80476042e6ffcf41060cccb9"}}]}"#).unwrap());
    }
}

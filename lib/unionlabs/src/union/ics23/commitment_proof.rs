use macros::model;

use crate::{
    errors::{required, MissingField},
    union::ics23::{
        existence_proof::{ExistenceProof, TryFromExistenceProofError},
        non_existence_proof::{NonExistenceProof, TryFromNonExistenceProofError},
    },
};

#[model(proto(raw(protos::cosmos::ics23::v1::CommitmentProof), into, from))]
pub enum CommitmentProof {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
}

#[cfg(feature = "ethabi")]
impl crate::encoding::Encode<crate::encoding::EthAbi> for CommitmentProof {
    fn encode(self) -> Vec<u8> {
        match self {
            CommitmentProof::Exist(exist) => {
                crate::encoding::Encode::<crate::encoding::EthAbi>::encode(exist)
            }
            CommitmentProof::Nonexist(nonexist) => {
                crate::encoding::Encode::<crate::encoding::EthAbi>::encode(nonexist)
            }
        }
    }
}

// #[cfg(feature = "ethabi")]
// impl crate::encoding::Decode<crate::encoding::EthAbi> for CommitmentProof {
//     type Error = ethers::abi::Error;

//     fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
//         ExistenceProof
//     }
// }

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromCommitmentProofError {
    #[error(transparent)]
    MissingField(MissingField),
    #[error("unable to decode existence proof")]
    Exist(#[from] TryFromExistenceProofError),
    #[error("unable to decode non existence proof")]
    Nonexist(#[from] TryFromNonExistenceProofError),
    #[error("batch proofs are not supported")]
    BatchNotSupported,
    #[error("compressed batch proofs are not supported")]
    CompressedBatchNotSupported,
}

impl TryFrom<protos::cosmos::ics23::v1::CommitmentProof> for CommitmentProof {
    type Error = TryFromCommitmentProofError;

    fn try_from(value: protos::cosmos::ics23::v1::CommitmentProof) -> Result<Self, Self::Error> {
        match required!(value.proof)? {
            protos::cosmos::ics23::v1::commitment_proof::Proof::Exist(exist) => {
                Ok(CommitmentProof::Exist(exist.try_into()?))
            }
            protos::cosmos::ics23::v1::commitment_proof::Proof::Nonexist(nonexist) => {
                Ok(CommitmentProof::Nonexist(nonexist.try_into()?))
            }
            protos::cosmos::ics23::v1::commitment_proof::Proof::Batch(_) => {
                Err(TryFromCommitmentProofError::BatchNotSupported)
            }
            protos::cosmos::ics23::v1::commitment_proof::Proof::Compressed(_) => {
                Err(TryFromCommitmentProofError::CompressedBatchNotSupported)
            }
        }
    }
}

impl From<CommitmentProof> for protos::cosmos::ics23::v1::CommitmentProof {
    fn from(value: CommitmentProof) -> Self {
        Self {
            proof: Some(match value {
                CommitmentProof::Exist(exist) => {
                    protos::cosmos::ics23::v1::commitment_proof::Proof::Exist(exist.into())
                }
                CommitmentProof::Nonexist(nonexist) => {
                    protos::cosmos::ics23::v1::commitment_proof::Proof::Nonexist(nonexist.into())
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use ethers::abi::{AbiType, Tokenize};
    use hex_literal::hex;

    use super::*;
    use crate::encoding::{DecodeAs, EncodeAs, EthAbi, Proto};

    #[test]
    fn valid() {
        const BZ: &[u8] = &hex!("0aab020a147a785a4e6b534c64634d655657526c7658456644121e76616c75655f666f725f7a785a4e6b534c64634d655657526c76584566441a090801180120012a0100222508011221012634b831468dbafb1fc61a979c348ff8462da9a7d550191a6afc916ade16cc9922250801122101ab814d419bfc94ee9920d0ce993ce5da011e43613daf4b6f302855760083d7dd222508011221015a1568c73eaeaba567a6b2b2944b0e9a0228c931884cb5942f58ed835b8a7ac522250801122101a171412db5ee84835ef247768914e835ff80b7711e4aa8060871c2667ec3ea2922250801122101f9c2491884de24fb61ba8f358a56b306a8989bd35f1f8a4c8dabce22f703cc14222508011221012f12a6aa6270eff8a1628052938ff5e36cfcc5bf2eaedc0941ee46398ebc7c38");

        let proof = CommitmentProof::decode_as::<Proto>(BZ).unwrap();

        dbg!(&proof);

        let ethabi_bytes = proof.encode_as::<EthAbi>();

        println!("{}", hex::encode(ethabi_bytes));

        let bz = hex::decode("12c0070a14544f31483668784a4b667136547a56767649ffff12cf030a14544f31483668784a4b667136547a567676497747121e76616c75655f666f725f544f31483668784a4b667136547a5676764977471a090801180120012a01002225080112210143e19cb5e5dab017734caa78a2e2bccbb4797b7dc5a91abeab630c66fa6b162522250801122101b575404a1bb42b0fef8ae7f217af88aec769f7d66b5bc4b2913e74d651365473222508011221017c22dc50e866f9a1dce517ea01621161cecd70f4bdcd024b5a392746a1c8dc2622250801122101578105344f2c98c323ba0b8ca31e75aaa2b865cc389681e300b14d1c20713796222708011201011a20895c070c14546ecef7f5cb3a4bda1fd436a0ff99190f90bd037cbeaf52b2ffc1222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe1ad5030a14544f433344683150664f76657538585166635778121e76616c75655f666f725f544f433344683150664f766575385851666357781a090801180120012a0100222708011201011a20415d4cfaed0bfc98ac32acc219a8517bfa1983a15cc742e8b2f860167484bd46222708011201011a2098d853d9cc0ee1d2162527f660f2b90ab55b13e5534f1b7753ec481d7901d3ec222708011201011a20b5113e6000c5411b7cfa6fd09b6752a43de0fcd3951ed3b154d162deb53224a2222708011201011a208ce18cd72cc83511cb8ff706433f2fa4208c85b9f4c8d0ed71a614f24b89ae6c22250801122101c611244fe6b5fda4257615902eb24c14efcd9708c7c875d1ac5e867767aa1eab222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe").unwrap();

        let proof = CommitmentProof::decode_as::<Proto>(&bz).unwrap();

        dbg!(&proof);

        let ethabi_bytes = proof.encode_as::<EthAbi>();

        println!("{}", hex::encode(ethabi_bytes));

        let ep_tokens = crate::union::ics23::existence_proof::ExistenceProofEthAbi {
            key: vec![].into(),
            value: vec![].into(),
            leaf_prefix: vec![].into(),
            path: vec![],
        }
        .into_tokens();

        dbg!(&ep_tokens);
        dbg!(hex::encode(ethers::abi::ethabi::encode(&ep_tokens)));

        dbg!(crate::union::ics23::existence_proof::ExistenceProofEthAbi::param_type().to_string());
    }
}

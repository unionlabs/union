use macros::model;

use crate::{
    errors::{ExpectedLength, InvalidLength, MissingField},
    union::ics23::{existence_proof::ExistenceProof, non_existence_proof::NonExistenceProof},
};

#[model(proto(raw(protos::ibc::core::commitment::v1::MerkleProof), into, from))]
// TODO: Rename to optimized merkle proof
pub enum MerkleProof {
    Membership(ExistenceProof, ExistenceProof),
    NonMembership(NonExistenceProof, ExistenceProof),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromMerkleProofError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("unable to decode existence proof")]
    Existence(#[from] crate::union::ics23::existence_proof::TryFromExistenceProofError),
    #[error("unable to decode non existence proof")]
    NonExistence(#[from] crate::union::ics23::non_existence_proof::TryFromNonExistenceProofError),
    #[error("invalid commitment proof type")]
    InvalidCommitmentProofType,
    #[error("invalid proofs length")]
    InvalidProofsLength(#[from] InvalidLength),
}

impl TryFrom<protos::ibc::core::commitment::v1::MerkleProof> for MerkleProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::ibc::core::commitment::v1::MerkleProof,
    ) -> Result<Self, Self::Error> {
        use protos::cosmos::ics23::v1::{
            commitment_proof::Proof as RawProof, CommitmentProof as RawCommitmentProof,
        };

        let proofs: [_; 2] = value.proofs.try_into().map_err(|invalid: Vec<_>| {
            TryFromMerkleProofError::InvalidProofsLength(InvalidLength {
                expected: ExpectedLength::Exact(2),
                found: invalid.len(),
            })
        })?;

        match proofs {
            [RawCommitmentProof {
                proof: Some(RawProof::Exist(exist_1)),
            }, RawCommitmentProof {
                proof: Some(RawProof::Exist(exist_2)),
            }] => Ok(Self::Membership(exist_1.try_into()?, exist_2.try_into()?)),
            [RawCommitmentProof {
                proof: Some(RawProof::Nonexist(non_exist)),
            }, RawCommitmentProof {
                proof: Some(RawProof::Exist(exist)),
            }] => Ok(Self::NonMembership(
                non_exist.try_into()?,
                exist.try_into()?,
            )),
            [_, _] => Err(TryFromMerkleProofError::InvalidCommitmentProofType),
        }
    }
}

impl From<MerkleProof> for protos::ibc::core::commitment::v1::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        use protos::cosmos::ics23::v1::{
            commitment_proof::Proof as RawProof, CommitmentProof as RawCommitmentProof,
        };

        match value {
            MerkleProof::Membership(a, b) => Self {
                proofs: vec![
                    RawCommitmentProof {
                        proof: Some(RawProof::Exist(a.into())),
                    },
                    RawCommitmentProof {
                        proof: Some(RawProof::Exist(b.into())),
                    },
                ],
            },
            MerkleProof::NonMembership(a, b) => Self {
                proofs: vec![
                    RawCommitmentProof {
                        proof: Some(RawProof::Nonexist(a.into())),
                    },
                    RawCommitmentProof {
                        proof: Some(RawProof::Exist(b.into())),
                    },
                ],
            },
        }
    }
}

#[cfg(feature = "ethabi")]
impl crate::encoding::Encode<crate::encoding::EthAbi> for MerkleProof {
    fn encode(self) -> Vec<u8> {
        use ethers::abi::Tokenizable;

        use crate::union::ics23::{
            existence_proof::ExistenceProofEthAbi, non_existence_proof::NonExistenceProofEthAbi,
        };

        match self {
            MerkleProof::Membership(a, b) => ethers::abi::ethabi::encode(&[
                ExistenceProofEthAbi::from(a).into_token(),
                ExistenceProofEthAbi::from(b).into_token(),
            ]),
            MerkleProof::NonMembership(a, b) => ethers::abi::ethabi::encode(&[
                NonExistenceProofEthAbi::from(a).into_token(),
                ExistenceProofEthAbi::from(b).into_token(),
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use ethers::abi::{ethabi, AbiType, Token, Tokenizable};
    use hex_literal::hex;

    use super::*;
    use crate::{
        encoding::{DecodeAs, EncodeAs, EthAbi, Proto},
        union::ics23::{
            existence_proof::ExistenceProofEthAbi, non_existence_proof::NonExistenceProofEthAbi,
        },
    };

    #[test]
    fn ethabi() {
        proto_eth_roundtrip(&hex::decode("0a96061293060a15014152090b0c95c948edc407995560feed4a9df88812fa020a15014152090b0c95c948edc407995560feed4a9df81e129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c122180420031a0b0801180120012a0300027822290801122502047820170c890f01b9fa9ab803511bbc7be7c25359309f04d021a72e0a9b93b8ff72c020222c0801120504089601201a21205f282a80f1d186fa1f7b237f81e8bc9a4bb40d5a03cbbdffdd421b1ad4cb16f4222c0801120506109601201a2120e9c65294b7106c7323dcabe4532232c319afc78cd373e338f12df43f8ecfa909222c080112050a309601201a2120a95af7890dba33514ea28a3db7b409f4887b058d6d1e43960c4cd45bb1d9bef81afc020a150143e46d91544517a037a8029b6c7f86f62bab389b129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e3167306a786d79323567357436716461677132646b636c7578376334366b77796d38646563667712460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034611ea6606f6241fdeb0db1854a785eaa2fef5770694237daaf46057cadb3903180320031a0c0801180120012a0400029601222c0801120502049601201a2120532543090d1564b206e953fd6f97000d9b78bd5a8a424f551d483a58b3f54c57222a0801122604089601207e55a1ee8006e9c29c895a8de8ea8cdc6aaddc10e05ea3d3ee8fac786a73c02d20222c0801120506109601201a2120e9c65294b7106c7323dcabe4532232c319afc78cd373e338f12df43f8ecfa909222c080112050a309601201a2120a95af7890dba33514ea28a3db7b409f4887b058d6d1e43960c4cd45bb1d9bef80a80020afd010a0361636312205281c416bf4f80b9d99428a09f91ff311968a3b2adb199342d63c9db20a417e91a090801180120012a010022250801122101ba30cf8122e71a87fea08d0da9499e0373495a64e1648de8f08ca1a73e1fc1a8222708011201011a203489cd05a389a1d165f19003cea0994df9e55a5cb53b3d659417040be528b86d222708011201011a20e5c60ddccacb1c6b0be7957e8d7a86dc0f8bcec91c91d666d39eb1ebedd1bdf1222708011201011a2047a4c9a64496594e8b255443aa979293b2c7120150cf31e0eeeb8a2a987fd7e8222708011201011a2053bca15bed4becbdfd1b4cd0e63bd3845646022a99a2289a6678d8608f092207").unwrap());
        proto_eth_roundtrip(&hex::decode("0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7").unwrap());
        proto_eth_roundtrip(&hex::decode("0abc020ab9020a18636f6e6e656374696f6e732f636f6e6e656374696f6e2d31125b0a0930382d7761736d2d3112230a0131120d4f524445525f4f524445524544120f4f524445525f554e4f524445524544180222250a0e636f6d6574626c732d6e65772d30120c636f6e6e656374696f6e2d301a050a0369626328061a0c0801180120012a040002f006222c080112050204f006201a212075c4910f51207d3c65960120fe931f138e2624668d75869f51b8442593dd5eab222a080112260408de0a2002b6fcf07091245d162f1196b003c555c564980e02c4d4a9fa0a249798f4b25e20222c08011205060ede0a201a2120ff6b0a04e076eecbabfee4e751c0523cbedba898211b5847404e2d954a2203e3222a08011226081ede0a20635053419cfb6a81c839860d99f3ed002840124a790ddd9f066d8bce63f9df54200afc010af9010a03696263122024b15e198bcf648dee62c7ca1fd8c3950c85c3d898833180c3e3c412ccbc559d1a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc222508011221016376cbd7b917c7105ddac35bdeddd79e6c9cbbc66dd227941599de2b9bc8b3de222708011201011a200d68ac7c3e8daf94c65ccdfe5b7397f50e80325240ef9b2a0ec483afaea30544").unwrap());
        proto_eth_roundtrip(&hex::decode("0afa020af7020a15014152090b0c95c948edc407995560feed4a9df81e129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c122180120011a0b0801180120012a0300020222290801122502040220170c890f01b9fa9ab803511bbc7be7c25359309f04d021a72e0a9b93b8ff72c020222b08011204040802201a2120a89a7b1aedf861a8c6316009af3d19448bfe8834dfb5546c7e1af7f95c3000b4222b08011204061002201a212029347d33c119e85fc1335f43ad17c4a1986ad44c71837158ceffd36e2f38f986222b080112040a3002201a2120e284b7ed0385d018b1ffcd6f33bf6ac575fb7731704d0ae71be278bd8bf5e0b50a80020afd010a03616363122082d7d632a58654a81bb6764379eff4b6e641e96620a12dac0e250e6caf94f7761a090801180120012a010022250801122101ba30cf8122e71a87fea08d0da9499e0373495a64e1648de8f08ca1a73e1fc1a8222708011201011a208a19e0585632ebada293099d24f28707d453266ae7ded6e854dfd8a025c7ce71222708011201011a204a22410f42f7706402b38c460e74d712c95cea8e6e370c691f43c0abf3f4e104222708011201011a20b999d9a62cbd36a843f207580c4802d194e6441f7f3715ddce55d5194d46e57a222708011201011a2022ecbf124eff995ecf01998dd8346b71810af164e192feeb4d4287085128b9df").unwrap());
    }

    fn proto_eth_roundtrip(bz: &[u8]) {
        let proof = MerkleProof::decode_as::<Proto>(bz).unwrap();
        let ethabi_bz = proof.clone().encode_as::<EthAbi>();

        match proof {
            MerkleProof::Membership(exist_1, exist_2) => {
                println!("{}", hex::encode(&ethabi_bz));

                let [exist_1_tokens, exist_2_tokens]: [Token; 2] = ethabi::decode(
                    &[
                        ExistenceProofEthAbi::param_type(),
                        ExistenceProofEthAbi::param_type(),
                    ],
                    &ethabi_bz,
                )
                .unwrap()
                .try_into()
                .unwrap();

                let exist_1_ethabi = ExistenceProofEthAbi::from_token(exist_1_tokens).unwrap();
                let exist_2_ethabi = ExistenceProofEthAbi::from_token(exist_2_tokens).unwrap();

                assert_eq!(exist_1, exist_1_ethabi.into());
                assert_eq!(exist_2, exist_2_ethabi.into());
            }
            MerkleProof::NonMembership(non_exist, exist) => {
                println!("{}", hex::encode(&ethabi_bz));

                let [non_exist_tokens, exist_tokens] = ethabi::decode(
                    &[
                        NonExistenceProofEthAbi::param_type(),
                        ExistenceProofEthAbi::param_type(),
                    ],
                    &ethabi_bz,
                )
                .unwrap()
                .try_into()
                .unwrap();

                let non_exist_ethabi =
                    NonExistenceProofEthAbi::from_token(non_exist_tokens).unwrap();
                let exist_ethabi = ExistenceProofEthAbi::from_token(exist_tokens).unwrap();

                assert_eq!(non_exist, non_exist_ethabi.into());
                assert_eq!(exist, exist_ethabi.into());
            }
        }
    }
}

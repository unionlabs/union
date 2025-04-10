use unionlabs::{
    cosmos::ics23::{commitment_proof::CommitmentProof, proof_spec::ProofSpec},
    ibc::core::commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
    primitives::Bytes,
};

pub use crate::proof_specs::{IAVL_PROOF_SPEC, TENDERMINT_PROOF_SPEC};
use crate::{
    existence_proof,
    verify::{self},
};

pub const SDK_SPECS: [ProofSpec; 2] = [IAVL_PROOF_SPEC, TENDERMINT_PROOF_SPEC];

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VerifyMembershipError {
    #[error("root calculation ({0})")]
    RootCalculation(existence_proof::CalculateRootError),
    #[error("{0}")]
    InnerVerification(verify::VerifyMembershipError),
    #[error("calculated root ({calculated}) does not match the given ({given}) value", calculated = serde_utils::to_hex(calculated), given = serde_utils::to_hex(found))]
    InvalidRoot { found: Bytes, calculated: Bytes },
    #[error("expected the size of proofs to be ({expected}), found ({found})")]
    InvalidProofsLength { expected: usize, found: usize },
    #[error("expected the size of key path to be ({expected}), found ({found})")]
    InvalidKeyPathLength { expected: usize, found: usize },
    #[error("proof type is expected to be `Exist`")]
    InvalidProofType,
    #[error("could not retrieve the key due to invalid indexing")]
    InvalidIndexing,
    #[error("nonexistence proof has empty left and right proof")]
    EmptyNonExistenceProof,
    #[error("given proof is empty")]
    EmptyProof,
}

pub fn verify_membership(
    proof: &MerkleProof,
    specs: &[ProofSpec],
    consensus_root: &MerkleRoot,
    path: &[Vec<u8>],
    value: Vec<u8>,
) -> Result<(), VerifyMembershipError> {
    if proof.proofs.len() != specs.len() {
        return Err(VerifyMembershipError::InvalidProofsLength {
            expected: specs.len(),
            found: proof.proofs.len(),
        });
    }

    if path.len() != specs.len() {
        return Err(VerifyMembershipError::InvalidKeyPathLength {
            expected: specs.len(),
            found: path.len(),
        });
    }

    verify_chained_membership_proof(
        consensus_root.hash.as_ref(),
        specs,
        &proof.proofs,
        path,
        value,
        0,
    )
}

pub fn verify_non_membership(
    proof: &MerkleProof,
    specs: &[ProofSpec],
    consensus_root: &MerkleRoot,
    key_path: &[Vec<u8>],
) -> Result<(), VerifyMembershipError> {
    // this will also assert `specs` and `key_path` is not empty, since they are all asserted
    // to be the same length
    if proof.proofs.is_empty() {
        return Err(VerifyMembershipError::EmptyProof);
    }

    if proof.proofs.len() != specs.len() {
        return Err(VerifyMembershipError::InvalidProofsLength {
            expected: specs.len(),
            found: proof.proofs.len(),
        });
    }

    if key_path.len() != specs.len() {
        return Err(VerifyMembershipError::InvalidKeyPathLength {
            expected: specs.len(),
            found: key_path.len(),
        });
    }

    let CommitmentProof::Nonexist(nonexist) = &proof.proofs[0] else {
        return Err(VerifyMembershipError::InvalidProofType);
    };

    // Even both are `Some`, still calculate the left branch
    let subroot = match (&nonexist.left, &nonexist.right) {
        (Some(ep), _) | (None, Some(ep)) => {
            existence_proof::calculate_root(ep).map_err(VerifyMembershipError::RootCalculation)?
        }
        _ => return Err(VerifyMembershipError::EmptyNonExistenceProof),
    };

    let key = key_path.last().expect("len is >= 1");
    verify::verify_non_membership(&specs[0], &subroot, nonexist, key)
        .map_err(VerifyMembershipError::InnerVerification)?;

    verify_chained_membership_proof(
        consensus_root.hash.as_ref(),
        specs,
        &proof.proofs,
        key_path,
        subroot,
        1,
    )
}

fn verify_chained_membership_proof(
    root: &[u8],
    specs: &[ProofSpec],
    proofs: &[CommitmentProof],
    keys: &[Vec<u8>],
    value: Vec<u8>,
    index: usize,
) -> Result<(), VerifyMembershipError> {
    proofs
        .iter()
        .enumerate()
        .skip(index)
        .try_fold(value, |value, (i, proof)| {
            let CommitmentProof::Exist(ref existence_proof) = proof else {
                return Err(VerifyMembershipError::InvalidProofType);
            };

            let subroot = existence_proof::calculate_root(existence_proof)
                .map_err(VerifyMembershipError::RootCalculation)?;

            let key = keys
                .len()
                .checked_sub(1 + i)
                .and_then(|i| keys.get(i))
                .ok_or(VerifyMembershipError::InvalidIndexing)?;

            verify::verify_membership(&specs[i], &subroot, existence_proof, key, &value)
                .map_err(VerifyMembershipError::InnerVerification)?;

            Ok(subroot)
        })
        .and_then(|value| {
            if value.as_slice() == root {
                Ok(())
            } else {
                Err(VerifyMembershipError::InvalidRoot {
                    found: value.into(),
                    calculated: root.to_vec().into(),
                })
            }
        })
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        encoding::{Bincode, DecodeAs, Proto},
        ethereum::ibc_commitment_key,
        ibc::core::commitment::{
            merkle_path::MerklePath, merkle_proof::MerkleProof, merkle_root::MerkleRoot,
        },
        primitives::{encoding::HexUnprefixed, H256},
    };

    use super::{verify_membership, verify_non_membership, VerifyMembershipError, SDK_SPECS};
    use crate::verify;

    fn chained_membership(
        proof: &[u8],
        root: &[u8],
        value: &[u8],
        path: &[&str],
    ) -> Result<(), VerifyMembershipError> {
        let path = MerklePath {
            key_path: path.iter().map(ToString::to_string).collect(),
        };
        let proofs = MerkleProof::decode_as::<Proto>(proof).unwrap();
        verify_membership(
            &proofs,
            &SDK_SPECS,
            &MerkleRoot {
                hash: H256::try_from(root).unwrap(),
            },
            &path
                .key_path
                .into_iter()
                .map(|x| x.into_bytes())
                .collect::<Vec<_>>(),
            value.into(),
        )
    }

    // union-testnet-2
    // key = 0x01 + take(20, sha256(account))
    // proof, value = nix run .#uniond -- genstateproof 345 "014152090B0C95C948EDC407995560FEED4A9DF81E" "/store/acc/key" --node https://rpc.0xc0dejug.uno:443
    // path = split('/', '/acc/${key}')
    // root = nix run .#uniond -- query block 346 --node https://rpc.0xc0dejug.uno:443 | jq .block.header.app_hash
    #[test]
    fn testnet_exist() {
        let root = hex!("B802C903BEFE08624832AAF853DBEA4EDE1F7D50E88BEDD6317831F45CC74A3D");
        let proof = hex!("0afa020af7020a15014152090b0c95c948edc407995560feed4a9df81e129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c122180120011a0b0801180120012a0300020222290801122502040220170c890f01b9fa9ab803511bbc7be7c25359309f04d021a72e0a9b93b8ff72c020222b08011204040802201a2120a89a7b1aedf861a8c6316009af3d19448bfe8834dfb5546c7e1af7f95c3000b4222b08011204061002201a212029347d33c119e85fc1335f43ad17c4a1986ad44c71837158ceffd36e2f38f986222b080112040a3002201a2120e284b7ed0385d018b1ffcd6f33bf6ac575fb7731704d0ae71be278bd8bf5e0b50a80020afd010a03616363122082d7d632a58654a81bb6764379eff4b6e641e96620a12dac0e250e6caf94f7761a090801180120012a010022250801122101ba30cf8122e71a87fea08d0da9499e0373495a64e1648de8f08ca1a73e1fc1a8222708011201011a208a19e0585632ebada293099d24f28707d453266ae7ded6e854dfd8a025c7ce71222708011201011a204a22410f42f7706402b38c460e74d712c95cea8e6e370c691f43c0abf3f4e104222708011201011a20b999d9a62cbd36a843f207580c4802d194e6441f7f3715ddce55d5194d46e57a222708011201011a2022ecbf124eff995ecf01998dd8346b71810af164e192feeb4d4287085128b9df");
        let value = hex!("0a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c12218012001");
        chained_membership(
            &proof,
            &root,
            &value,
            &["acc", unsafe {
                &String::from_utf8_unchecked(
                    hex!("014152090B0C95C948EDC407995560FEED4A9DF81E").to_vec(),
                )
            }],
        )
        .unwrap();
        // let path: Vec<Vec<u8>> = vec![
        //     b"acc".into(),
        //     hex!("014152090B0C95C948EDC407995560FEED4A9DF81E").into(),
        // ];

        // MerklePath::try_from_proto_bytes()
    }

    #[test]
    fn connection_exists() {
        let root = hex!("899CD0B55A4FEDE9AF3C959C43ED3AE6805293642590A81CD95B4C97F89CC424");
        let proof = hex!("0abc020ab9020a18636f6e6e656374696f6e732f636f6e6e656374696f6e2d31125b0a0930382d7761736d2d3112230a0131120d4f524445525f4f524445524544120f4f524445525f554e4f524445524544180222250a0e636f6d6574626c732d6e65772d30120c636f6e6e656374696f6e2d301a050a0369626328061a0c0801180120012a040002f006222c080112050204f006201a212075c4910f51207d3c65960120fe931f138e2624668d75869f51b8442593dd5eab222a080112260408de0a2002b6fcf07091245d162f1196b003c555c564980e02c4d4a9fa0a249798f4b25e20222c08011205060ede0a201a2120ff6b0a04e076eecbabfee4e751c0523cbedba898211b5847404e2d954a2203e3222a08011226081ede0a20635053419cfb6a81c839860d99f3ed002840124a790ddd9f066d8bce63f9df54200afc010af9010a03696263122024b15e198bcf648dee62c7ca1fd8c3950c85c3d898833180c3e3c412ccbc559d1a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc222508011221016376cbd7b917c7105ddac35bdeddd79e6c9cbbc66dd227941599de2b9bc8b3de222708011201011a200d68ac7c3e8daf94c65ccdfe5b7397f50e80325240ef9b2a0ec483afaea30544");
        let value = hex!("0a0930382d7761736d2d3112230a0131120d4f524445525f4f524445524544120f4f524445525f554e4f524445524544180222250a0e636f6d6574626c732d6e65772d30120c636f6e6e656374696f6e2d301a050a036962632806");

        assert_eq!(
            chained_membership(&proof, &root, &value, &["ibc", "connections/connection-1"]),
            Ok(())
        );
    }

    #[test]
    fn client_state_exists() {
        let root = hex!("971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC2F");
        let proof = hex!("0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7");
        let value = hex!("0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e701");

        assert_eq!(
            chained_membership(
                &proof,
                &root,
                &value,
                &["ibc", "clients/08-wasm-1/clientState"]
            ),
            Ok(())
        );
    }

    #[test]
    fn existence_proof_root_mismatch() {
        let root = hex!("971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC22");
        let proof = hex!("0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7");
        let value = hex!("0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e701");

        assert!(matches!(
            chained_membership(
                &proof,
                &root,
                &value,
                &["ibc", "clients/08-wasm-1/clientState"]
            ),
            Err(VerifyMembershipError::InvalidRoot { .. })
        ));
    }

    #[test]
    fn existence_proof_key_mismatch() {
        let root = hex!("971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC2F");
        let proof = hex!("0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7");
        let value = hex!("0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e701");

        assert!(matches!(
            chained_membership(
                &proof,
                &root,
                &value,
                &["ibc", "clients/08-wasm-2/clientState"]
            ),
            Err(VerifyMembershipError::InnerVerification(
                verify::VerifyMembershipError::ExistenceProofVerify(
                    verify::VerifyError::KeyAndExistenceProofKeyMismatch { .. }
                )
            ))
        ));
    }

    #[test]
    fn existence_proof_value_mismatch() {
        let root = hex!("971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC2F");
        let proof = hex!("0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7");
        let value = hex!("0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e700");

        assert!(matches!(
            chained_membership(
                &proof,
                &root,
                &value,
                &["ibc", "clients/08-wasm-1/clientState"]
            ),
            Err(VerifyMembershipError::InnerVerification(
                verify::VerifyMembershipError::ExistenceProofVerify(
                    verify::VerifyError::ValueAndExistenceProofValueMismatch { .. }
                )
            ))
        ));
    }

    #[test]
    fn account_non_existence() {
        let root = H256::new(hex!(
            "79f694c8c821fdb4a6368635125c4f8754791d0cb18a3234d16846b8bd1b8af1"
        ));

        let proofs: cometbft_types::crypto::proof_ops::ProofOps = serde_json::from_str(r#"{"ops":[{"type":"ics23:iavl","key":"q80=","data":"ErkBCgKrzRKyAQoWbmV4dENvbm5lY3Rpb25TZXF1ZW5jZRIIAAAAAAAAAAAaCwgBGAEgASoDAAICIikIARIlAgQCIBbd8oOqTAzz2N37/K2H8sEXU4MpoF2tObMSWeovzgwrICIpCAESJQQIAiC/e92Vz+ipIjtfwS/AQClqyJTFZnFm9xT57lLBlJ2FOCAiKwgBEicGEKCcASB8yrzWR1APjhwhj/IN9xudq2d3bMqTdFQ3ibwBhyuVByA="},{"type":"ics23:simple","key":"aWJj","data":"CvkBCgNpYmMSIC+Djnty/u8Aq4yU7e+160oGfn5YDINR1/yvhhhIaHlfGgkIARgBIAEqAQAiJwgBEgEBGiDuCyeNLY+9uuuNJlej7iVn6FAZd3L1N6bBfGo4fe0rZCIlCAESIQFZckG8cRgqZGow7krfhYaPj1WV+NHi06jP3AzsgCQMBCIlCAESIQG6kYZ6dYB+K9horB1f2/t2QbzklLUPZX85JPwGiciHpyIlCAESIQF+bA9XuNY3FzraIK1ZiLenkCsDKJfYE1jU4jZmbVdDNSInCAESAQEaIGSuJu8jPsinlMyTZkCjypibd+BjdVz7PZEVK9p85uCF"}]}"#).unwrap();

        let proofs = proofs
            .ops
            .into_iter()
            .map(|op| {
                <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(&*op.data)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let proof =
            MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof { proofs })
                .unwrap();

        let root = MerkleRoot { hash: root };

        assert_eq!(
            verify_non_membership(
                &proof,
                &SDK_SPECS,
                &root,
                &[b"ibc".to_vec(), hex!("abcd").to_vec()]
            ),
            Ok(())
        );
    }

    // #[test]
    // fn try_proof() {
    //     let proof= serde_json::from_str::<MerkleProof>(r#"{"proofs":[{"@type":"exist","@value":{"key":"0x636f6e6e656374696f6e732f636f6e6e656374696f6e2d32","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002c005","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204c00520f3c95230594f910cfcda0f08040272de9ff25ebdc362fa2984fe04372b5c56fb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x0408c00520d4c306fc76462b92c3888a22f78962e742022a4fa4b272947acf199993820a0720","suffix":"0x0"},{"hash":"sha256","prefix":"0x0610c00520","suffix":"0x2002a35cfb0a1fe5aec25e1d915887411f67741f9c511b3befa22eb95bc709792d"},{"hash":"sha256","prefix":"0x0a22c0052008808333ca466e463fd6174d936b7e9ae0153af98f42fd848b7c7cb53580de2b20","suffix":"0x0"}],"value":"0x0a0930382d7761736d2d3112140a0131120f4f524445525f554e4f524445524544180122130a0a636f6d6574626c732d301a050a03696263"}},{"@type":"exist","@value":{"key":"0x696263","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x01","suffix":"0x2cd8b50700950546180ad979135a8708c2ea2098fff6ade31b7e40eb5dcf7c05"},{"hash":"sha256","prefix":"0x012cf3feea58fcdb48b73c2cdd1b018c90c4078f924385675a0e9457168cd47ff1","suffix":"0x0"},{"hash":"sha256","prefix":"0x01373d3c4151d1fbe9641325af4682f5c936b22ddd6f27693369920ec5db527eb5","suffix":"0x0"},{"hash":"sha256","prefix":"0x01b0b8ee671be7e2122d443854cee9939a9a8b323d535db88124e99490b975b87e","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0x40c17eb6aef68c760f6d7eb72e33e09610e5523f7eb6e9416a2e0ea0cc9fc171"}],"value":"0xfa6e73dfa59c3977f62f4f14ec1a1c1b4b8c90e644cb274cb938b484f3310e7d"}}]}"#).unwrap();

    //     verify_membership(
    //         &proof,
    //         &SDK_SPECS,
    //         &MerkleRoot { hash: hex!("F7ED1D182E325EA21817DC6048C7043056F76AB044642504CE57DDC5C1B47CD3").into() },
    //         &[b"ibc".to_vec(), ConnectionPath { connection_id: ConnectionId::new(2) }.to_string().into_bytes()],
    //         hex!("0a0930382d7761736d2d3112140a0131120f4f524445525f554e4f524445524544180122130a0a636f6d6574626c732d301a050a03696263").into()
    //     ).unwrap();
    // }

    #[test]
    fn ethermint_proof() {
        // let proof = serde_json::from_str::<MerkleProof>(r#"{"proofs":[{"@type":"exist","@value":{"key":"0x0270719a4ca2d463806863b6a86281c93b8854ddc224b2c12774d8f0bc59368ca29df35712bb7604aedaebd945e71357644d6fd8d0","value":"0x9e7dde17244338c4e83fed8f65c6665f89fcfad812dee37df220f29e8e557f56","leaf":{"hash":"sha256","prehash_key":"no_hash","prehash_value":"sha256","length":"var_proto","prefix":"0x0002b4cbf102"},"path":[{"hash":"sha256","prefix":"0x0204b8c8f40220","suffix":"0x20dc4ac709f529da79e78508b78b3bec8d7e5a742b7325d78e4e1625ad4d289f39"},{"hash":"sha256","prefix":"0x0406b8c8f40220809527dab567250ed5d3da0808897de1ee488ba76122daf357cc29ea36f942be20","suffix":"0x0"},{"hash":"sha256","prefix":"0x060ab8c8f40220","suffix":"0x20e91cd6e7580715ecb89aa60a246bbfc5b9a0f29d73c34f058b07423e99cc3c19"},{"hash":"sha256","prefix":"0x0810b8c8f40220aed12e57823a9d1d37daeba946be3d53fa551fd1618a82ed2a00ebac3260bcda20","suffix":"0x0"},{"hash":"sha256","prefix":"0x0a20b8c8f40220","suffix":"0x20b826535cca1ca2a60a2fdc6d4a03a70fd5aacc50ff2f7cacc5ed335a7122377c"},{"hash":"sha256","prefix":"0x0c48b8c8f40220","suffix":"0x2040c6ebdf4b7ec32b88087b1ef96a9c2e96caa42684babb6a4dbde8f181ee33a7"},{"hash":"sha256","prefix":"0x0e8601b8c8f4022027b61130bfaf00cbda92eef76c966e85584b1e80a835d56819dc861085c2af0e20","suffix":"0x0"},{"hash":"sha256","prefix":"0x10d601b8c8f40220","suffix":"0x201b25cfa9d93abb008cb30b550107b12dfca536b3c065a7223dc1f942d9e55147"},{"hash":"sha256","prefix":"0x12c202b8c8f402201ada6c425cdb28eb3c51bc0bcaf8beb5f3df9c297d3ab472df19cc76e6a8050b20","suffix":"0x0"},{"hash":"sha256","prefix":"0x148a05b8c8f40220cb0ca74344d967333238a4ccdaf7035f79d8f9935d4f4397a151cc3dee2f64c020","suffix":"0x0"},{"hash":"sha256","prefix":"0x168a0bb8c8f402200cadc5956a147b1ade4677ce204f2eb83f8cd42b7f7e30e175f21bdea34fdd1420","suffix":"0x0"},{"hash":"sha256","prefix":"0x1a9418b8c8f402204691091cc5276b060a60e5ffb9b61c940978c585661a244fa456d3576f30b97c20","suffix":"0x0"},{"hash":"sha256","prefix":"0x1cbe3cb8c8f402204607d3b2bf0178fbedab3ae30a3ce0a8f147e232829ce8a18ae0b5fcf8e5455920","suffix":"0x0"},{"hash":"sha256","prefix":"0x1e9470b8c8f40220","suffix":"0x205199b20006fb2319c2d5769aa2cc4c213a09539d858eef522da43e0c493861a7"},{"hash":"sha256","prefix":"0x20baa201b8c8f40220e0126ea69c7de16a6c32a07c5cd01b824e0074e60132af4ec1ba51f525f443f520","suffix":"0x0"},{"hash":"sha256","prefix":"0x24c0e702fecff40220349914a78da6b9fd658834cef79f04297c2a6b35eafa90c151704c5125da544420","suffix":"0x0"},{"hash":"sha256","prefix":"0x26de8f05cad2f402204c6774b7e86de4abb7e30c1315f1a235c7a6aa17b8f72a200158496532f7d62420","suffix":"0x0"},{"hash":"sha256","prefix":"0x28cce709f2d2f40220933b9b096825854895749fed32066612c9bd62772e73b84c83cbc7b8032e7de120","suffix":"0x0"},{"hash":"sha256","prefix":"0x2af2ab11f2d2f40220dad159982e9d0efc24da40898170232d7967486dc4e1913cbbf3f885d0dd28a520","suffix":"0x0"},{"hash":"sha256","prefix":"0x2c80ed2bf8d3f40220f07c36ead5de4939dfe80974b059a61665676f26ea7cff863053e295300654a920","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ea6da52f8d3f402201af0e150b43d5bb329af9cfe4bfe6b287e267521dc71c06ece34882964c468c720","suffix":"0x0"},{"hash":"sha256","prefix":"0x30d4f2a901c0d4f40220edf14f868da07984503e4ddb9ff8eb24dab256bf050a1bb125e002ae5db9bbf020","suffix":"0x0"},{"hash":"sha256","prefix":"0x34beacd302c0d4f40220e74db5b4b2d2eb541960d650cc7025e242a47129e27fee348b023f4c821de50c20","suffix":"0x0"},{"hash":"sha256","prefix":"0x36c6de8204c0d4f40220","suffix":"0x209c0da09285895cd1f569aa5fc3532285b0039154e38ef868243c74cdd33cb2c3"},{"hash":"sha256","prefix":"0x38a2e7ce06c0d4f40220","suffix":"0x2015fc849f15c2ed78a091287e17092e994d71dec2977a7e6b0fde069e11340daa"}]}},{"@type":"exist","@value":{"key":"0x65766d","value":"0x2330e8ddde6a2aaaccc4717685fa620c6c9172533c330c1b9dad0d4ded3866c2","leaf":{"hash":"sha256","prehash_key":"no_hash","prehash_value":"sha256","length":"var_proto","prefix":"0x00"},"path":[{"hash":"sha256","prefix":"0x015b933aed2e085b4eaed2f042b39ea402e3998cf2901a1e75755b767de1cb8faf","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0xe1749f3c3c9afeebd5ea7e684ac0f83ec955142cbac282295bdf22cc9b0483db"},{"hash":"sha256","prefix":"0x01855afe7941d4071ed7d1e5b6c4531b02ae4273de0ba03b409258fd993bba20ba","suffix":"0x0"},{"hash":"sha256","prefix":"0x017b12eb72cf3db4723d82ea7a07ccd75dc0f68a7c765263e512182e845df2a338","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0x05e1e0dd912e9111849118b313e761de8b38815f33760abe0d772de7a85e46f2"}]}}]}"#).unwrap();

        let proof = MerkleProof::decode_as::<Bincode>(&hex!("02000000000000000000000035000000000000000270719a4ca2d463806863b6a86281c93b8854ddc24e5d7ba1faafe63015a843357624e9aa59aa28e0fe6cabc4b3d73bc9dbae303720000000000000001a06b8264038ba3c02560b22e19e55d102be9dd0c893c4219f9b9114607dabae0100000000000000010000000100000006000000000000000002b8c8f40218000000000000000100000028000000000000000204b8c8f40220ff2a62aee5264080675c397f9fee63390927a738a59b85aaf8d93df99fccecfd2000000000000000000100000007000000000000000408b8c8f4022021000000000000002028c6aa56e988c1d178dd4397b7fddfeda04979706b0136849b39a391bf7b4fc40100000007000000000000000610b8c8f402202100000000000000201d81a42f85a560fdd58f3bf6f31e26962c1d6f83e2ea52ecf59047f38aff16270100000028000000000000000a20b8c8f402203f14997a4e70fdfdab184958d07995e9bc73031d8727d9cb02df6ae103145dbe2000000000000000000100000007000000000000000c48b8c8f4022021000000000000002040c6ebdf4b7ec32b88087b1ef96a9c2e96caa42684babb6a4dbde8f181ee33a70100000029000000000000000e8601b8c8f4022027b61130bfaf00cbda92eef76c966e85584b1e80a835d56819dc861085c2af0e20000000000000000001000000080000000000000010d601b8c8f402202100000000000000201b25cfa9d93abb008cb30b550107b12dfca536b3c065a7223dc1f942d9e5514701000000290000000000000012c202b8c8f402201ada6c425cdb28eb3c51bc0bcaf8beb5f3df9c297d3ab472df19cc76e6a8050b200000000000000000010000002900000000000000148a05b8c8f40220cb0ca74344d967333238a4ccdaf7035f79d8f9935d4f4397a151cc3dee2f64c0200000000000000000010000002900000000000000168a0bb8c8f402200cadc5956a147b1ade4677ce204f2eb83f8cd42b7f7e30e175f21bdea34fdd142000000000000000000100000029000000000000001a9418b8c8f402204691091cc5276b060a60e5ffb9b61c940978c585661a244fa456d3576f30b97c2000000000000000000100000029000000000000001cbe3cb8c8f402204607d3b2bf0178fbedab3ae30a3ce0a8f147e232829ce8a18ae0b5fcf8e545592000000000000000000100000008000000000000001e9470b8c8f402202100000000000000205199b20006fb2319c2d5769aa2cc4c213a09539d858eef522da43e0c493861a7010000002a0000000000000020baa201b8c8f40220e0126ea69c7de16a6c32a07c5cd01b824e0074e60132af4ec1ba51f525f443f5200000000000000000010000002a0000000000000024aee702b8c8f402209d3f3a4cb9427c2eb1af7573f81259d3fe4cb36433f7ea94b0252346e1de14ff200000000000000000010000002a0000000000000026b48f05d0c8f402206c66244b291560e9b9ab611a53cacc02c8fb9c11c6da7991f4907a3bc7c2e6c7200000000000000000010000002a0000000000000028eee609d0c8f4022046daf856b1640859635ed8f1d2a136b0fd58e152b72fc117b3aba39c47d6c828200000000000000000010000002a000000000000002ac8aa11dec8f40220f18e4420874809b9975cc478c616992e57efcdad3234573c1c0d172e2508954f200000000000000000010000002a000000000000002cf4e92becc8f40220de398493ea7a4162928477ae7dc6688e6baa611430ba76e78ab59ce0220c1892200000000000000000010000002a000000000000002eded352ecc8f4022054dfb0f73add06a3a44b9e6d90eb0d3bed8732a9e38118c6e774dd86664ab8d8200000000000000000010000002b0000000000000030a8e9a901ecc8f402204715b794125abb515c86d2a61a28b2c6ddd496f44f9c014986a29c1bd90c03e0200000000000000000010000002b0000000000000034d49ad302ecc8f402206d9ea57a1ee2285ed32f1eb0d37ce68e336958f4a3df27527be6dd06a8b37eee200000000000000000010000000a0000000000000036c4c68204ecc8f402202100000000000000204a26d7adc39fc56f3121c31a92d1f8c6ce79ec38bcdf13d6f7cbc6992ce37646010000000a000000000000003888bece06ecc8f40220210000000000000020ce6535d3f0b0e50767320f1c95cfa29bfbef174db67d7ca75db17a964abd4ab700000000030000000000000065766d2000000000000000b48a31fd5a4726074cd588a39a7077355b149faf6a5e2171338c49a8e735914f010000000000000001000000010000000100000000000000000500000000000000010000002100000000000000015b933aed2e085b4eaed2f042b39ea402e3998cf2901a1e75755b767de1cb8faf00000000000000000100000001000000000000000120000000000000001b57b7eb6d2772ec3bb924e0e28bc74e65874764ab342eb16661026b2e991635010000002100000000000000011e90948ac20a5444219f52c6441c94cd6c10ed07253a4cc93e0ece2597bf6923000000000000000001000000210000000000000001cdbaef1813b50311051041b2a308e06fdba0fbd98a19d6c6b3aacb46453881980000000000000000010000000100000000000000012000000000000000d28f5d69f7aa9cad02d9ec1a84419d3ba7d499c28e0bc7be43ac5f1a1da5d0ce")).unwrap();

        dbg!(&proof);

        let root = MerkleRoot {
            hash: "zgU0ki8JI472s//JSi+3rOYRSRrFhLNdN11bwuCmVso="
                .parse()
                .unwrap(),
            // hash: "11DCC71B4996BE60DA890E11B946CDD4B2E599830C831D607A9E222EBCF563D3"
            //     .parse::<FixedBytes<32, HexUnprefixed>>()
            //     .unwrap()
            //     .into_encoding(),
        };

        dbg!(&root.hash.as_encoding::<HexUnprefixed>());

        verify_membership(
            &proof,
            &SDK_SPECS,
            &root,
            &[
                b"evm".to_vec(),
                [0x02]
                    .into_iter()
                    .chain(hex!("70719a4ca2d463806863b6a86281c93b8854ddc2"))
                    .chain(
                        ibc_commitment_key(
                            hex!(
                                "c3a24b0501bd2c13a7e57f2db4369ec4c223447539fc0724a9d55ac4a06ebd4d"
                            )
                            .into(),
                        )
                        .to_be_bytes(),
                    )
                    .collect(),
            ],
            hex!("1a06b8264038ba3c02560b22e19e55d102be9dd0c893c4219f9b9114607dabae").to_vec(),
        )
        .unwrap();
    }

    #[test]
    fn packet_timeout_non_membership() {
        let proof = serde_json::from_str::<MerkleProof>(r#"{"proofs":[{"@type":"nonexist","@value":{"key":"0x0316d5260c4a4bc907b79822b792cf51c57f05f311f1e79309b7cc3ece4e277af300c8e8bda249383359718501c9fc5e6db0215edfd2ba8a8812b08665c226e57844","left":{"key":"0x0316d5260c4a4bc907b79822b792cf51c57f05f311f1e79309b7cc3ece4e277af300c8e6b50d683ea8bc68beb2a555a59bac272a6ebd8470d612d11b3eb26a4a5d","value":"0x01502841361a25f62632a8138da2f5da1948539832b0fc2bfb733cc7b081836e","leaf":{"hash":"sha256","prehash_key":"no_hash","prehash_value":"sha256","length":"var_proto","prefix":"0x0002b8a332"},"path":[{"hash":"sha256","prefix":"0x0204dc904220","suffix":"0x20aa4be3b4460f469a54a997e9fc0e4c5a15360c5ae1db92b49cacb4b22214ecb7"},{"hash":"sha256","prefix":"0x04088ec24220","suffix":"0x2014292d77fd88baf068c554db08686c7556f8d789473ee01e91c161718fde1488"},{"hash":"sha256","prefix":"0x060e8ec24220","suffix":"0x20192764ea5e79cb32fb3f11d53665d29a0d777f825a6491961880ba1e54340dcc"},{"hash":"sha256","prefix":"0x0a2686824820396ef3cfb88b312d20207809d07000efd88f2f7dc5229feb6cfda4d68265ceda20","suffix":"0x0"},{"hash":"sha256","prefix":"0x0e5eb2bc4d202060c7dd2d09459ea291a7e3f62d2c3360fcd3ea27086c028517b7e82e2e202920","suffix":"0x0"},{"hash":"sha256","prefix":"0x10a401d88b5120","suffix":"0x200299e87712cc651d04fad22c28362ad369aeaace382f114129927cebc5814b3f"},{"hash":"sha256","prefix":"0x12ee02d8f35420f2b92607b1b503d7b99066321684b44916ec3d8a259e9e5e455998c2a731af9520","suffix":"0x0"},{"hash":"sha256","prefix":"0x14a605dcc25520","suffix":"0x200adfc7b130f05f71c93a3c9279bfc0a5f02a7c4cc0cf8a0259da481133333ec0"},{"hash":"sha256","prefix":"0x169008dcc25520","suffix":"0x2069b0cf34242930daf1c8b4fac14678e5e15fc3887c9747a225da780b6e275290"},{"hash":"sha256","prefix":"0x18c610dcc25520","suffix":"0x20bd446e2cb44c82425f1f283d9ba42301e348822b2d5f91666afe3b94a4f9c662"},{"hash":"sha256","prefix":"0x1ad41da2e85520","suffix":"0x209e357f1f4b212b7f0cd13736eec2a20a094ff17e19f57065076c60325dc5d7f4"},{"hash":"sha256","prefix":"0x1ca836a2e855202011f44d3da40ce2d9b4d69ff3df4ba7b0f60bb4d74b45cdcfffe52a1d7e595920","suffix":"0x0"},{"hash":"sha256","prefix":"0x1eda5aa2e85520ebda986e64b5c8babb379a66e2df0ed13bf5ef0969912eaffbfc8922882774f420","suffix":"0x0"},{"hash":"sha256","prefix":"0x20fe9701a2e855201691ba603c7516af8f6e5e3870b2e9b5d93c15740f6a8b5e067d75ee01fe62c820","suffix":"0x0"},{"hash":"sha256","prefix":"0x228cdf02a4eb5520","suffix":"0x20406d8a6e864102c22e775f263a958ec616400989f622611c39a7acc6183d9b56"},{"hash":"sha256","prefix":"0x24aede05a4eb5520832741e67cb7525c0c6e44f08a2ccab31c4f6e471722e482ef339d3d3c3f993a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x26b4920dd4eb5520f558704bc9c29285f8f12c297be86ab3deca125bb8a20217d82930625210b8cb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ab2ab1bd4eb5520","suffix":"0x20337ac646c149de08f877f613c9d7798a96826cbf6977ed64f6ed21c36170ba8d"},{"hash":"sha256","prefix":"0x2ea4f736d4eb5520","suffix":"0x20717db97555e834d39eebe08926de9db1274bd0e4e4646e5fdbd3afd1649e0e62"},{"hash":"sha256","prefix":"0x30849ea701d4eb5520","suffix":"0x20f6d89b3895fbfb8b8d544bf6b55eeaafb38105d3300288f154e638e9de2b676d"},{"hash":"sha256","prefix":"0x34b8aeef02d4eb5520","suffix":"0x2016a4876d8857304b2b073839832c04e964558a6d4dbf276d5153ec910ad55069"},{"hash":"sha256","prefix":"0x36fec9e106d4eb5520","suffix":"0x20c65ebf738e8d4eafce3f004311c65b5eeda4af6fa1246c12b05255b888103801"}]},"right":{"key":"0x0316d5260c4a4bc907b79822b792cf51c57f05f311f1e79309b7cc3ece4e277af300c8ed3c81d8f763521509ba9b4d310c1f46dce8efe732a512cdd14de4ca7d6a","value":"0x01502841361a25f62632a8138da2f5da1948539832b0fc2bfb733cc7b081836e","leaf":{"hash":"sha256","prehash_key":"no_hash","prehash_value":"sha256","length":"var_proto","prefix":"0x0002eaba41"},"path":[{"hash":"sha256","prefix":"0x0204dc90422019e61e56713d298667526e3160efc7939ebec8acdba0a73a0a9a5d7d6e40c25820","suffix":"0x0"},{"hash":"sha256","prefix":"0x04088ec24220","suffix":"0x2014292d77fd88baf068c554db08686c7556f8d789473ee01e91c161718fde1488"},{"hash":"sha256","prefix":"0x060e8ec24220","suffix":"0x20192764ea5e79cb32fb3f11d53665d29a0d777f825a6491961880ba1e54340dcc"},{"hash":"sha256","prefix":"0x0a2686824820396ef3cfb88b312d20207809d07000efd88f2f7dc5229feb6cfda4d68265ceda20","suffix":"0x0"},{"hash":"sha256","prefix":"0x0e5eb2bc4d202060c7dd2d09459ea291a7e3f62d2c3360fcd3ea27086c028517b7e82e2e202920","suffix":"0x0"},{"hash":"sha256","prefix":"0x10a401d88b5120","suffix":"0x200299e87712cc651d04fad22c28362ad369aeaace382f114129927cebc5814b3f"},{"hash":"sha256","prefix":"0x12ee02d8f35420f2b92607b1b503d7b99066321684b44916ec3d8a259e9e5e455998c2a731af9520","suffix":"0x0"},{"hash":"sha256","prefix":"0x14a605dcc25520","suffix":"0x200adfc7b130f05f71c93a3c9279bfc0a5f02a7c4cc0cf8a0259da481133333ec0"},{"hash":"sha256","prefix":"0x169008dcc25520","suffix":"0x2069b0cf34242930daf1c8b4fac14678e5e15fc3887c9747a225da780b6e275290"},{"hash":"sha256","prefix":"0x18c610dcc25520","suffix":"0x20bd446e2cb44c82425f1f283d9ba42301e348822b2d5f91666afe3b94a4f9c662"},{"hash":"sha256","prefix":"0x1ad41da2e85520","suffix":"0x209e357f1f4b212b7f0cd13736eec2a20a094ff17e19f57065076c60325dc5d7f4"},{"hash":"sha256","prefix":"0x1ca836a2e855202011f44d3da40ce2d9b4d69ff3df4ba7b0f60bb4d74b45cdcfffe52a1d7e595920","suffix":"0x0"},{"hash":"sha256","prefix":"0x1eda5aa2e85520ebda986e64b5c8babb379a66e2df0ed13bf5ef0969912eaffbfc8922882774f420","suffix":"0x0"},{"hash":"sha256","prefix":"0x20fe9701a2e855201691ba603c7516af8f6e5e3870b2e9b5d93c15740f6a8b5e067d75ee01fe62c820","suffix":"0x0"},{"hash":"sha256","prefix":"0x228cdf02a4eb5520","suffix":"0x20406d8a6e864102c22e775f263a958ec616400989f622611c39a7acc6183d9b56"},{"hash":"sha256","prefix":"0x24aede05a4eb5520832741e67cb7525c0c6e44f08a2ccab31c4f6e471722e482ef339d3d3c3f993a20","suffix":"0x0"},{"hash":"sha256","prefix":"0x26b4920dd4eb5520f558704bc9c29285f8f12c297be86ab3deca125bb8a20217d82930625210b8cb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x2ab2ab1bd4eb5520","suffix":"0x20337ac646c149de08f877f613c9d7798a96826cbf6977ed64f6ed21c36170ba8d"},{"hash":"sha256","prefix":"0x2ea4f736d4eb5520","suffix":"0x20717db97555e834d39eebe08926de9db1274bd0e4e4646e5fdbd3afd1649e0e62"},{"hash":"sha256","prefix":"0x30849ea701d4eb5520","suffix":"0x20f6d89b3895fbfb8b8d544bf6b55eeaafb38105d3300288f154e638e9de2b676d"},{"hash":"sha256","prefix":"0x34b8aeef02d4eb5520","suffix":"0x2016a4876d8857304b2b073839832c04e964558a6d4dbf276d5153ec910ad55069"},{"hash":"sha256","prefix":"0x36fec9e106d4eb5520","suffix":"0x20c65ebf738e8d4eafce3f004311c65b5eeda4af6fa1246c12b05255b888103801"}]}}},{"@type":"exist","@value":{"key":"0x7761736d","value":"0x4db0491378f725764de609a71a4136ab6be503d84ac8d5ffbab71999a3d1b289","leaf":{"hash":"sha256","prehash_key":"no_hash","prehash_value":"sha256","length":"var_proto","prefix":"0x00"},"path":[{"hash":"sha256","prefix":"0x01a88c8b9f6e116cf2bbefa4e6f8f2536baacb38fc74c2badbc0a80477a773366f","suffix":"0x0"},{"hash":"sha256","prefix":"0x01956cff3b80169c81f4fceb114009c474235f13fedca894f904eff4ade91d8e1f","suffix":"0x0"},{"hash":"sha256","prefix":"0x011f031994027f022042795f47eeda31f89e95cf5a2a8bf63ec239f35a05b5bf68","suffix":"0x0"}]}}]}"#).unwrap();

        let root = MerkleRoot {
            hash: "0x21c446360f0235662d8deed989f74eaee1c54827d1eeb9fd76208af2e7461e08"
                .parse::<H256>()
                .unwrap()
                .into_encoding(),
        };

        dbg!(&root.hash.as_encoding::<HexUnprefixed>());

        verify_non_membership(
            &proof,
            &SDK_SPECS,
            &root,
            &[
                b"wasm".to_vec(),
                hex!("0316d5260c4a4bc907b79822b792cf51c57f05f311f1e79309b7cc3ece4e277af300c8e8bda249383359718501c9fc5e6db0215edfd2ba8a8812b08665c226e57844").to_vec()
            ],
        )
        .unwrap();
    }
}

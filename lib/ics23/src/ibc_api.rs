use unionlabs::{
    cosmos::ics23::{commitment_proof::CommitmentProof, proof_spec::ProofSpec},
    ibc::core::commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
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
    InvalidRoot { found: Vec<u8>, calculated: Vec<u8> },
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
                    found: value,
                    calculated: root.to_vec(),
                })
            }
        })
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        encoding::{DecodeAs, Proto},
        hash::H256,
        ibc::core::commitment::{
            merkle_path::MerklePath, merkle_proof::MerkleProof, merkle_root::MerkleRoot,
        },
        ics24::ConnectionPath,
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
        let root = H256(hex!(
            "C9A25B954FEF48EC601359591A28C9A2FD32A411421AEF2DC16DC8A68B3CFA98"
        ));
        let proof = hex!("0a96061293060a15014152090b0c95c948edc407995560feed4a9df88812fa020a15014152090b0c95c948edc407995560feed4a9df81e129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c122180420031a0b0801180120012a0300027822290801122502047820170c890f01b9fa9ab803511bbc7be7c25359309f04d021a72e0a9b93b8ff72c020222c0801120504089601201a21205f282a80f1d186fa1f7b237f81e8bc9a4bb40d5a03cbbdffdd421b1ad4cb16f4222c0801120506109601201a2120e9c65294b7106c7323dcabe4532232c319afc78cd373e338f12df43f8ecfa909222c080112050a309601201a2120a95af7890dba33514ea28a3db7b409f4887b058d6d1e43960c4cd45bb1d9bef81afc020a150143e46d91544517a037a8029b6c7f86f62bab389b129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e3167306a786d79323567357436716461677132646b636c7578376334366b77796d38646563667712460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034611ea6606f6241fdeb0db1854a785eaa2fef5770694237daaf46057cadb3903180320031a0c0801180120012a0400029601222c0801120502049601201a2120532543090d1564b206e953fd6f97000d9b78bd5a8a424f551d483a58b3f54c57222a0801122604089601207e55a1ee8006e9c29c895a8de8ea8cdc6aaddc10e05ea3d3ee8fac786a73c02d20222c0801120506109601201a2120e9c65294b7106c7323dcabe4532232c319afc78cd373e338f12df43f8ecfa909222c080112050a309601201a2120a95af7890dba33514ea28a3db7b409f4887b058d6d1e43960c4cd45bb1d9bef80a80020afd010a0361636312205281c416bf4f80b9d99428a09f91ff311968a3b2adb199342d63c9db20a417e91a090801180120012a010022250801122101ba30cf8122e71a87fea08d0da9499e0373495a64e1648de8f08ca1a73e1fc1a8222708011201011a203489cd05a389a1d165f19003cea0994df9e55a5cb53b3d659417040be528b86d222708011201011a20e5c60ddccacb1c6b0be7957e8d7a86dc0f8bcec91c91d666d39eb1ebedd1bdf1222708011201011a2047a4c9a64496594e8b255443aa979293b2c7120150cf31e0eeeb8a2a987fd7e8222708011201011a2053bca15bed4becbdfd1b4cd0e63bd3845646022a99a2289a6678d8608f092207");

        let proof = MerkleProof::decode_as::<Proto>(&proof).unwrap();
        let root = MerkleRoot { hash: root };

        assert_eq!(
            verify_non_membership(
                &proof,
                &SDK_SPECS,
                &root,
                &[b"acc".to_vec(), b"muh".to_vec()]
            ),
            Ok(())
        );
    }

    #[test]
    fn try_proof() {
        let proof= serde_json::from_str::<MerkleProof>(r#"{"proofs":[{"@type":"exist","@value":{"key":"0x636f6e6e656374696f6e732f636f6e6e656374696f6e2d32","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x0002c005","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x0204c00520f3c95230594f910cfcda0f08040272de9ff25ebdc362fa2984fe04372b5c56fb20","suffix":"0x0"},{"hash":"sha256","prefix":"0x0408c00520d4c306fc76462b92c3888a22f78962e742022a4fa4b272947acf199993820a0720","suffix":"0x0"},{"hash":"sha256","prefix":"0x0610c00520","suffix":"0x2002a35cfb0a1fe5aec25e1d915887411f67741f9c511b3befa22eb95bc709792d"},{"hash":"sha256","prefix":"0x0a22c0052008808333ca466e463fd6174d936b7e9ae0153af98f42fd848b7c7cb53580de2b20","suffix":"0x0"}],"value":"0x0a0930382d7761736d2d3112140a0131120f4f524445525f554e4f524445524544180122130a0a636f6d6574626c732d301a050a03696263"}},{"@type":"exist","@value":{"key":"0x696263","leaf":{"hash":"sha256","length":"var_proto","prefix":"0x00","prehash_key":"no_hash","prehash_value":"sha256"},"path":[{"hash":"sha256","prefix":"0x01","suffix":"0x2cd8b50700950546180ad979135a8708c2ea2098fff6ade31b7e40eb5dcf7c05"},{"hash":"sha256","prefix":"0x012cf3feea58fcdb48b73c2cdd1b018c90c4078f924385675a0e9457168cd47ff1","suffix":"0x0"},{"hash":"sha256","prefix":"0x01373d3c4151d1fbe9641325af4682f5c936b22ddd6f27693369920ec5db527eb5","suffix":"0x0"},{"hash":"sha256","prefix":"0x01b0b8ee671be7e2122d443854cee9939a9a8b323d535db88124e99490b975b87e","suffix":"0x0"},{"hash":"sha256","prefix":"0x01","suffix":"0x40c17eb6aef68c760f6d7eb72e33e09610e5523f7eb6e9416a2e0ea0cc9fc171"}],"value":"0xfa6e73dfa59c3977f62f4f14ec1a1c1b4b8c90e644cb274cb938b484f3310e7d"}}]}"#).unwrap();

        verify_membership(
            &proof,
            &SDK_SPECS,
            &MerkleRoot { hash: hex!("F7ED1D182E325EA21817DC6048C7043056F76AB044642504CE57DDC5C1B47CD3").into() },
            &[b"ibc".to_vec(), ConnectionPath { connection_id: "connection-2".parse().unwrap() }.to_string().into_bytes()],
            hex!("0a0930382d7761736d2d3112140a0131120f4f524445525f554e4f524445524544180122130a0a636f6d6574626c732d301a050a03696263").into()
        ).unwrap();
    }
}

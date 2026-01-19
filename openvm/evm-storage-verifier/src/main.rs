#![cfg_attr(not(test), no_std, no_main)]

use alloc::vec::Vec;

use openvm::io::read;
use serde::{Deserialize, Serialize};
use starknet_storage_verifier::{Membership, PedersenHash};
use starknet_types::{Felt, MerkleNode};
use unionlabs_primitives::{Bytes, H160, H256, U256};

extern crate alloc;

openvm::entry!(main);

#[derive(Debug, Serialize, Deserialize)]
pub enum Proof {
    Evm(EvmProof),
    Starknet(StarknetProof),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvmProof {
    pub state_root: H256,
    pub address: H160,
    pub storage_hash: H256,
    pub account_proof: Vec<Vec<u8>>,
    pub storage_proof: Vec<Vec<u8>>,
    pub key: U256,
    pub value: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StarknetProof {
    pub root: H256,
    pub key: H256,
    pub value: H256,
    pub proof: Vec<SerdeMerkleNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SerdeMerkleNode {
    BinaryNode { left: H256, right: H256 },
    EdgeNode { path: H256, length: u8, child: H256 },
}

fn main() {
    openvm::io::println("deserializing proof");

    let proof = read::<Proof>();

    openvm::io::println("deserialized proof");

    match proof {
        Proof::Evm(proof) => {
            evm_storage_verifier::verify_account_storage_root(
                proof.state_root,
                &proof.address,
                &proof.account_proof,
                &proof.storage_hash,
            )
            .unwrap();

            evm_storage_verifier::verify_storage_proof(
                proof.storage_hash,
                proof.key,
                proof.value,
                &proof.storage_proof,
            )
            .unwrap()
        }
        Proof::Starknet(proof) => {
            let felt = |bz: H256| Felt::from_be_bytes(*bz.get());

            let res = starknet_storage_verifier::verify_proof::<PedersenHash>(
                Felt::from_be_bytes(*proof.root.get()),
                Felt::from_be_bytes(*proof.key.get()),
                Felt::from_be_bytes(*proof.value.get()),
                &proof
                    .proof
                    .into_iter()
                    .map(|node| match node {
                        SerdeMerkleNode::BinaryNode { left, right } => MerkleNode::BinaryNode {
                            left: felt(left),
                            right: felt(right),
                        },
                        SerdeMerkleNode::EdgeNode {
                            path,
                            length,
                            child,
                        } => MerkleNode::EdgeNode {
                            path: felt(path),
                            length,
                            child: felt(child),
                        },
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap();

            assert_eq!(res, Membership::Membership);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use alloy::rpc::types::EIP1186AccountProofResponse;
    use starknet_storage_verifier::{Felt, Membership, MerkleNode, PedersenHash};
    use unionlabs_primitives::{FixedBytes, encoding::HexUnprefixed};

    use super::*;

    fn write_proof(proof: &impl Serialize) {
        let bz = openvm::serde::to_vec(proof).unwrap();

        let mut out = "0x01".to_owned();

        for word in bz {
            write!(
                &mut out,
                "{}",
                FixedBytes::<4, HexUnprefixed>::new(word.to_le_bytes())
            )
            .unwrap();
        }

        std::fs::write("../../input.hex", out).unwrap();
    }

    #[test]
    fn evm() {
        let proof = serde_json::from_slice::<EIP1186AccountProofResponse>(
            &std::fs::read(
                "../../lib/evm-storage-verifier/src/test/valid_storage_proof_sepolia.json",
            )
            .unwrap(),
        )
        .unwrap();

        write_proof(&EvmProof {
            state_root: alloy::hex!(
                "545e7cf676baca0fad067f9884fbb2a42090c0fa63a00c217c60688917deee6e"
            )
            .into(),
            address: proof.address.0.into(),
            storage_hash: proof.storage_hash.into(),
            account_proof: proof.account_proof.into_iter().map(Into::into).collect(),
            key: U256::from_be_bytes(proof.storage_proof[0].key.as_b256().0),
            value: proof.storage_proof[0].value.into(),
            storage_proof: proof.storage_proof[0]
                .clone()
                .proof
                .into_iter()
                .map(Into::into)
                .collect(),
        });
    }

    #[test]
    fn starknet() {
        // {"jsonrpc":"2.0","method":"starknet_getStorageProof","params":[{"block_number":3996475}, [], ["0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e"], [{"contract_address":"0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e", "storage_keys":["0x03d0f817b2e6b145a39886c95257e1bade33bc907b2125d2b4b93ced393d8e6b"]}]],"id":1}

        let proof: Vec<MerkleNode> = serde_json::from_str(
            r#"
[
  {
    "left": "0x30095df0bd831363806a03fecd7e73f1155881353aa0019bf04b4f5d1fea821",
    "right": "0x748cd654d464a07a7125dd1f59f3af6440bd1cb91e3eabd00e25bb67e18f47"
  },
  {
    "left": "0xbc8378d6f911a210f16ccb10bf3dfe70015297467da04ff31a64e8af0171e1",
    "right": "0x4929406f1a89910c18c385a255314e9087ee08d2d43bac141629fafe11d32ee"
  },
  {
    "left": "0x572aad9316302c0b1d289941b21505e5804eb65dc3cd8a4d0f44d93accf4841",
    "right": "0x2744655003106431884eca542ea98dc49bf7893b2f91eadc3ee2680ab8ef277"
  },
  {
    "left": "0x2f78a85e8c9885e3a59d501cdeea93ba0187a1ae01f359707d8afd9f74baee5",
    "right": "0x56d529ad090e5e0128d597230161c1c1bce16748c6290c544f8e79d468361a"
  },
  {
    "left": "0x4fe9a703e6d1058bb0e9877953c94c9c90b86a701c8f200fd9d44a923d1a5c7",
    "right": "0x29214ce46be3b69a8df879702e92572a66a2233720ac243ce6e5c7c66611d3"
  },
  {
    "left": "0x620c91dad524cc9b3373395f796f6da612ed0dca4878522a164d37adea3e360",
    "right": "0x7e0297ed3043db28612d7487b759ae291a5da3bb630d8af5aa010927829769e"
  },
  {
    "left": "0x6c06820feb357e347fd0af611070e8f9f83b046d230cf845adacd22dbc54cff",
    "right": "0x7e2b13cc29dfaf25fd8f5e8f18f259da549223dcdb9e8a91cce8178bcd17901"
  },
  {
    "left": "0x14d8b2f2795217c6ee4ac6a8e3fb9bde9baa4680411ec63f73b887f040fd50e",
    "right": "0x8eeb881ead580d16eebc8b3ecebc3816eb4bab2573442668242eb9fd73ac57"
  },
  {
    "left": "0x6bb29dd06cfca37492631004ea10b670b6ac104f2ccbe40e0889c790ff8faa3",
    "right": "0x4b2e6799b800af6b60ee81abb0f27c9fd42bb728b7ad60cdf56b3be4ddc5f4b"
  },
  {
    "left": "0x1eb10eac884e2d2bde5aaa6bac9adce4ba54f0961a4c00c21676fba0d8852fb",
    "right": "0xbce16eef6886a0ae81de15690d53085d74e35d4978305b1089c17fd6a4597d"
  },
  {
    "left": "0x498835c43eaae60a95bcd4e8e3bc89e524e41cf570d13fad12892ec6ec2ee40",
    "right": "0x46abdeb1716e0fc230793bf94fe2493d7699d4d178f6b19d0774a2d1ffbbc7c"
  },
  {
    "left": "0x5595953da4821e203284a77330b290d121340f93ec829334aacf46e968af880",
    "right": "0x50337184a321e3b087bef0290a8749f51a1bde4b07208efbd37cf1bb875cf96"
  },
  {
    "child": "0x4b6a88ec5cc90586fd5c285ae20224503306e5b304c5ddb26e643a53add589b",
    "length": 1,
    "path": "0x0"
  },
  {
    "left": "0x5169bd75bfe69fda75b0540c786db80c94daf757648db7697fad73bc822fc41",
    "right": "0x20517e604b1e1a4e4f43218586b02d0008a15e08e95b12d8f84cea400e2a58c"
  },
  {
    "left": "0x3b8af7a5e04753f361e87b29efc2a381cbb2385db4a23cd0ca369a8836d5046",
    "right": "0x6c900eb816f9c72490ed8f4e5d212e4a97f5f455b499d1108073a4fa00e66ad"
  },
  {
    "left": "0x2402022ec9f57d4a46f8eac414a7f5750b6080a155385e43381b129b7afc809",
    "right": "0x3de8920b04a078cb1f4b9a28446fc23ca1b54fa9515e4d9824a47603fedd4a5"
  },
  {
    "left": "0x26a1526df622c907d8785994112b6883f3aac08990f83280fab885c3b44ea98",
    "right": "0x1838240aa374acd6debf6d116ebb94a7263817c4d669ee882065a3b54528124"
  },
  {
    "child": "0x7ab8b6072e9d0957760c6d309c5ae7f27edaa5a85d1a05ee2ce1383872970ca",
    "length": 226,
    "path": "0x32c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e"
  },
  {
    "left": "0x5ea95db2890aab02eac34ee3ec2505449de85bab00bea978be1e29c708fdbbb",
    "right": "0x1f57fd6e3d709385f713877097c605a862a7e4bc066972e063700ce159df3da"
  },
  {
    "left": "0x1ccef54c3f4f8b21b45a0d6e8fb027c8bdd34d6f2ff9f42776a581441b69aaf",
    "right": "0x1f3c21a472345b5c074977d15481949b58100a3a51235ceb30263807779983c"
  },
  {
    "left": "0x708c497a0cc53dad98c127d954232cab3bee84a6bdd03efb57ffd2abb030b6",
    "right": "0x394c6c35bf159839dad252ef79150932b351732e2eee91f97486ef6c17e5712"
  },
  {
    "left": "0x1569dc0213aff99902f23c4a0a3f1b6d22220744f8cc12f6b516266758f9891",
    "right": "0x5b46548495f8969e54a116e8eb0b3f49d3fcdec358c3e1ba584c88d6253229d"
  },
  {
    "left": "0x6546df831ec386564b14284cf4a6f54321d8d719472754f6878dc07d475173e",
    "right": "0x33f9aaf486d92e8f5d4dd7c8ca3b7df694c01d31c060dcc58d80454f89b992e"
  },
  {
    "left": "0x4fb8ace2d0cf884a3a070718b82a2568d16e72cf582b702592e2d553b1885fe",
    "right": "0x3759e400e3bf0166eaa83e45494007bff64867c2f383e9b41d953471d62e76e"
  },
  {
    "left": "0x4257e8c08a26ee33a64622cffff95a83932ac81246c3ffbb2d8361aaae2d2b1",
    "right": "0x6e0f0db09122b2b46deb5fd2269b4587947b937cf14874bbac7c15883ca3bd8"
  },
  {
    "left": "0x5c9a95965d8c446e9bc5e968d33ce077d1e705c0fcdefb4f836254843f63c21",
    "right": "0x6cbbf2e225b06c542d324a9c3a55d792a5c95cb98543bfdd9d086e227f015d9"
  }
]
"#,
        )
        .unwrap();

        let key =
            Felt::from_hex("0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e")
                .unwrap();

        let value =
            Felt::from_hex("0x7ab8b6072e9d0957760c6d309c5ae7f27edaa5a85d1a05ee2ce1383872970ca")
                .unwrap();

        // contracts_proof.contract_leaves_data.storage_root
        let root =
            Felt::from_hex("0x2c6e3ddcdcf9bcd4b9e01c4b94408b6cf8b82ca9a1b40d808612483278b5afb")
                .unwrap();

        let res = starknet_storage_verifier::verify_proof::<PedersenHash>(root, key, value, &proof)
            .unwrap();

        assert_eq!(res, Membership::Membership);

        let h256 = |felt: Felt| H256::new(felt.to_be_bytes());

        let starknet_proof = StarknetProof {
            root: h256(root),
            key: h256(key),
            value: h256(value),
            proof: proof
                .into_iter()
                .map(|node| match node {
                    MerkleNode::BinaryNode { left, right } => SerdeMerkleNode::BinaryNode {
                        left: h256(left),
                        right: h256(right),
                    },
                    MerkleNode::EdgeNode {
                        path,
                        length,
                        child,
                    } => SerdeMerkleNode::EdgeNode {
                        path: h256(path),
                        length,
                        child: h256(child),
                    },
                })
                .collect::<Vec<_>>(),
        };

        dbg!(&starknet_proof);

        write_proof(&Proof::Starknet(starknet_proof));
    }
}

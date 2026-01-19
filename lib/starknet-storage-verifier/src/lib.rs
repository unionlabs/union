#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;

use bitvec::{order::Msb0, vec::BitVec};
use starknet_crypto::{pedersen_hash, poseidon_hash};
pub use starknet_types::{Felt, MerkleNode};

type CryptoFelt = starknet_crypto::Felt;

fn to_crypto_felt(felt: Felt) -> CryptoFelt {
    starknet_crypto::Felt::from_bytes_be(&felt.to_be_bytes())
}

pub trait FeltHash {
    fn hash(a: CryptoFelt, b: CryptoFelt) -> CryptoFelt;
}

pub enum PedersenHash {}

impl FeltHash for PedersenHash {
    fn hash(a: CryptoFelt, b: CryptoFelt) -> CryptoFelt {
        pedersen_hash(&a, &b)
    }
}

pub enum PoseidonHash {}

impl FeltHash for PoseidonHash {
    fn hash(a: CryptoFelt, b: CryptoFelt) -> CryptoFelt {
        poseidon_hash(a, b)
    }
}

pub fn hash_node<H: FeltHash>(node: &CryptoMerkleNode) -> CryptoFelt {
    match node {
        CryptoMerkleNode::BinaryNode { left, right } => H::hash(*left, *right),
        CryptoMerkleNode::EdgeNode {
            path,
            length,
            child,
        } => {
            H::hash(*child, *path) + {
                let mut length_ = [0; 32];
                // Safe as len() is guaranteed to be <= 251
                length_[31] = *length;
                CryptoFelt::from_bytes_be(&length_)
            }
        }
    }
}

#[derive(Debug)]
pub enum CryptoMerkleNode {
    BinaryNode {
        left: CryptoFelt,
        right: CryptoFelt,
    },
    EdgeNode {
        path: CryptoFelt,
        length: u8,
        child: CryptoFelt,
    },
}

pub fn felt_bits(felt: CryptoFelt) -> BitVec<u8, Msb0> {
    BitVec::from_slice(&felt.to_bytes_be())
}

#[derive(Debug, PartialEq)]
pub enum Membership {
    Membership,
    NonMembership,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("unused extra nodes provided")]
    UnusedNodes,
    #[error("value mismatch: expected {expected}, found {found}")]
    ValueMismatch { expected: Felt, found: Felt },
}

pub fn verify_proof<'a, H: FeltHash>(
    root: Felt,
    key: Felt,
    value: Felt,
    proof: impl IntoIterator<Item = &'a MerkleNode>,
) -> Result<Membership, Error> {
    let mut root = to_crypto_felt(root);

    let mut proof = proof
        .into_iter()
        .map(|n| match n {
            MerkleNode::BinaryNode { left, right } => CryptoMerkleNode::BinaryNode {
                left: to_crypto_felt(*left),
                right: to_crypto_felt(*right),
            },
            MerkleNode::EdgeNode {
                path,
                length,
                child,
            } => CryptoMerkleNode::EdgeNode {
                path: to_crypto_felt(*path),
                length: *length,
                child: to_crypto_felt(*child),
            },
        })
        .map(|n| (hash_node::<H>(&n), n))
        .collect::<BTreeMap<_, _>>();

    // https://github.com/eqlabs/pathfinder/blob/a34566b9a9f6ea6d7eb3889130d62c8f3fe6a499/crates/crypto/src/algebra/field/felt.rs#L176
    let mut remaining_path = &felt_bits(to_crypto_felt(key))[5..];

    while let Some(proof_node) = proof.remove(&root) {
        match proof_node {
            CryptoMerkleNode::BinaryNode { left, right } => {
                // Set the next hash to be the left or right hash,
                // depending on the direction
                // https://github.com/eqlabs/pathfinder/blob/a34566b9a9f6ea6d7eb3889130d62c8f3fe6a499/crates/merkle-tree/src/merkle_node.rs#L81
                root = match remaining_path[0] {
                    false => left,
                    true => right,
                };

                // Advance by a single bit
                remaining_path = &remaining_path[1..];
            }
            CryptoMerkleNode::EdgeNode {
                path,
                length,
                child,
            } => {
                let path_view = &felt_bits(path)[5..][(251 - length) as usize..251];
                let remaining_path_view = &remaining_path[..length as usize];

                if path_view != remaining_path_view {
                    // If paths don't match, we've found a proof of non membership because
                    // we:
                    // 1. Correctly moved towards the target insofar as is possible, and
                    // 2. hashing all the nodes along the path does result in the root hash,
                    //    which means
                    // 3. the target definitely does not exist in this tree
                    // return Some(Membership::NonMember);
                    return Ok(Membership::NonMembership);
                }

                // Set the next hash to the child's hash
                root = child;

                // Advance by the whole edge path
                remaining_path = &remaining_path[length as usize..];
            }
        }
    }

    if !proof.is_empty() {
        return Err(Error::UnusedNodes);
    }

    if root.to_bytes_be() != value.to_be_bytes() {
        return Err(Error::ValueMismatch {
            expected: value,
            found: Felt::from_be_bytes(root.to_bytes_be()),
        });
    }

    Ok(Membership::Membership)
}

#[test]
fn contract_membership() {
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

    let key = Felt::from_hex("0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e")
        .unwrap();
    let value = {
        let class_hash = CryptoFelt::from_hex_unchecked(
            "0x69b893a8b6e1bf94740e33d9584a01295510f3b51f024d9833b2acaf1be4045",
        );
        let nonce = CryptoFelt::from_hex_unchecked("0x0");
        let storage_root = CryptoFelt::from_hex_unchecked(
            "0x2c8771df74e758b1fed285eef0cd07cb84b55abfabfb0d6a0f1b7b3aff761fa",
        );

        // https://docs.starknet.io/learn/protocol/state#the-contract-trie
        Felt::from_be_bytes(
            pedersen_hash(
                &pedersen_hash(&pedersen_hash(&class_hash, &storage_root), &nonce),
                &CryptoFelt::ZERO,
            )
            .to_bytes_be(),
        )
    };

    dbg!(&value);

    // contracts_proof.contract_leaves_data.storage_root
    let root = Felt::from_hex("0x2c6e3ddcdcf9bcd4b9e01c4b94408b6cf8b82ca9a1b40d808612483278b5afb")
        .unwrap();

    let res = verify_proof::<PedersenHash>(root, key, value, &proof).unwrap();

    assert_eq!(res, Membership::Membership);
}

#[test]
fn contract_storage_membership() {
    // {"jsonrpc":"2.0","method":"starknet_getStorageProof","params":[{"block_number":3996475}, [], ["0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e"], [{"contract_address":"0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e", "storage_keys":["0x03d0f817b2e6b145a39886c95257e1bade33bc907b2125d2b4b93ced393d8e6b"]}]],"id":1}

    let proof: Vec<MerkleNode> = serde_json::from_str(
        r#"
[
  {
    "child": "0x56ef8be5dc020f5437e6611ca54e4f78c245c2e49592de3db76abfe0998eb22",
    "length": 1,
    "path": "0x0"
  },
  {
    "left": "0x778ebcee8874705995f911f4c7edaac1748f5b583c146e9c37dd48e30d11cfd",
    "right": "0x219c6c95d8eeee035ffa9bd5d301175569b6151874f157c4f9546f0073710db"
  },
  {
    "child": "0x49ff5b3a7d38e2b50198f408fa8281635b5bc81ee49ab87ac36c8324c214427",
    "length": 246,
    "path": "0x10f817b2e6b145a39886c95257e1bade33bc907b2125d2b4b93ced393d8e6b"
  },
  {
    "left": "0x2f26113a475400d1bc8dd0e9b2ea2fd548b5abe22e158568a9395780a58e2c1",
    "right": "0x310643b32d81e4ee4cf0723859775500280e40ebef3e3458ffec38d16911607"
  },
  {
    "left": "0x3e817efe680adf2c6072ed9f795191640549196d00a989a168cb96b1a2ffdb7",
    "right": "0x7415330dba1c847123bd543bbb684771a5706a03814c4919d437abaf070a169"
  },
  {
    "left": "0x1673d50ff33986889bd487dc1dcaccae706f620e54d4b7afa9821e1408da49b",
    "right": "0x50a62b544461e0d83bac95f26c7e0d906433b3f777ff5df13d074c45237b8c6"
  }
]
"#,
    )
    .unwrap();

    let key = Felt::from_hex("0x03d0f817b2e6b145a39886c95257e1bade33bc907b2125d2b4b93ced393d8e6b")
        .unwrap();
    let value = Felt::from_hex("0x49ff5b3a7d38e2b50198f408fa8281635b5bc81ee49ab87ac36c8324c214427")
        .unwrap();

    // contracts_proof.contract_leaves_data.storage_root
    let root = Felt::from_hex("0x2c8771df74e758b1fed285eef0cd07cb84b55abfabfb0d6a0f1b7b3aff761fa")
        .unwrap();

    let res = verify_proof::<PedersenHash>(root, key, value, &proof).unwrap();

    assert_eq!(res, Membership::Membership);
}

#[test]
fn contract_storage_non_membership() {
    // {"jsonrpc":"2.0","method":"starknet_getStorageProof","params":[{"block_number":3996475}, [], ["0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e"], [{"contract_address":"0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e", "storage_keys":["0x0"]}]],"id":1}

    let proof: Vec<MerkleNode> = serde_json::from_str(
        r#"
[
  {
    "child": "0x1611612cfc15e76d48f227e845073c85f4f55c3ef35921f169f8c475f6a819f",
    "length": 1,
    "path": "0x1"
  },
  {
    "left": "0x778ebcee8874705995f911f4c7edaac1748f5b583c146e9c37dd48e30d11cfd",
    "right": "0x219c6c95d8eeee035ffa9bd5d301175569b6151874f157c4f9546f0073710db"
  },
  {
    "child": "0x56ef8be5dc020f5437e6611ca54e4f78c245c2e49592de3db76abfe0998eb22",
    "length": 1,
    "path": "0x0"
  }
]
"#,
    )
    .unwrap();

    let key = Felt::from_hex("0x0").unwrap();
    let value = Felt::from_hex("0x0").unwrap();

    // contracts_proof.contract_leaves_data.storage_root
    let root = Felt::from_hex("0x2c8771df74e758b1fed285eef0cd07cb84b55abfabfb0d6a0f1b7b3aff761fa")
        .unwrap();

    let res = verify_proof::<PedersenHash>(root, key, value, &proof).unwrap();

    assert_eq!(res, Membership::NonMembership);
}

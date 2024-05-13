use gnark_mimc::{mimc_sum_bl12377, MiMCBls12377Constants};
use serde::{Deserialize, Serialize};
use unionlabs::{
    errors::{ExpectedLength, InvalidLength},
    hash::H256,
    ByteArrayExt,
};

// https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/path/PathResolver.java#L27
pub const SUB_TRIE_ROOT_PATH: [u8; 1] = [1];

// https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/node/LeafType.java#L21-L24
pub const LEAF_TYPE_VALUE: u8 = 0x16;

// https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/path/PathGenerator.java#L34
pub fn bytes_to_leaf_path(bytes: &[u8], terminator_path: u8) -> Vec<u8> {
    let mut path = vec![0u8; bytes.len() * 2 + 1];
    let mut j = 0;
    for b in bytes.iter().skip(j) {
        path[j] = b >> 4 & 15;
        path[j + 1] = b & 15;
        j += 2;
    }
    path[j] = terminator_path;
    path
}

// TODO: use bitvec
// https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/path/PathResolver.java#L82
pub fn node_index_to_bytes(trie_depth: usize, node_index: u64) -> Vec<u8> {
    hex::decode(format!("{node_index:0>trie_depth$b}")).unwrap()
}

// https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/path/PathResolver.java#L71
pub fn get_leaf_path(trie_depth: usize, node_index: u64) -> Vec<u8> {
    [
        SUB_TRIE_ROOT_PATH.as_ref(),
        &bytes_to_leaf_path(
            &node_index_to_bytes(trie_depth, node_index),
            LEAF_TYPE_VALUE,
        ),
    ]
    .concat()
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmptyLeafNode {}

impl EmptyLeafNode {
    // https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/node/EmptyLeafNode.java#L80
    pub const HASH: H256 = H256([0u8; 32]);

    pub fn hash(&self) -> H256 {
        Self::HASH
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafNode {
    pub previous: H256,
    pub next: H256,
    pub hashed_key: H256,
    pub value: H256,
}

impl LeafNode {
    // https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/node/LeafNode.java#L56
    pub fn hash(&self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        mimc_sum_bl12377(
            constants,
            [
                self.previous.as_ref(),
                self.next.as_ref(),
                self.hashed_key.as_ref(),
                self.value.as_ref(),
            ]
            .concat(),
        )
    }

    pub fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength> {
        let value = value.as_ref();
        let values = <[u8; 128]>::try_from(value).map_err(|_| InvalidLength {
            expected: ExpectedLength::Exact(128),
            found: value.len(),
        })?;
        Ok(Self {
            previous: values.array_slice::<0, 32>().into(),
            next: values.array_slice::<32, 32>().into(),
            hashed_key: values.array_slice::<64, 32>().into(),
            value: values.array_slice::<96, 32>().into(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchNode {
    pub left: H256,
    pub right: H256,
}

impl BranchNode {
    // https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/node/BranchNode.java#L82
    pub fn hash(&self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        mimc_sum_bl12377(
            constants,
            [self.left.as_ref(), self.right.as_ref()].concat(),
        )
    }

    pub fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength> {
        let value = value.as_ref();
        let values = <[u8; 64]>::try_from(value).map_err(|_| InvalidLength {
            expected: ExpectedLength::Exact(64),
            found: value.len(),
        })?;
        Ok(Self {
            left: values.array_slice::<0, 32>().into(),
            right: values.array_slice::<32, 32>().into(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootNode {
    pub next_free_node: H256,
    pub child_hash: H256,
}

impl RootNode {
    // Same as branch
    pub fn hash(&self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        mimc_sum_bl12377(
            constants,
            [self.next_free_node.as_ref(), self.child_hash.as_ref()].concat(),
        )
    }

    pub fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength> {
        let value = value.as_ref();
        let values = <[u8; 64]>::try_from(value).map_err(|_| InvalidLength {
            expected: ExpectedLength::Exact(64),
            found: value.len(),
        })?;
        Ok(Self {
            next_free_node: values.array_slice::<0, 32>().into(),
            child_hash: values.array_slice::<32, 32>().into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    EmptyLeaf(EmptyLeafNode),
    Leaf(LeafNode),
    Branch(BranchNode),
    Root(RootNode),
}

impl Node {
    pub fn hash(&self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        match self {
            Node::Leaf(node) => node.hash(constants),
            Node::Branch(node) => node.hash(constants),
            Node::Root(node) => node.hash(constants),
            Node::EmptyLeaf(node) => Ok(node.hash()),
        }
    }
}

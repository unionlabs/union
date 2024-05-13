use gnark_mimc::{mimc_sum_bl12377, MiMCBls12377, MiMCBls12377Constants};
use serde::{Deserialize, Serialize};
use unionlabs::{
    errors::InvalidLength,
    hash::{H160, H256},
    linea::{
        account::{MimcSafeBytes, ZkAccount},
        proof::{MerklePath, MerkleProof, NonInclusionProof},
    },
    uint::U256,
};

use crate::node::{get_leaf_path, BranchNode, Direction, LeafNode, Node, RootNode, Terminator};

// https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/trie/src/main/java/net/consensys/shomei/trie/ZKTrie.java#L64C1-L65C47
pub const ZK_TRIE_DEPTH: usize = 40;

pub trait ZkKey {
    fn hash(self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error>;
}

pub trait ZkValue {
    type Key: ZkKey;

    fn hash(self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error>;

    fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength>
    where
        Self: Sized;
}

impl ZkKey for H160 {
    fn hash(self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        let mut padded_key = [0u8; 32];
        padded_key[12..32].copy_from_slice(self.as_ref());
        mimc_sum_bl12377(constants, padded_key)
    }
}

impl ZkValue for ZkAccount {
    type Key = H160;

    fn hash(self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        mimc_sum_bl12377(constants, self.into_bytes())
    }

    fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength>
    where
        Self: Sized,
    {
        ZkAccount::decode(value)
    }
}

impl ZkKey for H256 {
    fn hash(self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        mimc_sum_bl12377(constants, MimcSafeBytes::from(self.0).into_bytes())
    }
}

impl ZkValue for H256 {
    type Key = H256;

    fn hash(self, constants: &MiMCBls12377Constants) -> Result<H256, gnark_mimc::Error> {
        mimc_sum_bl12377(constants, MimcSafeBytes::from(self.0).into_bytes())
    }

    fn decode(value: impl AsRef<[u8]>) -> Result<Self, InvalidLength>
    where
        Self: Sized,
    {
        H256::try_from(value.as_ref())
    }
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("invalid direction {0:?}")]
    InvalidDirection(Direction),
    #[error("missing root node")]
    MissingRoot,
    #[error("missing leaf node")]
    MissingLeaf,
    #[error("invalid field element length: {0}")]
    InvalidLength(InvalidLength),
    #[error("invalid mimc hashing: {0:?}")]
    MimcError(gnark_mimc::Error),
    #[error("could not decode leaf value")]
    CouldNotDecodeValue,
    #[error("invalid trie root, actual: {actual}, expected: {expected}")]
    RootMismatch { actual: H256, expected: H256 },
    #[error("invalid subtrie root, actual: {actual}, expected: {expected}")]
    SubtrieRootMismatch { actual: H256, expected: H256 },
    #[error("key mismatch, actual: {actual}, expected: {expected}")]
    KeyMismatch { actual: H256, expected: H256 },
    #[error("value mismatch, actual: {actual}, expected: {expected}")]
    ValueMismatch { actual: H256, expected: H256 },
    #[error("non adjacent node, left: {left:?}, right: {right:?}")]
    NonAdjacentNode {
        left: Box<LeafNode>,
        right: Box<LeafNode>,
    },
    #[error("key not in center, left: {left}, key: {key} right: {right}")]
    KeyNotInCenter { left: H256, key: H256, right: H256 },
}

impl From<InvalidLength> for Error {
    fn from(value: InvalidLength) -> Self {
        Self::InvalidLength(value)
    }
}

impl From<gnark_mimc::Error> for Error {
    fn from(value: gnark_mimc::Error) -> Self {
        Self::MimcError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifiablePath {
    pub root: RootNode,
    pub path: Vec<Node>,
    pub leaf: LeafNode,
}

impl TryFrom<&MerklePath> for VerifiablePath {
    type Error = Error;
    fn try_from(value: &MerklePath) -> Result<Self, Self::Error> {
        let root = RootNode::decode(
            value
                .proof_related_nodes
                .first()
                .ok_or(Error::MissingRoot)?,
        )?;
        let leaf = LeafNode::decode(value.proof_related_nodes.last().ok_or(Error::MissingLeaf)?)?;
        // Skip the root as we manually check against it
        let inner_path_len = value.proof_related_nodes.len() - 1;
        let path = value
            .proof_related_nodes
            .iter()
            .skip(1)
            .take(inner_path_len)
            .map(|raw_node| {
                if raw_node.len() == MiMCBls12377::FIELD_ELEMENT_BYTES_LEN * 2 {
                    BranchNode::decode(raw_node).map(Node::Branch)
                } else {
                    LeafNode::decode(raw_node).map(Node::Leaf)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(VerifiablePath { root, path, leaf })
    }
}

pub fn verify<V: ZkValue + Clone>(
    constants: &MiMCBls12377Constants,
    proof: &MerkleProof,
    root: H256,
    key: V::Key,
) -> Result<Option<V>, Error> {
    match proof {
        MerkleProof::Inclusion(inclusion_proof) => verify_inclusion_and_key::<V>(
            constants,
            inclusion_proof.leaf_index,
            &inclusion_proof.proof,
            root,
            key,
        )
        .map(|(_, value)| Some(value)),
        MerkleProof::NonInclusion(noninclusion_proof) => {
            verify_noninclusion::<V>(constants, noninclusion_proof, root, key).map(|_| None)
        }
    }
}

pub fn verify_noninclusion<V: ZkValue + Clone>(
    constants: &MiMCBls12377Constants,
    noninclusion_proof: &NonInclusionProof,
    root: H256,
    key: V::Key,
) -> Result<(), Error> {
    // left in root
    let (left_path, _) = verify_inclusion::<V>(
        constants,
        noninclusion_proof.left_leaf_index,
        &noninclusion_proof.left_proof,
        root,
    )?;
    // right in root
    let (right_path, _) = verify_inclusion::<V>(
        constants,
        noninclusion_proof.right_leaf_index,
        &noninclusion_proof.right_proof,
        root,
    )?;
    // N+.Prev == i-
    if U256::from_be_bytes(right_path.leaf.previous.0)
        != U256::from(noninclusion_proof.left_leaf_index)
    {
        return Err(Error::NonAdjacentNode {
            left: left_path.leaf.into(),
            right: right_path.leaf.into(),
        });
    }
    // N-.Next == i+
    if U256::from_be_bytes(left_path.leaf.next.0) != U256::from(noninclusion_proof.right_leaf_index)
    {
        return Err(Error::NonAdjacentNode {
            left: left_path.leaf.into(),
            right: right_path.leaf.into(),
        });
    }
    // HKey- < hash(k) < HKey+
    let recomputed_key = key.hash(constants)?;
    if left_path.leaf.hashed_key >= recomputed_key || recomputed_key >= right_path.leaf.hashed_key {
        return Err(Error::KeyNotInCenter {
            left: left_path.leaf.hashed_key,
            key: recomputed_key,
            right: right_path.leaf.hashed_key,
        });
    }
    Ok(())
}

pub fn verify_inclusion_and_key<V: ZkValue + Clone>(
    constants: &MiMCBls12377Constants,
    leaf_index: u64,
    merkle_path: &MerklePath,
    root: H256,
    key: V::Key,
) -> Result<(VerifiablePath, V), Error> {
    let (verifiable_path, value) = verify_inclusion::<V>(constants, leaf_index, merkle_path, root)?;
    let recomputed_key = key.hash(constants)?;
    if verifiable_path.leaf.hashed_key != recomputed_key {
        return Err(Error::KeyMismatch {
            actual: recomputed_key,
            expected: verifiable_path.leaf.hashed_key,
        });
    }
    Ok((verifiable_path, value))
}

pub fn verify_inclusion<V: ZkValue + Clone>(
    constants: &MiMCBls12377Constants,
    leaf_index: u64,
    merkle_path: &MerklePath,
    root: H256,
) -> Result<(VerifiablePath, V), Error> {
    let leaf_path = get_leaf_path(ZK_TRIE_DEPTH, leaf_index);
    let verifiable_path = VerifiablePath::try_from(merkle_path)?;
    // Verify the top root hash
    let recomputed_root = verifiable_path.root.hash(constants)?;
    if root != recomputed_root {
        return Err(Error::RootMismatch {
            actual: recomputed_root,
            expected: root,
        });
    }
    // The value is decoded then hashed, the decoding is required as the value
    // may need to be transformed before being hashed (ZkAccount keccak field
    // that need to be split in two elements to fit in the scalar field for
    // instance)
    let value = V::decode(&merkle_path.value).map_err(|_| Error::CouldNotDecodeValue)?;
    // Verify that the value is related to the leaf
    let recomputed_value = value.clone().hash(constants)?;
    if verifiable_path.leaf.value != recomputed_value {
        return Err(Error::ValueMismatch {
            actual: recomputed_root,
            expected: verifiable_path.leaf.value,
        });
    }
    // The algorithm we use is slightly more explicit, we actually extract both
    // the root so the inner path must exclude it
    // Minus root/leaf
    let inner_path = leaf_path.into_iter().skip(1);
    let hash = |x| mimc_sum_bl12377(constants, x);
    // Starts with the leaf hash, then recursively walk back to the tip of the
    // tree.
    let subtrie_root = verifiable_path.path.iter().zip(inner_path).rev().try_fold(
        H256::default(),
        |current_hash, (node, direction)| -> Result<H256, Error> {
            let node_hash = node.hash(constants)?;
            let next_hash = match direction {
                // We went on the left branch, we hash against the right sibling
                Direction::Left => hash([current_hash.as_ref(), node_hash.as_ref()].concat())?,
                // We went on the right branch, we hash against the left sibling
                Direction::Right => hash([node_hash.as_ref(), current_hash.as_ref()].concat())?,
                // We are at the leaf level, we start with it's hash
                Direction::Terminator(Terminator::Value) => node_hash,
            };
            Ok(next_hash)
        },
    )?;
    // Verify the subtrie root
    if verifiable_path.root.child_hash != subtrie_root {
        return Err(Error::SubtrieRootMismatch {
            actual: subtrie_root,
            expected: verifiable_path.root.child_hash,
        });
    }
    Ok((verifiable_path, value))
}

#[cfg(test)]
mod tests {
    use gnark_mimc::{new_mimc_bls12_377, new_mimc_constants_bls12_377};
    use hex_literal::hex;
    use unionlabs::{
        hash::{H160, H256},
        linea::{account::ZkAccount, proof::GetProof},
    };

    use super::verify;

    #[test]
    fn test_account_and_storage_inclusion() {
        let raw = r#"
{
  "accountProof": {
    "key": "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789",
    "leafIndex": 65362,
    "proof": {
      "proofRelatedNodes": [
        "0x00000000000000000000000000000000000000000000000000000000000120fe0393507c456718a986386c7923fe68b87c29d83ac7f7ce1cdb49afc7e66a4771",
        "0x008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809",
        "0x060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b",
        "0x0a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef322550a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef32255",
        "0x01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d",
        "0x090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484",
        "0x11c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f30611c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f306",
        "0x07f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d4807f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d48",
        "0x0f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd80f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd8",
        "0x0cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e27390cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e2739",
        "0x014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b",
        "0x11c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca55411c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca554",
        "0x1092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c221092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c22",
        "0x0969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a890969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a89",
        "0x079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368",
        "0x004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458",
        "0x0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd",
        "0x0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c",
        "0x0defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea80defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea8",
        "0x0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b",
        "0x1276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f51276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f5",
        "0x02a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b202a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b2",
        "0x070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978",
        "0x0133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed10133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed1",
        "0x09cbd26c486bc2217bce59337120283f655a7ba65075f98059249f471812d0480b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f84",
        "0x0c5c4d122720c4d6e7866d9b6bc6171c6259be90095976b665406ccf2dc6a8950305d7ebd7da4f82f061632eb7ec0c3060f51af848661d479bb64003f0fc5342",
        "0x0c4762f6af9f09a529e70f0b34b7afafe2bba8944eccdcdb95cf13e0ff00ab2209d8b650f132967dba1764abe34c3d446311503ba7712d5f474a6e159b085b5f",
        "0x0c3474a51e2654aca28b15add106ac676d92b9416ee788ac0b88873e77a009660176016fa85f1ba2375f784c72fae85763e12018e3e781c306f97ad9f826a22e",
        "0x108459110262f154aef2d43fce77d314a7ec867f0068987716ff51582847e498009a6be6c408befa4eb7e6141fd427a2ce6489d50bf5f9de6bde9e100ada3482",
        "0x118d3c53f9a3ea556029e867af93e9b4450cbacdf4dff29859e399ae16468e5102cefeff18d2980c8a9253c4609506472ba4764ea99efa6324dacf34740d9f05",
        "0x005d88c799974510f99c04afdaba0f6b8f62edd55d8d89910009e148385a72c30a8fac91e2023660e8ac50ff082578361ba0901b16fe691f9b78044cbf6d1c4b",
        "0x106f788c7d5990bec78f6c9cadd15604c99a8f1d56c875d324bb5ece63d83f3606694c69c43303aa1c614d60ed8fc66838f368b134cfc1ab00b6c83b2b5b3c8c",
        "0x0ba8fdb8888982dde981f8e2cc9177c8c3ce0607661e113604e436951776de9c0b9ec8fec4b0696c73e04fd6bee4aa345633d23ef0c6bc4e4bbcf757af2677f0",
        "0x0f5ae90881ea3398fd1a14fb83babc2335dfc4e6298aada1d827042d67dea48f0dee0a62e8ff86baddb091105d845c862089fe2f1963cd3798d636035da4d518",
        "0x03d41bdb96726bf7f745784e42eef043c8b797f788d9720e36e460502e14c9fb0923e0e0228d2fe8619e30581e3e225d4e99e0daa011e15ac34c28fa30ea2989",
        "0x11335bc4bf8a15d8c116cbdfe74242e80c7f60ac1a614d00f99fb9e1148126930502f7b7740708503e3858bc6df707cf4a1a751bcef3f2a5eb6eff9d8efa5cf8",
        "0x0d60d90907794deaabe1e532a128e17ac94ec30339f3e367bc9ecd0aa40fd8b6009f71be21f99f29acd62b42787c99e5192646f808306fff0960ef5cd9a5ac16",
        "0x0a6fd861ba25def420f5503fbbc4e0de2e54b4fbf0b22364e4a188eaf72ac58c02e49a2a28faca35409f471b4d981951aeabba2f091a427a2e88c53d1c7eeed3",
        "0x0a93ecccd90368342584da9a8623e89a7a71d36f1da58d9874d50c045587138b0476d671e749bd2cd45fe416e1409caa22863f8cebdf926920a9f68b150d92d7",
        "0x0821de61351452c22cf6bdafcd85be9a8cb3c2ad0af51f871d44221575785f9d12200803e31923cc68d6c9b906876643688e3a7ccb21264f933028b060564e4d",
        "0x0000000000000000000000000000000000000000000000000000000000001c1e000000000000000000000000000000000000000000000000000000000000a354000036e661469dd70081ada16334d16a4049a124e261cd93def5fff88f85afda01b285fb7d6e0c7e05505a348777221f3c9fb491bbbbea4853e62e93f415efe7",
        "0x000000000000000000000000000000000000000000000000000000000000894100000000000000000000000000000000000000000000000000000000000002db104a10331d6a854148a10b11c19cf2abae0412c9909ecefca54adc135ee57a950481fe75941093272afb1f8f76353afad6b89b1c19c383b07730c6f160b59243"
      ],
      "value": "0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000017d7498da306d2911280c3481d8b1510e16062ffaa631812c3ca53639329c1577354f0e4cd850dde1255c33de5e8c499e72ca1f49352847124c0dbfc30d0374d4d5d5e7cddb83c7ac93c806e738300b5357ecdc2e971d6438d34d8e4e17b99b758b1f9cac91c8e700000000000000000000000000000000000000000000000000000000000005c89"
    }
  },
  "storageProofs": [
    {
      "key": "0x975227e2a924779fb36829b74e9ab66f8d906444c0efb23059aaf437a9254f64",
      "leafIndex": 138,
      "proof": {
        "proofRelatedNodes": [
          "0x00000000000000000000000000000000000000000000000000000000000003a90c382f6158633dfaf5ea90b4b6aef05e0171d9c5e97a2f3aa41c3944e2d08f7c",
          "0x008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809",
          "0x060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b",
          "0x0a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef322550a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef32255",
          "0x01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d",
          "0x090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484",
          "0x11c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f30611c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f306",
          "0x07f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d4807f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d48",
          "0x0f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd80f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd8",
          "0x0cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e27390cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e2739",
          "0x014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b",
          "0x11c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca55411c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca554",
          "0x1092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c221092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c22",
          "0x0969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a890969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a89",
          "0x079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368",
          "0x004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458",
          "0x0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd",
          "0x0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c",
          "0x0defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea80defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea8",
          "0x0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b",
          "0x1276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f51276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f5",
          "0x02a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b202a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b2",
          "0x070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978",
          "0x0133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed10133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed1",
          "0x0b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f840b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f84",
          "0x0f3f9cf1e5ba6bdbb6daafc405bcceac97270fe89265b6a0faa2ba4bfd5cbf5d0f3f9cf1e5ba6bdbb6daafc405bcceac97270fe89265b6a0faa2ba4bfd5cbf5d",
          "0x08b60393196453ee74fdf240449d9aa2569875b43596ea2621eecda8d8909acd08b60393196453ee74fdf240449d9aa2569875b43596ea2621eecda8d8909acd",
          "0x10c439d656480d21a08c068717556fb8104a7a76e26f60e393ce4e36ae21e07b10c439d656480d21a08c068717556fb8104a7a76e26f60e393ce4e36ae21e07b",
          "0x09ea86c5cd59ac4bfca4e46e7b50bb37c8327350888ba71112ecf3f5093baaef09ea86c5cd59ac4bfca4e46e7b50bb37c8327350888ba71112ecf3f5093baaef",
          "0x0b971345bfa43e192ca2fb1c9ddd19f2dddf461243b1a54fdd5a4d581f850c110b971345bfa43e192ca2fb1c9ddd19f2dddf461243b1a54fdd5a4d581f850c11",
          "0x0edd0129edd35191a183ecd28cbcab2a48ad381215d8544acf35248639835dcd0edd0129edd35191a183ecd28cbcab2a48ad381215d8544acf35248639835dcd",
          "0x0d052b80abb809f9120c6b9884fffd52dd230a8dea0e503ee37a657412f956e4124085568263d79db22e8138cdfcddb82217762c26573f47a99464a1891998c0",
          "0x05e61cac7ebd2c56b6e841e2437573d262652dab2a93cf5c87ae6c77ea6e29620b2e2ac6538353a780d865eb117c6a15c9ce5482df3f82de22341ff53ff603bb",
          "0x081d406e2e7c445affbd6879217ba8ef422de57833bfd2117c67132b7c136b80041dc4f76e0dcec4e22f176ab6a40e8cfa6f15fd3be71dffc508c7d1e49a095f",
          "0x08f74df1f6c448f34dbebc04442406cccf4e59336dbdeb8820d056584f8e5c2e000d8662808f22994b99a5a7c5888e053462f631bd6ccc1bb5cfc409c6496e29",
          "0x01535de3a78232579c22be9a44bacd4ab197dcab60c15cad6ee5783a87e8fe3e035ca4181a3a2b7660a12b44b972a9b13751b7765a87b943690afae72084dc70",
          "0x0dc279f3ab0113621f49cce7fcd58b620db8940fa536685b0f085062ef5804500f809df436769c9dca43efa53adf5d802e5d9a164cd2a43aec2dedf4109131b1",
          "0x009a05037883da4556d1eb804b43c05fba7d961bbf77d48b06d4fd4b986159f6095ab3af585bcb3df9060b1651da2360891a221de0d9325c04a49d8caa0cd800",
          "0x06cdca5c9cced457b657b1af944d068a8ab962ef5fe08550778921a429c5bb2f106bd517f2778b534d455f1d780e8d823d918499b35488788a81546c22a2b257",
          "0x04c7934d9f58f8f85be28784af049898b132dc5e80f4e96d294dcfc883736c430c2a97661da9e1fcd930e97c6184379e9a8c99ba72bb3f941a542df11ca481d6",
          "0x00000000000000000000000000000000000000000000000000000000000002b400000000000000000000000000000000000000000000000000000000000002a00324558eb3216bfae60f436ae4f80653125d6783123282af0eaa3766492ac1c012023ca7988684c6679a91abc62dbf0a5f49f4a4468e7c4c2e6de9bedce00864",
          "0x000000000000000000000000000000000000000000000000000000000000004d0000000000000000000000000000000000000000000000000000000000000277034ca60d4657a94b25f98d458f8c879b4a67d24bb34650b9ade6cb0e0a4b6847043d8792aabcc5507963792b5efd5949aa034b1f784272b07eecfa5cc8b1b1d8"
        ],
        "value": "0x0000000000000000000000000000000000000000000000000000000000000183"
      }
    }
  ]
}
        "#;

        let proof = serde_json::from_str::<GetProof>(raw).unwrap();
        let account = verify::<ZkAccount>(
            &new_mimc_constants_bls12_377(),
            &proof.account_proof,
            H256(hex!(
                "0C76548458CC04A5AA09BFFA092B32C912AEE635C1C44364EBB911286A10263D"
            )),
            H160(hex!("5ff137d4b0fdcd49dca30c7cf57e578a026d2789")),
        )
        .unwrap()
        .unwrap();
        let value = verify::<H256>(
            &new_mimc_constants_bls12_377(),
            &proof.storage_proofs[0],
            account.storage_root,
            H256(hex!(
                "975227e2a924779fb36829b74e9ab66f8d906444c0efb23059aaf437a9254f64"
            )),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            value,
            hex!("0000000000000000000000000000000000000000000000000000000000000183").into()
        );
    }

    #[test]
    fn test_storage_noninclusion() {
        let raw = r#"
{
  "accountProof": {
    "key": "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789",
    "leafIndex": 65362,
    "proof": {
      "proofRelatedNodes": [
        "0x00000000000000000000000000000000000000000000000000000000000120fe0393507c456718a986386c7923fe68b87c29d83ac7f7ce1cdb49afc7e66a4771",
        "0x008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809",
        "0x060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b",
        "0x0a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef322550a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef32255",
        "0x01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d",
        "0x090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484",
        "0x11c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f30611c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f306",
        "0x07f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d4807f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d48",
        "0x0f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd80f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd8",
        "0x0cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e27390cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e2739",
        "0x014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b",
        "0x11c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca55411c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca554",
        "0x1092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c221092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c22",
        "0x0969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a890969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a89",
        "0x079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368",
        "0x004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458",
        "0x0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd",
        "0x0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c",
        "0x0defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea80defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea8",
        "0x0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b",
        "0x1276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f51276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f5",
        "0x02a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b202a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b2",
        "0x070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978",
        "0x0133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed10133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed1",
        "0x09cbd26c486bc2217bce59337120283f655a7ba65075f98059249f471812d0480b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f84",
        "0x0c5c4d122720c4d6e7866d9b6bc6171c6259be90095976b665406ccf2dc6a8950305d7ebd7da4f82f061632eb7ec0c3060f51af848661d479bb64003f0fc5342",
        "0x0c4762f6af9f09a529e70f0b34b7afafe2bba8944eccdcdb95cf13e0ff00ab2209d8b650f132967dba1764abe34c3d446311503ba7712d5f474a6e159b085b5f",
        "0x0c3474a51e2654aca28b15add106ac676d92b9416ee788ac0b88873e77a009660176016fa85f1ba2375f784c72fae85763e12018e3e781c306f97ad9f826a22e",
        "0x108459110262f154aef2d43fce77d314a7ec867f0068987716ff51582847e498009a6be6c408befa4eb7e6141fd427a2ce6489d50bf5f9de6bde9e100ada3482",
        "0x118d3c53f9a3ea556029e867af93e9b4450cbacdf4dff29859e399ae16468e5102cefeff18d2980c8a9253c4609506472ba4764ea99efa6324dacf34740d9f05",
        "0x005d88c799974510f99c04afdaba0f6b8f62edd55d8d89910009e148385a72c30a8fac91e2023660e8ac50ff082578361ba0901b16fe691f9b78044cbf6d1c4b",
        "0x106f788c7d5990bec78f6c9cadd15604c99a8f1d56c875d324bb5ece63d83f3606694c69c43303aa1c614d60ed8fc66838f368b134cfc1ab00b6c83b2b5b3c8c",
        "0x0ba8fdb8888982dde981f8e2cc9177c8c3ce0607661e113604e436951776de9c0b9ec8fec4b0696c73e04fd6bee4aa345633d23ef0c6bc4e4bbcf757af2677f0",
        "0x0f5ae90881ea3398fd1a14fb83babc2335dfc4e6298aada1d827042d67dea48f0dee0a62e8ff86baddb091105d845c862089fe2f1963cd3798d636035da4d518",
        "0x03d41bdb96726bf7f745784e42eef043c8b797f788d9720e36e460502e14c9fb0923e0e0228d2fe8619e30581e3e225d4e99e0daa011e15ac34c28fa30ea2989",
        "0x11335bc4bf8a15d8c116cbdfe74242e80c7f60ac1a614d00f99fb9e1148126930502f7b7740708503e3858bc6df707cf4a1a751bcef3f2a5eb6eff9d8efa5cf8",
        "0x0d60d90907794deaabe1e532a128e17ac94ec30339f3e367bc9ecd0aa40fd8b6009f71be21f99f29acd62b42787c99e5192646f808306fff0960ef5cd9a5ac16",
        "0x0a6fd861ba25def420f5503fbbc4e0de2e54b4fbf0b22364e4a188eaf72ac58c02e49a2a28faca35409f471b4d981951aeabba2f091a427a2e88c53d1c7eeed3",
        "0x0a93ecccd90368342584da9a8623e89a7a71d36f1da58d9874d50c045587138b0476d671e749bd2cd45fe416e1409caa22863f8cebdf926920a9f68b150d92d7",
        "0x0821de61351452c22cf6bdafcd85be9a8cb3c2ad0af51f871d44221575785f9d12200803e31923cc68d6c9b906876643688e3a7ccb21264f933028b060564e4d",
        "0x0000000000000000000000000000000000000000000000000000000000001c1e000000000000000000000000000000000000000000000000000000000000a354000036e661469dd70081ada16334d16a4049a124e261cd93def5fff88f85afda01b285fb7d6e0c7e05505a348777221f3c9fb491bbbbea4853e62e93f415efe7",
        "0x000000000000000000000000000000000000000000000000000000000000894100000000000000000000000000000000000000000000000000000000000002db104a10331d6a854148a10b11c19cf2abae0412c9909ecefca54adc135ee57a950481fe75941093272afb1f8f76353afad6b89b1c19c383b07730c6f160b59243"
      ],
      "value": "0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000017d7498da306d2911280c3481d8b1510e16062ffaa631812c3ca53639329c1577354f0e4cd850dde1255c33de5e8c499e72ca1f49352847124c0dbfc30d0374d4d5d5e7cddb83c7ac93c806e738300b5357ecdc2e971d6438d34d8e4e17b99b758b1f9cac91c8e700000000000000000000000000000000000000000000000000000000000005c89"
    }
  },
  "storageProofs": [
    {
      "key": "0x0000000000000000000000000000000000000000000000000000000000000000",
      "leftLeafIndex": 611,
      "leftProof": {
        "proofRelatedNodes": [
          "0x00000000000000000000000000000000000000000000000000000000000003a90c382f6158633dfaf5ea90b4b6aef05e0171d9c5e97a2f3aa41c3944e2d08f7c",
          "0x008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809",
          "0x060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b",
          "0x0a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef322550a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef32255",
          "0x01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d",
          "0x090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484",
          "0x11c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f30611c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f306",
          "0x07f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d4807f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d48",
          "0x0f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd80f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd8",
          "0x0cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e27390cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e2739",
          "0x014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b",
          "0x11c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca55411c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca554",
          "0x1092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c221092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c22",
          "0x0969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a890969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a89",
          "0x079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368",
          "0x004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458",
          "0x0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd",
          "0x0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c",
          "0x0defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea80defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea8",
          "0x0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b",
          "0x1276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f51276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f5",
          "0x02a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b202a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b2",
          "0x070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978",
          "0x0133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed10133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed1",
          "0x0b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f840b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f84",
          "0x0f3f9cf1e5ba6bdbb6daafc405bcceac97270fe89265b6a0faa2ba4bfd5cbf5d0f3f9cf1e5ba6bdbb6daafc405bcceac97270fe89265b6a0faa2ba4bfd5cbf5d",
          "0x08b60393196453ee74fdf240449d9aa2569875b43596ea2621eecda8d8909acd08b60393196453ee74fdf240449d9aa2569875b43596ea2621eecda8d8909acd",
          "0x10c439d656480d21a08c068717556fb8104a7a76e26f60e393ce4e36ae21e07b10c439d656480d21a08c068717556fb8104a7a76e26f60e393ce4e36ae21e07b",
          "0x09ea86c5cd59ac4bfca4e46e7b50bb37c8327350888ba71112ecf3f5093baaef09ea86c5cd59ac4bfca4e46e7b50bb37c8327350888ba71112ecf3f5093baaef",
          "0x0b971345bfa43e192ca2fb1c9ddd19f2dddf461243b1a54fdd5a4d581f850c110b971345bfa43e192ca2fb1c9ddd19f2dddf461243b1a54fdd5a4d581f850c11",
          "0x0edd0129edd35191a183ecd28cbcab2a48ad381215d8544acf35248639835dcd0edd0129edd35191a183ecd28cbcab2a48ad381215d8544acf35248639835dcd",
          "0x0f6516c2cee4cfd3c3453717d360474888ae7a5e3fbe9434c0434650b44c39200b019e3375b0a9280488dc8154335e83fcceb92a3bfb90fe1188c6a5c2723683",
          "0x0222fa2cc1728b6faff164e31b8b0c778f8b7c98046f625475e5d6ecd888e6e70cae760c7b8690d543c0558b3913c59e22748013099c0942e3960bb41d9078b8",
          "0x0fdfeacb22084128e246e1408b7975e3de40182d76d2e3b13e73a455231b6690010cdcdd77f54be0e2bb237ca3acbf30ce20fc53d03e1d377fe07a2657c6a452",
          "0x0eb38bc6c6d2dc3fc678880bad7bec0061cecaad838094521d352a0727944a5b049c14259c25252cb097d973dadae2e9645731f101d159115ce8dd6a6bc8d57c",
          "0x0e40da89d95f318c0e1f985a6554ed305c5f27f7cae7de3e07f41151e5a311f70ea08c7b543f2257955fa4e937c498ce21ce3f2a5bab5d4245638922c6cce06b",
          "0x04a3b8e7b06e29a06e335bf80be7a2908997f3294a9472be89cd95b1288e70c709cc26dc8c3b431c560c3847fde0ba114ac1bf58e3894738b7a35f72bf53a7b5",
          "0x005f762408388dc791d8064731ec0a4e6a256c69737f331b53f54d55308c87df0c7945adb1ac77e84bf94603a7e00de2dfc3c44d64d2ef5e5be63ae079fc15cc",
          "0x0254769f3f328564163e0be11c364aa2b4b651a975397c18c4608206a00998a00d3527a52738cd568ff1954312aeefda2bca64e4d91eb964fc3a0da5dd1c2b46",
          "0x0565ad1253bbaa5388ddb68dd52adb83a99c90a7a01f7037011c03727a2e1d3f0ba2a9a0599fad5ffb5fde06fc3368457d786ad2eabeb372cea6ed6481868094",
          "0x00000000000000000000000000000000000000000000000000000000000001ba00000000000000000000000000000000000000000000000000000000000002f1068da887b74fd30ff2e365193ddecf201afd9bd0181ffbc282939c943f0085d1120f168117038a271fcc94f9746eab5a6c682fa1efa41f23f6bacadedbc7b518",
          "0x00000000000000000000000000000000000000000000000000000000000000a100000000000000000000000000000000000000000000000000000000000001ff0226bb24dc7fb5f8356e291c3ca45555a828a0e02bcb822a2878eadc51a11f1f066ef290cc3e13cdddabed678d6e64d13941dd6a0c6ed789f5774a99b90921f2"
        ],
        "value": "0x0000000000000000000000000000000000000000000000000000375911dbcbbc"
      },
      "rightLeafIndex": 511,
      "rightProof": {
        "proofRelatedNodes": [
          "0x00000000000000000000000000000000000000000000000000000000000003a90c382f6158633dfaf5ea90b4b6aef05e0171d9c5e97a2f3aa41c3944e2d08f7c",
          "0x008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809",
          "0x060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b",
          "0x0a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef322550a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef32255",
          "0x01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d",
          "0x090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484",
          "0x11c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f30611c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f306",
          "0x07f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d4807f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d48",
          "0x0f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd80f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd8",
          "0x0cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e27390cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e2739",
          "0x014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b",
          "0x11c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca55411c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca554",
          "0x1092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c221092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c22",
          "0x0969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a890969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a89",
          "0x079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368",
          "0x004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458",
          "0x0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd",
          "0x0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c",
          "0x0defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea80defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea8",
          "0x0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b",
          "0x1276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f51276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f5",
          "0x02a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b202a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b2",
          "0x070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978",
          "0x0133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed10133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed1",
          "0x0b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f840b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f84",
          "0x0f3f9cf1e5ba6bdbb6daafc405bcceac97270fe89265b6a0faa2ba4bfd5cbf5d0f3f9cf1e5ba6bdbb6daafc405bcceac97270fe89265b6a0faa2ba4bfd5cbf5d",
          "0x08b60393196453ee74fdf240449d9aa2569875b43596ea2621eecda8d8909acd08b60393196453ee74fdf240449d9aa2569875b43596ea2621eecda8d8909acd",
          "0x10c439d656480d21a08c068717556fb8104a7a76e26f60e393ce4e36ae21e07b10c439d656480d21a08c068717556fb8104a7a76e26f60e393ce4e36ae21e07b",
          "0x09ea86c5cd59ac4bfca4e46e7b50bb37c8327350888ba71112ecf3f5093baaef09ea86c5cd59ac4bfca4e46e7b50bb37c8327350888ba71112ecf3f5093baaef",
          "0x0b971345bfa43e192ca2fb1c9ddd19f2dddf461243b1a54fdd5a4d581f850c110b971345bfa43e192ca2fb1c9ddd19f2dddf461243b1a54fdd5a4d581f850c11",
          "0x0edd0129edd35191a183ecd28cbcab2a48ad381215d8544acf35248639835dcd0edd0129edd35191a183ecd28cbcab2a48ad381215d8544acf35248639835dcd",
          "0x0d052b80abb809f9120c6b9884fffd52dd230a8dea0e503ee37a657412f956e4124085568263d79db22e8138cdfcddb82217762c26573f47a99464a1891998c0",
          "0x057a6e9128039b33095b2f3a29a0a4fd749c71bbdf024304b2979061ef696fda02609e7431e6a45621b7f5a2208d7b2fb036e64f1302666b157de4b082633069",
          "0x037b1a185403907b8636d653feaa7b9ad7a5d84a510bda3c0a12bc9472a4e5720c08e5c3f0f216918297ea9b1d23483c6c013f440f0068b25302c6d49fbf446a",
          "0x0d3421780fcce001fa7b4b27f5e39c220f05d0ffc491d8db6c6b5bcb4bcfb6a90298723fb96997c79f8bd6801f7d331234a09d6516ece5efba060a4ae6bcbf48",
          "0x0c276aa23e1ae6fdc99c3cc16f5ca012b50c616fb0684853e3bdad0379bc05fd0e65e9874dec3ac06a76fb1975a6758236c27541abb8806cf7e461e39dcae2b9",
          "0x0e7f129f0ed133acf079202b62b76e9c1f090c38369877039a41f389cfb28f04120861f2c2857a0ae1efbb7c5bbdf831a16e3648c3173bc35164ede64dfbf264",
          "0x00132e22083f4c5cd6faf3ed72a15db28e32d9ca043a0af318c4761b9fff9b8506f131ac26d2cba71ff6f3a4690468563bb280e426224b29627006c92b30a4a1",
          "0x07ef4499f02012217d7ba74661d161165ba8ce341eb8feedc6fb91c9d3daf6ea022140824283e42f43fc52f0fa23057ce4869a8c5ca5a263b08257011df91fbc",
          "0x10d1e689f780d25322ea063b70d009e4ecf8d2e24044e61849b526e5c92045780f386aa614b62d8f207b4aa98af1f28d4ed491ac4e371aa51aa2bb6bb7b9d2b9",
          "0x000000000000000000000000000000000000000000000000000000000000032c000000000000000000000000000000000000000000000000000000000000019d128c9ea07e20e3771e0d5c074ab0d250cf02502f2fcf253c3e627819053d8062063bfbb79af7fd4d7834a275cad3d9ccef2d5ade138040201e4b0533f5360ad6",
          "0x000000000000000000000000000000000000000000000000000000000000026300000000000000000000000000000000000000000000000000000000000001840233297165af3cab341e7e30b38dc8bf19d538e0ab6c6a842b2acbf536027b150e43b52047962596b0ecab1ed42e1774bd419bd21323899e4afd25bb6635bd52"
        ],
        "value": "0x000000000000000000000000000000000000000000000000000000000000007d"
      }
    }
  ]
}
        "#;

        let proof = serde_json::from_str::<GetProof>(raw).unwrap();
        let account = verify::<ZkAccount>(
            &new_mimc_constants_bls12_377(),
            &proof.account_proof,
            H256(hex!(
                "0C76548458CC04A5AA09BFFA092B32C912AEE635C1C44364EBB911286A10263D"
            )),
            H160(hex!("5ff137d4b0fdcd49dca30c7cf57e578a026d2789")),
        )
        .unwrap()
        .unwrap();
        let value = verify::<H256>(
            &new_mimc_constants_bls12_377(),
            &proof.storage_proofs[0],
            account.storage_root,
            H256::default(),
        )
        .unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_account_noninclusion() {
        let raw = r#"
{
  "accountProof": {
    "key": "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2780",
    "leftLeafIndex": 50915,
    "leftProof": {
      "proofRelatedNodes": [
        "0x00000000000000000000000000000000000000000000000000000000000120fe0393507c456718a986386c7923fe68b87c29d83ac7f7ce1cdb49afc7e66a4771",
        "0x008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809",
        "0x060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b",
        "0x0a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef322550a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef32255",
        "0x01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d",
        "0x090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484",
        "0x11c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f30611c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f306",
        "0x07f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d4807f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d48",
        "0x0f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd80f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd8",
        "0x0cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e27390cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e2739",
        "0x014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b",
        "0x11c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca55411c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca554",
        "0x1092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c221092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c22",
        "0x0969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a890969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a89",
        "0x079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368",
        "0x004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458",
        "0x0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd",
        "0x0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c",
        "0x0defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea80defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea8",
        "0x0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b",
        "0x1276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f51276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f5",
        "0x02a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b202a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b2",
        "0x070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978",
        "0x0133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed10133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed1",
        "0x09cbd26c486bc2217bce59337120283f655a7ba65075f98059249f471812d0480b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f84",
        "0x0c5c4d122720c4d6e7866d9b6bc6171c6259be90095976b665406ccf2dc6a8950305d7ebd7da4f82f061632eb7ec0c3060f51af848661d479bb64003f0fc5342",
        "0x0c4762f6af9f09a529e70f0b34b7afafe2bba8944eccdcdb95cf13e0ff00ab2209d8b650f132967dba1764abe34c3d446311503ba7712d5f474a6e159b085b5f",
        "0x125e0b74c9f944431f57100ababee80095eff47b54bcbeb59790ccab72af8a8d0ad6bd567eb864b9e5ce168bf3a7c87ee3963bc68577cb40d2c5118e69b02449",
        "0x0ced56af58b33a68e2b2491f0e45eb9aeccf95cdee3692dcc6d22055ffc07c67034599ba21f7c7c190d1a3df45d61faf51eba99d40671bafb8a9f0987c1379a0",
        "0x0cf419b794abebf2631eb9b71850e4720d5832926808fb04811b468cc34059e7023a96fe63cb2a1237ab59f91c368dba466d9670461b9f6177099aaaaab77479",
        "0x0651d5368092407b9dcd8f50f62c8e2d7a3ceac68c83bfb4c1c6a72ddf919519126f610f1b0dcc4004d4e591f227faff14211aea80ff74e65e197c8c276f3dd3",
        "0x0349cf762365c0272e1bbcdc42e189fb98f0bb81949c8f74af78d54734b393dd0898977dd60d5255f6d179d9c118a9715d9b8f179a76c92cfa4b50583086d174",
        "0x04ad89f67466c52ab58ce2b607f1b8444feec75c301f43f0c9a20b792059e48f02d966dd372e7021d97187b428f14de7d8f9b016c6082ed9c032f504b934de7c",
        "0x0402473cd6b64ef02e23ffde16a97698dd9f6e79c3922bb06169504cb13d259302e3e785b26b76f90b834588317d7efe81c21f8f6e127fc341e13f9173d2d33e",
        "0x0bfaf269c21ddbe687cf0a56120f9da606cff565c418ca081d9523cefa5ee4900d219e305070c0a5ffda3b2d20799b3876e4f899f21e790c5e61607e0a33cda1",
        "0x08eb97abb2df2a1affcd757cdd20fd0777e53470a38818e8febdae4236744efd0c3bee7aa429be1836b93f6871cff32c80888251602cb6782d7efbcfa96187f7",
        "0x0d147eadaed99af1df14632babd90117d3b6927af8f22902ffec8c7e0c81ac7310ee5b6b5b5b359bdb76271dcbf28e006f4c93c024175751fd3628dd1e6f1be9",
        "0x1130fa8251e74fc2eb80e836e988bdcd156d4bca6b3a8c118ed5e17ac2b926220883f050c04011d5437f4bf78454f01e24974a3acde10dab94ba8a7ee09c5cbd",
        "0x0043381a4fb2920cae78e46cd412a078e4905f2ae436b3233aa162b15b3a6f2c12252dfbb99f31c4713a63abff2141b357f20eef57cbdb1c7687c1d3f64878c6",
        "0x0a064a4646cb7f9fd9b3dcecb1fd8cbd1f2b859d97b56486ea002d285477346d03569d58aa002050c477fb037ca9564212f71476982ea550693529862d31579f",
        "0x0000000000000000000000000000000000000000000000000000000000005d44000000000000000000000000000000000000000000000000000000000000ce6212214e893b51fb4bff3f919f49f5e3d98f9e90a0744098760f05ea5c0ad37e0b02ada81e47c1223c7d15ba45be0b16ee69bd6a5e2c26de3f2facb725db2a2563",
        "0x00000000000000000000000000000000000000000000000000000000000088190000000000000000000000000000000000000000000000000000000000008cf50213dedcc29d8a1353bafac93de02055eb9fa839071f1a6257c8c1ed608ec01010a5194238ff342de60302f1918116fe22fd005173786d19a588d90a7bc82278"
      ],
      "value": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de0b6b3a764000007977874126658098c066972282d4c85f230520af3847e297fe7524f976873e50134373b65f439c874734ff51ea349327c140cde2e47a933146e6f9f2ad8eb17c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a4700000000000000000000000000000000000000000000000000000000000000000"
    },
    "rightLeafIndex": 36085,
    "rightProof": {
      "proofRelatedNodes": [
        "0x00000000000000000000000000000000000000000000000000000000000120fe0393507c456718a986386c7923fe68b87c29d83ac7f7ce1cdb49afc7e66a4771",
        "0x008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809008a47a2a53dd5183a2dc127c399a004e2a6c7e60f73e104d7d79e6a2bd7e809",
        "0x060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b060f08aed06ffb90efc9705dc38d37a7000da1add99cef1b8a84b9e72e7c8b7b",
        "0x0a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef322550a06dc31ae8e893bca0a076decb8c0caa9036b5f394abf79d7956411eef32255",
        "0x01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d01f35ef342eaa841ee4306d38f2a1adeafe8967d23c31fe1a379b9a69353da6d",
        "0x090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484090d53176fd185da729d0d68e0c0e646ef148f15864685f4ba56be7b7cbb2484",
        "0x11c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f30611c8e229e3e2ae40a4959e036d500753aaedb52cda67d9caf60f0629f0b4f306",
        "0x07f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d4807f048ac696418580a55a864a10ed030871fd615d5ab460c54d6184c16441d48",
        "0x0f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd80f5dc218160db17cfe8044d7ac4fd55dfcbdf2676815e2c15388f189bf144cd8",
        "0x0cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e27390cdf7d06a4b4b0e71713048f5f6ea86016467e909a27bfeeeca67b56c17e2739",
        "0x014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b014030b5cbe31660da2d33b6b1265b82bbde9a7ab7f331f8b274f2b798a45a3b",
        "0x11c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca55411c8aeb3dc3ca059a29ba20d4471b20987d74a0d79ff8ecda247df6a02eca554",
        "0x1092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c221092d1b2349c4fbc88ea0202cf88685e4e316c99697063f786201b27d46e2c22",
        "0x0969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a890969f4e85b86f0eb36ad13dfb1f35346d7d6518308dc27e73452c649850f1a89",
        "0x079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368079081f446c9a0c7b404834742cea1909426ccfc4696d19e1a08531b0cc30368",
        "0x004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458004d50e626bda007887a31f60883e58bce50a1a3e7a3384b9ec18dab319dd458",
        "0x0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd0b2ae68e3af633dac72090cc9c9b0dce76cebf5117101a265f54b3b9a851b3cd",
        "0x0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c0b7a8a9fe0ee619c9bd7ff504dcb47bdce0193546b53a79dedd5251f4f56f36c",
        "0x0defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea80defe934a1ae079cf6ec6022145b60128eeb30503eea4404da990fc2b2430ea8",
        "0x0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b0e42718d49cb8c4be515181eda51f41d3b8198af5a2139a4670a8ee06b904a2b",
        "0x1276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f51276c046afd611be02a66cf85498d7210a15293357afe07968a86c89356662f5",
        "0x02a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b202a9fd706c3c223f9374481b7495fb775c1675407556d93f1edabfe54b3fc9b2",
        "0x070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978070382f72e9f322433fb44fc4acfefd74b277b19b6cc1784379e7ca7338a2978",
        "0x0133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed10133209cd7936e208da6b743428ff7195e8ef92d3dac72472146ac7497355ed1",
        "0x09cbd26c486bc2217bce59337120283f655a7ba65075f98059249f471812d0480b03678742039acaae14fd3964e2d6261b74410043c536f07bcf1bc4495d9f84",
        "0x0c5c4d122720c4d6e7866d9b6bc6171c6259be90095976b665406ccf2dc6a8950305d7ebd7da4f82f061632eb7ec0c3060f51af848661d479bb64003f0fc5342",
        "0x0bd67fb37adfe69856936db64ba99e14abd6a73124332d622937c0e27697c6ce023c0ebe791b9fedd67c5d6bcb60b7e8f1b34a8bc3212626520e7ef92de490d2",
        "0x115b79d9b04cd131a492cba8892fe3768ffc4d7e6ba4883648bb735cdc98231a11db4ddd2d019276c2bf4b5a63b25d69e43ade93e229401fe636c8d2b0abfcd2",
        "0x036a0c38dc37c9dd30b3dd485a7a61e9c55cc6db0ab664122d93f566b40b642f0f71a7ca8aea7890f183d07c6bba517ac6132f7bb188f07ec32ca33cda9ddb9d",
        "0x126841c8020009187e5b49a59787a2a297227db1e11b5a78a1cace93cc4ce09c116ee798f5d5acf0b5df94de55ecb6ca1d638228746d2cdeed3c5c187fc3bd4e",
        "0x05d430417e93bbd414de5f2c529407b97eb843cc083d8105a2a6ec0868abc27004f133f13f993f98f15c3cf3e1eff101113834c763234bc8e1896ec3861063b2",
        "0x10530f986ec1a17af8fef6d2fbbb901accfd9d60828c6c0f6a6da1e926f948790f05a301b210f45406769f0ac2c45748da6dccce762d4adcc5dc85de4990f2c1",
        "0x0284a0134f5ac9cd8acfa1c79ca9f035f9728e5de39373e9ddca53625bd1ed280aa5ae2f615ab38828700102da3413de7a96d8878f5f097d09063fe6e7f0a12b",
        "0x0f7e46da1c787eb19b7788a82848f478e82bc2ac3575f90426d75c72f479c7ec1268704b94d6bd2a1a6d8ffbf27f433a36aca5a5fa661c6187e2e4b8cbc848a9",
        "0x06c9e4b29458dde1213ff22ed7452f39b8c70b7ef314c5a7949838837a48ea4808291f2d7f0133f66b8b23b06e8a5f6c4a292cea5bb4dcf4081c49cabd7d6c66",
        "0x10ab620750e513b9bc4a14270f8e7ea8e3e7d2187bbf37af175ae65a69b2b3a80584348d8fe95db0d10ec360e4d5e09f789f3021fc250e9e24977413fb500333",
        "0x0c185bc72eb982991d63be5c18a36298fccea38d8ad1363e54e01ae41ef6885909a48e907f9e6574b0d2f29a8e35ef2bccfca4316a03b666539b6f795b8ec08f",
        "0x0a42d55940760dbd55a4bf2e987f202b63291dc230115362fae168c00027b6ec0b838ec78059b519c8d17d9b4748fef69886684d209d3583e29261e6f9d7a7b7",
        "0x0f909e7857d476f194af529bcc51ef515960148352cae9b4e2a4292bd207bcf01218b51c9c9be2c5f66829c972eeae0d473d16770955718d50add1ed7ac1d4f2",
        "0x127c1c643468d9bc6fed375cfce594c169c5855b2bc19bd70818ee4d1e4ac6d812831b628352e7c7064f6d3440f9e2d03fa3336702edba27aea748ddff0478f9",
        "0x0000000000000000000000000000000000000000000000000000000000008de60000000000000000000000000000000000000000000000000000000000004ff211d6fa740eb934a32b9e49dd7f2f655eb5c8a8014b36ad55b0cd66bae1b7b3a810a5194238ff342de60302f1918116fe22fd005173786d19a588d90a7bc82278",
        "0x000000000000000000000000000000000000000000000000000000000000c6e3000000000000000000000000000000000000000000000000000000000000cf420213ef7ebf37eca24c05838ef0597f9aac9e9ba9dd49e75012ab573c8b8c3f3910a5194238ff342de60302f1918116fe22fd005173786d19a588d90a7bc82278"
      ],
      "value": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de0b6b3a764000007977874126658098c066972282d4c85f230520af3847e297fe7524f976873e50134373b65f439c874734ff51ea349327c140cde2e47a933146e6f9f2ad8eb17c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a4700000000000000000000000000000000000000000000000000000000000000000"
    }
  },
  "storageProofs": []
}
        "#;

        let proof = serde_json::from_str::<GetProof>(raw).unwrap();
        let account = verify::<ZkAccount>(
            &new_mimc_constants_bls12_377(),
            &proof.account_proof,
            H256(hex!(
                "0C76548458CC04A5AA09BFFA092B32C912AEE635C1C44364EBB911286A10263D"
            )),
            H160(hex!("5ff137d4b0fdcd49dca30c7cf57e578a026d2780")),
        )
        .unwrap();
        assert_eq!(account, None);
    }

    // https://github.com/Consensys/shomei/blob/955b4d8100f1a12702cdefc3fa79b16dd1c038e6/core/src/test/java/net/consensys/shomei/ZkAccountTest.java#L34
    #[test]
    fn test_hash_zero_account() {
        let mimc_constants = new_mimc_constants_bls12_377();
        let mimc = new_mimc_bls12_377(&mimc_constants);
        let hash = mimc
            .update(&ZkAccount::default().into_bytes())
            .unwrap()
            .finalize();
        assert_eq!(
            hash,
            hex!("0f170eaef9275fd6098a06790c63a141e206e0520738a4cf5cf5081d495e8682")
        );
    }

    #[test]
    fn test_hashed_key() {
        let mimc_constants = new_mimc_constants_bls12_377();
        let mimc = new_mimc_bls12_377(&mimc_constants);
        let hash = mimc
            .update(hex!(
                "0000000000000000000000005ff137d4b0fdcd49dca30c7cf57e578a026d2789"
            ))
            .unwrap()
            .finalize();
        assert_eq!(
            H256(hex!(
                "104a10331d6a854148a10b11c19cf2abae0412c9909ecefca54adc135ee57a95"
            )),
            hash.try_into().unwrap(),
        );
    }
}

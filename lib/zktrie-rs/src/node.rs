use core::marker::PhantomData;

use crate::{
    handling_elems_and_byte32, hash_elems_with_domain, Byte32, Error, Fr, Hash, HashScheme,
    HASH_BYTE_LEN,
};

#[derive(Debug, Eq)]
pub struct Node<H: HashScheme> {
    value: NodeValue,
    hash: Hash,
    _phantom: PhantomData<H>,
}

impl<H: HashScheme> Clone for Node<H> {
    fn clone(&self) -> Self {
        Node {
            value: self.value.clone(),
            hash: self.hash,
            _phantom: self._phantom,
        }
    }
}

impl<H: HashScheme> PartialEq for Node<H> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value) && self.hash.eq(&other.hash)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeValue {
    Empty,
    Leaf(LeafNode),
    Branch(BranchNode),
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BranchType {
    BothTerminal = 6u8,
    LeftTerminal = 7u8,
    RightTerminal = 8u8,
    BothBranch = 9u8,
}

impl BranchType {
    #[must_use]
    pub fn left(&self, hash: Hash) -> BranchHash {
        match self {
            Self::BothTerminal | Self::LeftTerminal => BranchHash::Terminal(hash),
            Self::BothBranch | Self::RightTerminal => BranchHash::Branch(hash),
        }
    }

    #[must_use]
    pub fn right(&self, hash: Hash) -> BranchHash {
        match self {
            Self::BothTerminal | Self::RightTerminal => BranchHash::Terminal(hash),
            Self::BothBranch | Self::LeftTerminal => BranchHash::Branch(hash),
        }
    }

    #[must_use]
    pub fn deduce_upgrade(&self, go_right: bool) -> Self {
        if go_right {
            match self {
                Self::BothTerminal => Self::LeftTerminal,
                Self::LeftTerminal => *self,
                Self::RightTerminal | Self::BothBranch => Self::BothBranch,
            }
        } else {
            match self {
                Self::BothTerminal => Self::RightTerminal,
                Self::LeftTerminal | Self::BothBranch => Self::BothBranch,
                Self::RightTerminal => *self,
            }
        }
    }

    #[must_use]
    pub fn deduce_downgrade(&self, at_right: bool) -> Self {
        if at_right {
            match &self {
                Self::LeftTerminal => Self::BothTerminal,
                Self::BothBranch => Self::RightTerminal,
                Self::BothTerminal | Self::RightTerminal => {
                    panic!("can not downgrade a node with terminal child ({self:?})")
                }
            }
        } else {
            match &self {
                Self::BothBranch => Self::LeftTerminal,
                Self::RightTerminal => Self::BothTerminal,
                Self::BothTerminal | Self::LeftTerminal => {
                    panic!("can not downgrade a node with terminal child ({self:?})")
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BranchHash {
    Terminal(Hash),
    Branch(Hash),
}

impl Default for BranchHash {
    fn default() -> Self {
        BranchHash::Terminal(Hash::default())
    }
}

impl BranchHash {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn fr(&self) -> Result<Fr, Error> {
        self.hash().fr().map_err(Error::NotInField)
    }

    #[must_use]
    pub fn hash(&self) -> &Hash {
        match self {
            BranchHash::Branch(n) => n,
            BranchHash::Terminal(n) => n,
        }
    }
}

impl<H: HashScheme> Node<H> {
    pub fn from_bytes(mut b: &[u8]) -> Result<Self, Error> {
        if b.is_empty() {
            return Err(Error::NodeBytesBadSize);
        }
        let ty = b[0];
        b = &b[1..];
        match ty {
            0 | 6 | 7 | 8 | 9 => {
                // branch
                if b.len() != 2 * HASH_BYTE_LEN {
                    return Err(Error::NodeBytesBadSize);
                }
                let left = {
                    let hash = Hash::from_bytes(&b[..HASH_BYTE_LEN]);
                    if matches!(ty, 6 | 7) {
                        BranchHash::Terminal(hash)
                    } else {
                        BranchHash::Branch(hash)
                    }
                };
                let right = {
                    let hash = Hash::from_bytes(&b[HASH_BYTE_LEN..HASH_BYTE_LEN * 2]);
                    if matches!(ty, 6 | 8) {
                        BranchHash::Terminal(hash)
                    } else {
                        BranchHash::Branch(hash)
                    }
                };
                Self::new_branch(left, right)
            }
            1 | 4 => {
                // leaf
                if b.len() < HASH_BYTE_LEN + 4 {
                    return Err(Error::NodeBytesBadSize);
                }
                let node_key = Hash::from_bytes(&b[..HASH_BYTE_LEN]);
                let mark = {
                    let mut buf = [0_u8; 4];
                    buf.copy_from_slice(&b[HASH_BYTE_LEN..HASH_BYTE_LEN + 4]);
                    u32::from_le_bytes(buf)
                };
                let preimage_len = (mark & 255) as usize;
                let compressed_flags = mark >> 8;
                let mut value_preimage = Vec::with_capacity(preimage_len);
                let mut cur_pos = HASH_BYTE_LEN + 4;
                if b.len() < cur_pos + preimage_len * 32 + 1 {
                    return Err(Error::NodeBytesBadSize);
                }
                for i in 0..preimage_len {
                    let val = {
                        let mut b: &[u8] = &b[i * 32 + cur_pos..(i + 1) * 32 + cur_pos];
                        let mut bytes = [0_u8; 32];
                        if b.len() > bytes.len() {
                            b = &b[..bytes.len()];
                        }
                        let dst = if b.len() > bytes.len() {
                            &mut bytes[..]
                        } else {
                            &mut bytes[..b.len()]
                        };
                        dst.copy_from_slice(b);
                        bytes.into()
                    };
                    value_preimage.push(val);
                }
                cur_pos += preimage_len * 32;
                let preimage_size = b[cur_pos] as usize;
                cur_pos += 1;
                let key_preimage = if preimage_size != 0 {
                    if b.len() < cur_pos + preimage_size {
                        return Err(Error::NodeBytesBadSize);
                    }
                    Some(Byte32::from_bytes(&b[cur_pos..cur_pos + preimage_size]))
                } else {
                    None
                };
                Self::new_leaf(node_key, compressed_flags, value_preimage, key_preimage)
            }
            2 | 5 => Ok(Node::new_empty()),
            ty => Err(Error::InvalidNodeFound(ty)),
        }
    }

    pub fn new_branch_ty(ty: BranchType, left: Hash, right: Hash) -> Result<Self, Error> {
        let left = ty.left(left);
        let right = ty.right(right);
        Self::new_branch(left, right)
    }

    pub fn new_branch(left: BranchHash, right: BranchHash) -> Result<Self, Error> {
        let value = BranchNode::new(left, right);
        Ok(Self {
            hash: value.hash::<H>()?,
            value: NodeValue::Branch(value),
            _phantom: PhantomData,
        })
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::new_empty()
    }

    #[must_use]
    pub fn new_empty() -> Self {
        let value = NodeValue::Empty;
        Self {
            value,
            hash: Hash::default(),
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn branch(&self) -> Option<&BranchNode> {
        match &self.value {
            NodeValue::Branch(node) => Some(node),
            _ => None,
        }
    }

    #[must_use]
    pub fn leaf(&self) -> Option<&LeafNode> {
        match &self.value {
            NodeValue::Leaf(node) => Some(node),
            _ => None,
        }
    }

    #[must_use]
    pub fn match_leaf_key(&self, k: &Hash) -> bool {
        match self.leaf() {
            Some(leaf) => &leaf.key == k,
            None => false,
        }
    }

    pub fn new_leaf(
        key: Hash,
        value_flag: u32,
        value_preimage: Vec<Byte32>,
        key_preimage: Option<Byte32>,
    ) -> Result<Self, Error> {
        let node = LeafNode {
            key,
            compressed_flags: value_flag,
            value_preimage,
            key_preimage,
        };
        Ok(Self {
            hash: node.hash::<H>()?,
            value: NodeValue::Leaf(node),
            _phantom: PhantomData,
        })
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(&self.value, NodeValue::Empty)
    }

    #[must_use]
    pub fn is_branch(&self) -> bool {
        matches!(&self.value, NodeValue::Branch(_))
    }

    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(&self.value, NodeValue::Leaf(_))
    }

    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self.value, NodeValue::Empty | NodeValue::Leaf(_))
    }

    #[must_use]
    pub fn value(&self) -> &NodeValue {
        &self.value
    }

    #[must_use]
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    #[must_use]
    pub fn data(&self) -> &[u8] {
        match &self.value {
            NodeValue::Leaf(leaf) => leaf.data(),
            _ => &[],
        }
    }

    pub fn leaf_set_keypreimage(self, preimage: Option<Byte32>) -> Result<Self, Error> {
        match self.value {
            NodeValue::Leaf(leaf) => Self::new_leaf(
                leaf.key,
                leaf.compressed_flags,
                leaf.value_preimage,
                preimage,
            ),
            _ => Err(Error::ExpectedLeafNode),
        }
    }

    #[must_use]
    pub fn canonical_value(&self) -> Vec<u8> {
        match &self.value {
            NodeValue::Empty => vec![5],
            NodeValue::Leaf(node) => node.canonical_value(),
            NodeValue::Branch(node) => node.canonical_value(),
        }
    }

    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        node_bytes(self, self.leaf().and_then(|n| n.key_preimage))
    }
}

#[must_use]
pub fn node_bytes<H: HashScheme>(n: &Node<H>, key_preimage: Option<Byte32>) -> Vec<u8> {
    let mut data = n.canonical_value();
    if let Some(key) = &key_preimage {
        assert!(!data.is_empty());
        *data.last_mut().unwrap() = u8::try_from(key.len()).unwrap();
        data.extend_from_slice(key.bytes());
    }
    data
}

pub struct EmptyNode {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LeafNode {
    pub key: Hash,
    pub compressed_flags: u32,
    pub value_preimage: Vec<Byte32>,
    pub key_preimage: Option<Byte32>,
}

impl LeafNode {
    pub fn hash<H: HashScheme>(&self) -> Result<Hash, Error> {
        let value_hash = self.value_hash::<H>()?;
        leaf_hash::<H>(&self.key, &value_hash).map_err(Error::NotInField)
    }

    pub fn value_hash<H: HashScheme>(&self) -> Result<Hash, Error> {
        handling_elems_and_byte32::<H>(self.compressed_flags, &self.value_preimage)
            .map_err(Error::NotInField)
    }

    #[must_use]
    pub fn data(&self) -> &[u8] {
        let ptr = self.value_preimage.as_ptr().cast::<u8>();
        unsafe { std::slice::from_raw_parts(ptr, self.value_preimage.len() * 32) }
    }

    #[must_use]
    pub fn ty() -> u8 {
        4
    }

    #[must_use]
    pub fn canonical_value(&self) -> Vec<u8> {
        let size = 1
            + 32
            + 4
            + self.value_preimage.len() * 32
            + 1
            + self.key_preimage.as_ref().map(|_| 32).unwrap_or_default();
        let mut val = Vec::with_capacity(size);
        val.push(Self::ty());
        val.extend_from_slice(&self.key.bytes());

        let compressed_flag =
            (self.compressed_flags << 8) + u32::try_from(self.value_preimage.len()).unwrap();
        val.extend_from_slice(&compressed_flag.to_le_bytes());
        for elm in &self.value_preimage {
            val.extend_from_slice(elm.bytes());
        }
        val.push(0); // length of key preimage
        val
    }
}

pub fn leaf_hash<H: HashScheme>(k: &Hash, v: &Hash) -> Result<Hash, String> {
    let domain = (LeafNode::ty() as usize).into();

    Ok(hash_elems_with_domain::<H>(
        &domain,
        &k.fr()?,
        &v.fr()?,
        &[],
    ))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BranchNode {
    pub left: BranchHash,
    pub right: BranchHash,
}

impl BranchNode {
    #[must_use]
    pub fn new(left: BranchHash, right: BranchHash) -> Self {
        BranchNode { left, right }
    }

    #[must_use]
    pub fn ty(&self) -> BranchType {
        match (&self.left, &self.right) {
            (BranchHash::Terminal(_), BranchHash::Terminal(_)) => BranchType::BothTerminal,
            (BranchHash::Terminal(_), BranchHash::Branch(_)) => BranchType::LeftTerminal,
            (BranchHash::Branch(_), BranchHash::Terminal(_)) => BranchType::RightTerminal,
            (BranchHash::Branch(_), BranchHash::Branch(_)) => BranchType::BothBranch,
        }
    }

    #[must_use]
    pub fn canonical_value(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 * 2 + 1);
        out.push(self.ty() as u8);
        out.extend_from_slice(&self.left.hash().bytes());
        out.extend_from_slice(&self.right.hash().bytes());
        out
    }

    pub fn hash<H: HashScheme>(&self) -> Result<Hash, Error> {
        let domain = (self.ty() as u64).into();
        let hash = hash_elems_with_domain::<H>(&domain, &self.left.fr()?, &self.right.fr()?, &[]);
        Ok(hash)
    }
}

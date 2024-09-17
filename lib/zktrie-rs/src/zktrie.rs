use core::marker::PhantomData;

use poseidon_rs::Fr;

use crate::{
    test_bit, to_secure_key, BranchHash, BranchType, Byte32, Database, Error, Hash, HashScheme,
    Node, NodeValue, PreimageDatabase, MAGIC_SMT_BYTES, ZERO_HASH,
};

#[derive(Clone)]
pub struct ZkTrie<H: HashScheme> {
    root: Hash,
    max_level: usize,
    phantom: PhantomData<H>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrieData<H: HashScheme> {
    NotFound,
    Node(Node<H>),
}

impl<H: HashScheme> TrieData<H> {
    #[must_use]
    pub fn get(&self) -> &[u8] {
        match self {
            Self::Node(node) => node.data(),
            Self::NotFound => &[],
        }
    }
}

impl<H: HashScheme> ZkTrie<H> {
    #[must_use]
    pub fn new(max_level: usize, root: Hash) -> Self {
        Self {
            root,
            max_level,
            phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn hash(&self) -> &Hash {
        &self.root
    }

    pub fn get_data<D>(&self, db: &mut D, key: &[u8]) -> Result<TrieData<H>, Error>
    where
        D: Database<Node = Node<H>>,
    {
        let k = to_secure_key::<H>(key)?;
        let node_key: Hash = k.into();
        match self.try_get_node(db, &node_key) {
            Ok(node) => Ok(TrieData::Node(node)),
            Err(Error::KeyNotFound) => Ok(TrieData::NotFound),
            Err(err) => Err(err),
        }
    }

    pub fn try_get_node<D>(&self, db: &mut D, node_key: &Hash) -> Result<Node<H>, Error>
    where
        D: Database<Node = Node<H>>,
    {
        let path = get_path(self.max_level, node_key.raw_bytes());
        let mut next_hash = self.root;
        for (level, direction) in path.iter().take(self.max_level).enumerate() {
            let n = self
                .get_node(db, &next_hash)?
                .ok_or(Error::NodeNotFound((level, next_hash)))?;
            match n.value() {
                NodeValue::Empty => return Err(Error::KeyNotFound),
                NodeValue::Leaf(leaf) => {
                    if node_key == &leaf.key {
                        return Ok(n);
                    }
                    return Err(Error::KeyNotFound);
                }
                NodeValue::Branch(branch) => {
                    if *direction {
                        next_hash = *branch.right.hash();
                    } else {
                        next_hash = *branch.left.hash();
                    }
                }
            }
        }
        Err(Error::ReachedMaxLevel)
    }

    pub fn delete<D>(&mut self, db: &mut D, key: &[u8]) -> Result<(), Error>
    where
        D: Database<Node = Node<H>>,
    {
        let k = to_secure_key::<H>(key)?;
        let key_hash = k.into();

        // mitigate the create-delete issue: do not delete non-existent key
        match self.try_get_node(db, &key_hash) {
            Ok(_) => {}
            Err(Error::KeyNotFound) => return Ok(()),
            Err(err) => return Err(err),
        }

        self.try_delete(db, &key_hash)
    }

    pub(crate) fn try_delete<D>(&mut self, db: &mut D, node_key: &Hash) -> Result<(), Error>
    where
        D: Database<Node = Node<H>>,
    {
        let path = get_path(self.max_level, node_key.raw_bytes());
        let mut next_hash = self.root;
        let mut siblings = Vec::new();
        for i in 0..self.max_level {
            let Some(n) = self.get_node(db, &next_hash)? else {
                return Err(Error::KeyNotFound);
            };

            match n.value() {
                NodeValue::Empty => return Err(Error::KeyNotFound),
                NodeValue::Leaf(leaf) => {
                    if &leaf.key == node_key {
                        // remove and go up with sibling
                        self.rm_and_upload(db, &path, node_key, &siblings)?;
                        return Ok(());
                    }
                    return Err(Error::KeyNotFound);
                }
                NodeValue::Branch(branch) => {
                    if path[i] {
                        next_hash = *branch.right.hash();
                        siblings.push((branch.ty(), *branch.left.hash()));
                    } else {
                        next_hash = *branch.left.hash();
                        siblings.push((branch.ty(), *branch.right.hash()));
                    }
                }
            }
        }
        Err(Error::KeyNotFound)
    }

    fn rm_and_upload<D>(
        &mut self,
        db: &mut D,
        path: &[bool],
        _key: &Hash,
        siblings: &[(BranchType, Hash)],
    ) -> Result<Hash, Error>
    where
        D: Database<Node = Node<H>>,
    {
        if siblings.is_empty() {
            self.root = ZERO_HASH;
            return Ok(self.root);
        }

        if siblings.last().unwrap().0 != BranchType::BothTerminal {
            let nn = Node::new_empty();
            self.root = self.recalculate_path_until_root(db, path, nn, siblings)?;
            return Ok(self.root);
        }

        if siblings.len() == 1 {
            self.root = siblings[0].1;
            return Ok(self.root);
        }

        let (_, to_upload) = &siblings[siblings.len() - 1];

        for i in (0..siblings.len() - 1).rev() {
            if siblings[i].1 == Hash::default() {
                continue;
            }
            let new_node_type = siblings[i].0.deduce_downgrade(path[i]);
            let new_node = if path[i] {
                <Node<H>>::new_branch_ty(new_node_type, siblings[i].1, *to_upload)?
            } else {
                <Node<H>>::new_branch_ty(new_node_type, *to_upload, siblings[i].1)?
            };
            match self.add_node(db, &new_node) {
                Err(Error::NodeKeyAlreadyExists) | Ok(_) => {}
                Err(err) => return Err(err),
            }
            self.root = self.recalculate_path_until_root(db, path, new_node, &siblings[..i])?;
            return Ok(self.root);
        }

        self.root = *to_upload;

        Ok(self.root)
    }

    pub fn update<D>(
        &mut self,
        db: &mut D,
        key: &[u8],
        v_flag: u32,
        v_preimage: Vec<Byte32>,
    ) -> Result<(), Error>
    where
        D: PreimageDatabase<Node = Node<H>>,
    {
        let k = to_secure_key::<H>(key)?;
        Self::update_preimage(db, key, &k);
        let key_hash = k.into();
        self.try_update(db, &key_hash, v_flag, v_preimage)?;
        Ok(())
    }

    fn update_preimage<D: PreimageDatabase>(db: &mut D, preimage: &[u8], hash_field: &Fr) {
        db.update_preimage(preimage, hash_field);
    }

    pub(crate) fn try_update<D>(
        &mut self,
        db: &mut D,
        key: &Hash,
        v_flag: u32,
        v_preimage: Vec<Byte32>,
    ) -> Result<(), Error>
    where
        D: Database<Node = Node<H>>,
    {
        let new_leaf_node = <Node<H>>::new_leaf(*key, v_flag, v_preimage, None)?;
        let path = get_path(self.max_level, key.raw_bytes());

        let root = self.root;
        let new_root_result = self.add_leaf(db, new_leaf_node, &root, 0, &path, true);
        let new_root = match new_root_result {
            Err(Error::EntryIndexAlreadyExists) => {
                panic!("Encounter unexpected errortype: ErrEntryIndexAlreadyExists")
            }
            Err(err) => return Err(err),
            Ok(new_root) => new_root,
        };
        self.root = *(new_root.hash());
        Ok(())
    }

    // GetNode gets a node by node hash from the MT.  Empty nodes are not stored in the
    // tree; they are all the same and assumed to always exist.
    // <del>for non exist key, return (NewEmptyNode(), nil)</del>
    pub(crate) fn get_node<D>(&self, db: &mut D, hash: &Hash) -> Result<Option<Node<H>>, Error>
    where
        D: Database<Node = Node<H>>,
    {
        if hash.is_zero() {
            return Ok(Some(<Node<H>>::empty()));
        }
        db.get_node(hash)
    }

    fn recalculate_path_until_root<D>(
        &mut self,
        db: &mut D,
        path: &[bool],
        mut node: Node<H>,
        siblings: &[(BranchType, Hash)],
    ) -> Result<Hash, Error>
    where
        D: Database<Node = Node<H>>,
    {
        for i in (0..siblings.len()).rev() {
            let node_hash = *node.hash();
            node = if path[i] {
                <Node<H>>::new_branch_ty(siblings[i].0, siblings[i].1, node_hash)?
            } else {
                <Node<H>>::new_branch_ty(siblings[i].0, node_hash, siblings[i].1)?
            };
            match self.add_node(db, &node) {
                Err(Error::NodeKeyAlreadyExists) | Ok(_) => {}
                Err(err) => return Err(err),
            }
        }
        return Ok(*node.hash());
    }

    // addLeaf recursively adds a newLeaf in the MT while updating the path, and returns the node hash
    // of the new added leaf.
    pub fn add_leaf<D>(
        &mut self,
        db: &mut D,
        new_leaf: Node<H>,
        curr_node_hash: &Hash,
        lvl: usize,
        path: &[bool],
        force_update: bool,
    ) -> Result<BranchHash, Error>
    where
        D: Database<Node = Node<H>>,
    {
        if lvl > self.max_level - 1 {
            return Err(Error::ReachedMaxLevel);
        }

        let Some(n) = self.get_node(db, curr_node_hash)? else {
            return Err(Error::NodeNotFound((lvl, *curr_node_hash)));
        };

        match n.value() {
            NodeValue::Empty => {
                let nn = self.add_node(db, &new_leaf)?;
                Ok(BranchHash::Terminal(nn))
            }
            NodeValue::Leaf(old_leaf) => {
                let new_leaf_value = new_leaf.leaf().unwrap();
                if old_leaf.key == new_leaf_value.key {
                    if new_leaf.hash() == n.hash() {
                        return Ok(BranchHash::Terminal(*n.hash()));
                    } else if force_update {
                        let hash = self.update_node(db, new_leaf)?;
                        return Ok(BranchHash::Terminal(hash));
                    }
                    return Err(Error::EntryIndexAlreadyExists);
                }
                let path_old_leaf = get_path(self.max_level, old_leaf.key.raw_bytes());
                let hash = self.push_leaf(db, new_leaf, &n, lvl, path, &path_old_leaf)?;
                Ok(BranchHash::Branch(hash))
            }
            NodeValue::Branch(branch) => {
                let new_parent_node = if path[lvl] {
                    // go right
                    let new_node_hash = self.add_leaf(
                        db,
                        new_leaf,
                        branch.right.hash(),
                        lvl + 1,
                        path,
                        force_update,
                    )?;
                    <Node<H>>::new_branch(branch.left.clone(), new_node_hash)?
                } else {
                    // go left
                    let new_node_hash = self.add_leaf(
                        db,
                        new_leaf,
                        branch.left.hash(),
                        lvl + 1,
                        path,
                        force_update,
                    )?;
                    <Node<H>>::new_branch(new_node_hash, branch.right.clone())?
                };
                let hash = self.add_node(db, &new_parent_node)?;
                Ok(BranchHash::Branch(hash))
            }
        }
    }

    // pushLeaf recursively pushes an existing oldLeaf down until its path diverges
    // from newLeaf, at which point both leafs are stored, all while updating the
    // path. pushLeaf returns the node hash of the parent of the oldLeaf and newLeaf
    pub fn push_leaf<D>(
        &mut self,
        db: &mut D,
        new_leaf: Node<H>,
        old_leaf: &Node<H>,
        lvl: usize,
        path_new_leaf: &[bool],
        path_old_leaf: &[bool],
    ) -> Result<Hash, Error>
    where
        D: Database<Node = Node<H>>,
    {
        if lvl > self.max_level - 2 {
            return Err(Error::ReachedMaxLevel);
        }
        if path_new_leaf[lvl] == path_old_leaf[lvl] {
            let next_node_hash = self.push_leaf(
                db,
                new_leaf,
                old_leaf,
                lvl + 1,
                path_new_leaf,
                path_old_leaf,
            )?;
            let new_parent_node = if path_new_leaf[lvl] {
                // go right
                <Node<H>>::new_branch(BranchHash::empty(), BranchHash::Branch(next_node_hash))?
            } else {
                // go left
                <Node<H>>::new_branch(BranchHash::Branch(next_node_hash), BranchHash::empty())?
            };
            return self.add_node(db, &new_parent_node);
        }
        let new_parent_node = if path_new_leaf[lvl] {
            <Node<H>>::new_branch(
                BranchHash::Terminal(*old_leaf.hash()),
                BranchHash::Terminal(*new_leaf.hash()),
            )?
        } else {
            <Node<H>>::new_branch(
                BranchHash::Terminal(*new_leaf.hash()),
                BranchHash::Terminal(*old_leaf.hash()),
            )?
        };
        self.add_node(db, &new_leaf)?;
        let new_parent_hash = self.add_node(db, &new_parent_node)?;
        Ok(new_parent_hash)
    }

    // addNode adds a node into the MT and returns the node hash. Empty nodes are
    // not stored in the tree since they are all the same and assumed to always exist.
    pub fn add_node<D>(&mut self, db: &mut D, n: &Node<H>) -> Result<Hash, Error>
    where
        D: Database<Node = Node<H>>,
    {
        let hash = n.hash();
        if n.is_empty() {
            return Ok(*hash);
        }

        if let Some(old) = db.get_node(hash)? {
            if &old != n {
                return Err(Error::NodeKeyAlreadyExists);
            }
            Ok(*hash)
        } else {
            let n = db.update_node(n.clone())?;
            Ok(*n.hash())
        }
    }

    // updateNode updates an existing node in the MT.  Empty nodes are not stored
    // in the tree; they are all the same and assumed to always exist.
    pub fn update_node<D>(&mut self, db: &mut D, n: Node<H>) -> Result<Hash, Error>
    where
        D: Database<Node = Node<H>>,
    {
        let hash = n.hash();
        if n.is_empty() {
            return Ok(*hash);
        }
        let n = db.update_node(n)?;
        Ok(*n.hash())
    }

    pub fn walk<F, D>(
        &self,
        db: &mut D,
        key_hash: &Hash,
        mut from_level: usize,
        mut write_node: F,
    ) -> Result<(), Error>
    where
        D: Database<Node = Node<H>>,
        F: FnMut(&mut D, Node<H>) -> Result<(), Error>,
    {
        let path = get_path(self.max_level, key_hash.raw_bytes());
        let mut nodes = Vec::new();
        let mut tn = self.root;
        for direction in path.iter().take(self.max_level) {
            let n = match self.get_node(db, &tn) {
                Ok(Some(n)) => n,
                Ok(None) => return Err(Error::KeyNotFound),
                Err(err) => return Err(err),
            };
            let mut finished = true;
            match n.value() {
                NodeValue::Leaf(_) | NodeValue::Empty => {}
                NodeValue::Branch(branch) => {
                    finished = false;
                    tn = if *direction {
                        *branch.right.hash()
                    } else {
                        *branch.left.hash()
                    };
                }
            }
            nodes.push(n);
            if finished {
                break;
            }
        }

        for n in nodes {
            if from_level > 0 {
                from_level -= 1;
                continue;
            }
            write_node(db, n)?;
        }
        Ok(())
    }

    pub fn proof<D>(&self, db: &mut D, key: &[u8]) -> Result<Vec<Vec<u8>>, Error>
    where
        D: Database<Node = Node<H>>,
    {
        let k = to_secure_key::<H>(key)?;
        let node_key: Hash = k.into();
        let mut proof = Vec::new();
        self.prove(db, &node_key.bytes(), 0, |_, node| {
            proof.push(node.bytes());
            Ok(())
        })?;
        proof.push(MAGIC_SMT_BYTES.to_vec());
        Ok(proof)
    }

    // Prove is a simplified calling of ProveWithDeletion
    pub fn prove<D, F>(
        &self,
        db: &mut D,
        key: &[u8],
        from_level: usize,
        write_node: F,
    ) -> Result<(), Error>
    where
        D: Database<Node = Node<H>>,
        F: FnMut(&mut D, Node<H>) -> Result<(), Error>,
    {
        type N<H> = fn(Node<H>, Option<Node<H>>);
        self.prove_with_deletion::<D, F, N<H>>(db, key, from_level, write_node, None)
    }

    // ProveWithDeletion constructs a merkle proof for key. The result contains all encoded nodes
    // on the path to the value at key. The value itself is also included in the last
    // node and can be retrieved by verifying the proof.
    //
    // If the trie does not contain a value for key, the returned proof contains all
    // nodes of the longest existing prefix of the key (at least the root node), ending
    // with the node that proves the absence of the key.
    //
    // If the trie contain value for key, the onHit is called BEFORE writeNode being called,
    // both the hit leaf node and its sibling node is provided as arguments so caller
    // would receive enough information for launch a deletion and calculate the new root
    // base on the proof data
    // Also notice the sibling can be nil if the trie has only one leaf
    pub fn prove_with_deletion<D, F, N>(
        &self,
        db: &mut D,
        key: &[u8],
        from_level: usize,
        mut write_node: F,
        mut on_hit: Option<N>,
    ) -> Result<(), Error>
    where
        D: Database<Node = Node<H>>,
        F: FnMut(&mut D, Node<H>) -> Result<(), Error>,
        N: Fn(Node<H>, Option<Node<H>>),
    {
        if key.len() != 32 {
            return Err(Error::InvalidField);
        }

        let k = Hash::from_bytes(key);

        let mut prev: Option<Node<H>> = None;
        self.walk(db, &k, from_level, |db, node| {
            let prev_branch_node = prev
                .as_ref()
                .map(|n| n.branch().expect("unexpected behavior in prove"));

            let Some(on_hit) = &mut on_hit else {
                prev = Some(node.clone());
                return write_node(db, node);
            };

            if node.match_leaf_key(&k) {
                match prev_branch_node {
                    Some(prev) => {
                        let node_hash = node.hash();
                        let sibling = if node_hash == prev.left.hash() {
                            prev.right.hash()
                        } else {
                            prev.left.hash()
                        };
                        match self.get_node(db, sibling) {
                            Ok(sibling_node) => on_hit(node.clone(), sibling_node),
                            Err(_) => on_hit(node.clone(), None),
                        }
                    }
                    None => on_hit(node.clone(), None),
                };
            }

            prev = Some(node.clone());

            write_node(db, node)
        })
    }
}

#[must_use]
pub fn get_path(num_level: usize, k: &[u8]) -> Vec<bool> {
    let mut path = Vec::with_capacity(num_level);
    for n in 0..num_level {
        path.push(test_bit(k, n));
    }
    path
}

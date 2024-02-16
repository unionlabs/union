use std::{collections::BTreeMap, sync::Arc};

use poseidon_rs::Fr;

use crate::{Error, Hash, HashScheme, Node};

pub trait Database {
    type Node;
    fn get_node(&self, key: &Hash) -> Result<Option<Arc<Self::Node>>, Error>;
    fn update_node(&mut self, node: Self::Node) -> Result<Arc<Self::Node>, Error>;
}

pub trait PreimageDatabase: Database {
    fn update_preimage(&mut self, preimage: &[u8], hash_field: &Fr);
    fn preimage(&self, key: &Fr) -> Vec<u8>;
}

pub struct MemDB<H: HashScheme> {
    map: BTreeMap<Hash, Arc<Node<H>>>,
    preimages: BTreeMap<Fr, Vec<u8>>,
}

impl<H: HashScheme> MemDB<H> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            preimages: BTreeMap::new(),
        }
    }
}

impl<H: HashScheme> PreimageDatabase for MemDB<H> {
    fn preimage(&self, key: &Fr) -> Vec<u8> {
        match self.preimages.get(key) {
            Some(val) => val.clone(),
            None => Vec::new(),
        }
    }

    fn update_preimage(&mut self, preimage: &[u8], hash_field: &Fr) {
        self.preimages.insert(*hash_field, preimage.to_vec());
    }
}

impl<H: HashScheme> Database for MemDB<H> {
    type Node = Node<H>;
    fn get_node(&self, key: &Hash) -> Result<Option<Arc<Self::Node>>, Error> {
        Ok(self.map.get(key).map(|n| n.clone()))
    }

    fn update_node(&mut self, node: Self::Node) -> Result<Arc<Self::Node>, Error> {
        let node = Arc::new(node);
        self.map.insert(*node.hash(), node.clone());
        Ok(node.clone())
    }
}

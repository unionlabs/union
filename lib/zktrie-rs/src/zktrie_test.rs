use std::collections::BTreeMap;

use unionlabs::hash::H256;

use crate::{
    byte32_test::TestHash, copy_truncated, decode_smt_proofs, to_secure_key, Byte32, Database,
    Error, Hash, MemDB, Node, NodeValue, PoseidonHash, TrieData, ZkTrie, ZERO_HASH,
};

pub struct TestTrie(ZkTrie<TestHash>, MemDB<TestHash>);

impl TestTrie {
    pub fn new(max_level: usize) -> Self {
        let db = MemDB::default();
        let root = Hash::default();
        let trie = <ZkTrie<TestHash>>::new(max_level, root);
        Self(trie, db)
    }

    pub fn root(&self) -> &Hash {
        self.0.hash()
    }

    pub fn try_get(&mut self, key: &[u8]) -> Result<TrieData<TestHash>, Error> {
        match self.0.try_get_node(&mut self.1, &Hash::from_bytes(key)) {
            Ok(node) => Ok(TrieData::Node(node)),
            Err(Error::KeyNotFound) => Ok(TrieData::NotFound),
            Err(err) => Err(err),
        }
    }

    pub fn update_word(&mut self, key: Byte32, value: Byte32) -> Result<(), Error> {
        let key = Hash::from_bytes(key.bytes());
        self.0.try_update(&mut self.1, &key, 1, vec![value])
    }

    pub fn add_word(&mut self, key: Byte32, value: Byte32) -> Result<(), Error> {
        match self.try_get(key.bytes())? {
            TrieData::Node(_) => return Err(Error::EntryIndexAlreadyExists),
            TrieData::NotFound => {}
        }
        let node_key = Hash::from_bytes(key.bytes());
        self.0.try_update(&mut self.1, &node_key, 1, vec![value])?;
        Ok(())
    }

    pub fn get_leaf_node_by_word(
        &mut self,
        key_preimage: &Byte32,
    ) -> Result<Node<TestHash>, Error> {
        self.0
            .try_get_node(&mut self.1, &Hash::from_bytes(key_preimage.bytes()))
    }

    pub fn delete_word(&mut self, key: &Byte32) -> Result<(), Error> {
        let new_key: Hash = key.into();
        self.0.try_delete(&mut self.1, &new_key)
    }
}

fn byte32_from_byte(b: u8) -> Byte32 {
    Byte32::from_bytes(&[b])
}

#[test]
fn test_zktrie_impl_update() {
    let k1 = byte32_from_byte(1);
    let k2 = byte32_from_byte(2);
    let k3 = byte32_from_byte(3);

    {
        // update 1
        let mut mt1 = TestTrie::new(10);
        mt1.add_word(k1, byte32_from_byte(1)).unwrap();
        let root1 = mt1.root().bytes();

        let mut mt2 = TestTrie::new(10);
        mt2.add_word(k1, byte32_from_byte(2)).unwrap();
        mt2.update_word(k1, byte32_from_byte(1)).unwrap();
        let root2 = mt2.root().bytes();

        assert_eq!(root1, root2);
        assert_eq!(
            root1,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 1, 0, 0,
                32, 0, 1, 0, 1
            ]
        );
    }

    {
        // update 2
        let mut mt1 = TestTrie::new(10);
        mt1.add_word(k1, byte32_from_byte(1)).unwrap();
        mt1.add_word(k2, byte32_from_byte(2)).unwrap();
        let root1 = mt1.root().bytes();

        let mut mt2 = TestTrie::new(10);
        mt2.add_word(k1, byte32_from_byte(1)).unwrap();
        mt2.add_word(k2, byte32_from_byte(3)).unwrap();
        mt2.update_word(k2, byte32_from_byte(2)).unwrap();
        let root2 = mt2.root().bytes();
        assert_eq!(
            root1,
            [
                34, 95, 229, 137, 232, 203, 223, 228, 36, 160, 50, 230, 226, 253, 17, 50, 118, 43,
                32, 121, 76, 255, 97, 240, 199, 14, 143, 117, 123, 106, 14, 215
            ]
        );
        assert_eq!(root1, root2);
    }

    {
        // update 1,2,3
        let mut mt1 = TestTrie::new(10);
        let mut mt2 = TestTrie::new(10);
        let keys = [k1, k2, k3];
        for (key, i) in keys.iter().zip(0..) {
            mt1.add_word(*key, byte32_from_byte(i)).unwrap();
        }
        for (key, i) in keys.iter().zip(0..) {
            mt2.add_word(*key, byte32_from_byte(i + 3)).unwrap();
        }
        for (key, i) in keys.iter().zip(0..) {
            mt1.update_word(*key, byte32_from_byte(i + 6)).unwrap();
            mt2.update_word(*key, byte32_from_byte(i + 6)).unwrap();
        }
        let root1 = mt1.root().bytes();
        let root2 = mt2.root().bytes();
        assert_eq!(
            root1,
            [
                25, 48, 90, 243, 147, 27, 34, 3, 233, 24, 50, 77, 134, 203, 220, 232, 124, 23, 204,
                92, 4, 114, 102, 187, 126, 213, 252, 140, 121, 211, 132, 76
            ]
        );
        assert_eq!(root1, root2);
    }

    {
        // update same value
        let mut mt = TestTrie::new(10);
        let keys = [k1, k2, k3];
        for key in keys {
            mt.add_word(key, byte32_from_byte(1)).unwrap();
            mt.update_word(key, byte32_from_byte(1)).unwrap();
            let node = mt.get_leaf_node_by_word(&key).unwrap();
            let leaf = node.leaf().unwrap();
            assert_eq!(leaf.value_preimage.len(), 1);
            assert_eq!(leaf.value_preimage[0], byte32_from_byte(1));
        }
    }

    {
        // update non-existent word
        let mut mt = TestTrie::new(10);
        mt.update_word(k1, byte32_from_byte(1)).unwrap();
        let node = mt.get_leaf_node_by_word(&k1).unwrap();
        let node = node.leaf().unwrap();
        assert_eq!(node.value_preimage.len(), 1);
        assert_eq!(node.value_preimage[0], byte32_from_byte(1));
    }
}

#[test]
fn test_zktrie_impl_add() {
    let k1 = Byte32::from_bytes(&[1]);
    let k2 = Byte32::from_bytes(&[2]);
    let k3 = Byte32::from_bytes(&[3]);

    let kv_map = {
        let mut n = BTreeMap::new();
        n.insert(k1, k1);
        n.insert(k2, k2);
        n.insert(k3, k3);
        n
    };

    {
        // Add 1 and 2 in different orders
        let orders = vec![vec![k1, k2], vec![k2, k1]];

        let mut roots = vec![];
        for order in orders {
            let mut trie = TestTrie::new(10);
            for key in order {
                let value = kv_map.get(&key).unwrap();
                trie.add_word(key, *value).unwrap();
            }
            roots.push(trie.0.hash().bytes());
        }
        assert_eq!(roots[0], roots[1]);
    }

    {
        // Add 1, 2, 3 in different orders
        let orders = vec![
            vec![k1, k2, k3],
            vec![k1, k3, k2],
            vec![k2, k1, k3],
            vec![k2, k3, k1],
            vec![k3, k1, k2],
            vec![k3, k2, k1],
        ];

        let mut roots = vec![];
        for order in orders {
            let mut trie = TestTrie::new(10);
            for key in order {
                let value = kv_map.get(&key).unwrap();
                trie.add_word(key, *value).unwrap();
            }
            roots.push(trie.0.hash().bytes());
        }

        for i in 1..roots.len() {
            assert_eq!(roots[0], roots[i]);
        }
    }

    {
        // Add twice
        let keys = vec![k1, k2, k3];

        let mut trie = TestTrie::new(10);
        for key in keys {
            trie.add_word(key, *kv_map.get(&key).unwrap()).unwrap();
            let err = trie.add_word(key, *kv_map.get(&key).unwrap());
            assert_eq!(err, Err(Error::EntryIndexAlreadyExists));
        }
    }
}

#[test]
fn test_zktrie_impl_delete() {
    let k1 = byte32_from_byte(1);
    let k2 = byte32_from_byte(2);
    let k3 = byte32_from_byte(3);
    let k4 = byte32_from_byte(4);

    {
        // Test deletion leads to empty tree
        let empty_mt = TestTrie::new(10);
        let mut mt1 = TestTrie::new(10);
        mt1.add_word(k1, byte32_from_byte(1)).unwrap();
        mt1.delete_word(&k1).unwrap();
        assert_eq!(mt1.root(), &ZERO_HASH);
        assert_eq!(empty_mt.root(), mt1.root());

        let keys = [k1, k2, k3, k4];
        let mut mt2 = TestTrie::new(10);
        for key in keys {
            mt2.add_word(key, byte32_from_byte(1)).unwrap();
        }
        for key in keys {
            mt2.delete_word(&key).unwrap();
        }
        assert_eq!(mt2.root(), &ZERO_HASH);
        assert_eq!(empty_mt.root(), mt2.root());

        let mut mt3 = TestTrie::new(10);
        for key in keys {
            mt3.add_word(key, byte32_from_byte(1)).unwrap();
        }
        for key in keys.iter().rev() {
            mt3.delete_word(key).unwrap();
        }
        assert_eq!(&ZERO_HASH, mt3.root());
        assert_eq!(empty_mt.root(), mt3.root());
    }

    {
        // Test equivalent trees after deletion
        let keys = [k1, k2, k3, k4];
        let mut mt1 = TestTrie::new(10);
        for (key, i) in keys.iter().zip(0..) {
            mt1.add_word(*key, byte32_from_byte(i + 1)).unwrap();
        }
        mt1.delete_word(&k1).unwrap();
        mt1.delete_word(&k2).unwrap();

        let mut mt2 = TestTrie::new(10);
        mt2.add_word(k3, byte32_from_byte(3)).unwrap();
        mt2.add_word(k4, byte32_from_byte(4)).unwrap();
        assert_eq!(mt1.root(), mt2.root());

        let mut mt3 = TestTrie::new(10);
        for (key, i) in keys.iter().zip(0..) {
            mt3.add_word(*key, byte32_from_byte(i + 1)).unwrap();
        }
        mt3.delete_word(&k1).unwrap();
        mt3.delete_word(&k3).unwrap();
        let mut mt4 = TestTrie::new(10);
        mt4.add_word(k2, byte32_from_byte(2)).unwrap();
        mt4.add_word(k4, byte32_from_byte(4)).unwrap();

        assert_eq!(mt3.root(), mt4.root());
    }

    {
        // Test repeat deletion
        let mut mt = TestTrie::new(10);
        mt.add_word(k1, byte32_from_byte(1)).unwrap();
        mt.delete_word(&k1).unwrap();
        assert_eq!(mt.delete_word(&k1), Err(Error::KeyNotFound));
    }

    {
        // Test deletion of non-existent node
        let mut mt = TestTrie::new(10);
        assert_eq!(mt.delete_word(&k1), Err(Error::KeyNotFound));
    }
}

#[test]
fn test_merkle_tree_add_update_get_word() {
    struct TestData {
        key: u8,
        initial_val: u8,
        updated_val: u8,
    }
    impl TestData {
        pub fn new(key: u8, initial_val: u8, updated_val: u8) -> Self {
            Self {
                key,
                initial_val,
                updated_val,
            }
        }
    }
    let test_data = &[
        TestData::new(1, 2, 7),
        TestData::new(3, 4, 8),
        TestData::new(5, 6, 9),
    ];
    let mut mt = TestTrie::new(10);

    for td in test_data {
        let key = Byte32::from_bytes(&[td.key]);
        let value = Byte32::from_bytes_padding(&[td.initial_val]);
        mt.add_word(key, value).unwrap();

        let node = mt
            .get_leaf_node_by_word(&Byte32::from_bytes(&[td.key]))
            .unwrap();
        if let NodeValue::Leaf(leaf) = node.value() {
            assert_eq!(1, leaf.value_preimage.len());
            assert_eq!(value, leaf.value_preimage[0]);
        } else {
            unreachable!()
        }
    }

    let result = mt.add_word(Byte32::from_bytes(&[5]), Byte32::from_bytes_padding(&[7]));
    assert_eq!(result, Err(Error::EntryIndexAlreadyExists));

    for td in test_data {
        mt.update_word(
            Byte32::from_bytes(&[td.key]),
            Byte32::from_bytes_padding(&[td.updated_val]),
        )
        .unwrap();

        let node = mt
            .get_leaf_node_by_word(&Byte32::from_bytes(&[td.key]))
            .unwrap();
        if let NodeValue::Leaf(node) = node.value() {
            assert_eq!(node.value_preimage.len(), 1);
            assert_eq!(
                Byte32::from_bytes_padding(&[td.updated_val]),
                node.value_preimage[0]
            );
        } else {
            unreachable!();
        }
    }

    let result = mt.get_leaf_node_by_word(&Byte32::from_bytes_padding(&[100]));
    assert_eq!(result, Err(Error::KeyNotFound));
}

#[test]
fn test_merkle_tree_deletion() {
    {
        // Check root consistency
        let mut trie = TestTrie::new(10);
        let mut hashes = vec![trie.0.hash().bytes()];

        let tmp = [0_u8; 32];
        for i in 0..6 {
            let key = Byte32::from_bytes(&{
                let mut t = tmp;
                if let Some(v) = t.last_mut() {
                    *v = i;
                }
                t
            });
            let value = Byte32::from_bytes(&{
                let mut t = tmp;
                t[0] = i;
                t
            });
            trie.add_word(key, value).unwrap();
            hashes.push(trie.0.hash().bytes());
        }
        for i in (0..6).rev() {
            let key = Byte32::from_bytes(&[i]);
            trie.delete_word(&key).unwrap();
            assert_eq!(trie.0.hash().bytes(), hashes[i as usize]);
        }
    }

    {
        // Check depth
        let mut trie = TestTrie::new(10);
        let key1 = Byte32::from_bytes(&[67]);
        trie.add_word(key1, byte32_from_byte(67)).unwrap();
        let root_phase1 = *trie.0.hash();
        let key2 = Byte32::from_bytes(&[131]);
        trie.add_word(key2, byte32_from_byte(131)).unwrap();
        let root_phase2 = *trie.root();

        let assert_key_depth = |trie: &mut TestTrie, key: &Byte32, expected_dep: usize| {
            let mut level_cnt = 0;
            trie.0
                .walk(&mut trie.1, &Hash::from_bytes(key.bytes()), 0, |_, _| {
                    level_cnt += 1;
                    Ok(())
                })
                .unwrap();
            assert_eq!(expected_dep, level_cnt);
        };

        assert_key_depth(&mut trie, &key1, 8);
        assert_key_depth(&mut trie, &key2, 8);

        trie.delete_word(&key2).unwrap();
        assert_key_depth(&mut trie, &key1, 1);
        assert_eq!(&root_phase1, trie.root());

        trie.add_word(key2, byte32_from_byte(131)).unwrap();
        assert_eq!(&root_phase2, trie.root());
        assert_key_depth(&mut trie, &key1, 8);

        // delete node with parent sibling
        let key3 = Byte32::from_bytes(&[19]);
        trie.add_word(key3, byte32_from_byte(19)).unwrap();
        trie.delete_word(&key3).unwrap();
        assert_key_depth(&mut trie, &key1, 8);
        assert_eq!(&root_phase2, trie.root());

        let key4 = Byte32::from_bytes(&[4]);
        trie.add_word(key4, byte32_from_byte(4)).unwrap();
        assert_key_depth(&mut trie, &key4, 2);
        trie.delete_word(&key4).unwrap();
        assert_eq!(&root_phase2, trie.root());
    }
}

#[test]
fn test_new_zktrie() {
    let root = Hash::default();
    let trie = <ZkTrie<TestHash>>::new(248, root);
    assert_eq!(trie.hash(), &ZERO_HASH);
}

#[test]
fn test_zktrie_random() {
    let root = Hash::default();
    let mut db = MemDB::default();
    let db = &mut db;
    let mut trie = <ZkTrie<TestHash>>::new(248, root);

    let data = vec![
        (
            hex::decode("2edac2e5866fdd10ccdc27a7cab08453f0f59b92c18403693082143d66ee3474")
                .unwrap(),
            hex::decode("00000000000000000000000000000000000000000000000000ee121424c78f15")
                .unwrap(),
        ),
        (
            hex::decode("3e6e49c9e1aaf563ab0c8372f2494528fb3040a9c290cf0dfb7f45230d8b43fe")
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000632826117cfb39eefd")
                .unwrap(),
        ),
        (
            hex::decode("ed812ec25670a9f1e970acd0f591e82de9f941301d24780ae8f12666b08ab360")
                .unwrap(),
            hex::decode("00000000000000000000000000000000000000000000000000ee121424c78f15")
                .unwrap(),
        ),
    ];
    let delete_idx = vec![0];

    for (k, v) in &data {
        let k: H256 = k.clone().try_into().unwrap();
        let v: H256 = v.clone().try_into().unwrap();
        let v = Byte32::from_bytes_padding(v.get());
        trie.update(db, k.as_ref(), 1, vec![v]).unwrap();
    }
    for idx in delete_idx {
        let key: H256 = data[idx].0.clone().try_into().unwrap();
        trie.delete(db, key.as_ref()).unwrap();
    }
    assert_eq!(
        trie.hash(),
        &Hash::from_hex("15e0373f921e4f9d3c5f29f89a5714f50ed9b7b461c04e8ecc4262ae65588079")
            .unwrap(),
    );
}

#[test]
fn test_zktrie_get_update_delete() {
    let mut db = MemDB::default();
    let db = &mut db;
    let root = Hash::default();
    let mut trie = <ZkTrie<TestHash>>::new(248, root);

    let val = trie.get_data(db, b"key").unwrap();
    assert_eq!(val, TrieData::NotFound);
    assert_eq!(trie.hash(), &ZERO_HASH);

    trie.update(db, b"key", 1, vec![Byte32::from_bytes_padding(&[1])])
        .unwrap();
    let expect = Hash::from_bytes(&[
        0x23_u8, 0x36, 0x5e, 0xbd, 0x71, 0xa7, 0xad, 0x35, 0x65, 0xdd, 0x24, 0x88, 0x47, 0xca,
        0xe8, 0xe8, 0x8, 0x21, 0x15, 0x62, 0xc6, 0x83, 0xdb, 0x8, 0x4f, 0x5a, 0xfb, 0xd1, 0xb0,
        0x3d, 0x4c, 0xb5,
    ]);
    assert_eq!(trie.hash(), &expect);

    let val = trie.get_data(db, b"key").unwrap();
    assert_eq!(val.get(), Byte32::from_bytes_padding(&[1]).bytes());

    trie.delete(db, b"key").unwrap();
    assert_eq!(trie.hash(), &ZERO_HASH);

    let val = trie.get_data(db, b"key").unwrap();
    assert_eq!(TrieData::NotFound, val);
}

#[test]
fn test_zktrie_prove_and_prove_with_deletion() {
    let mut db = MemDB::default();
    let db = &mut db;
    let mut trie = <ZkTrie<TestHash>>::new(248, Hash::default());

    let keys = &["key1", "key2", "key3", "key4", "key5"];
    for (i, key_str) in keys.iter().enumerate() {
        let mut key = vec![0_u8; 32];
        copy_truncated(&mut key, key_str.as_bytes());

        trie.update(db, &key, i as u32 + 1, vec![byte32_from_byte(i as u8 + 1)])
            .unwrap();

        let k = to_secure_key::<TestHash>(&key).unwrap();

        for j in 0..i {
            let key: Hash = k.into();
            trie.prove_with_deletion(db, &key.bytes(), j, |_, _| Ok(()), Some(|_, _| {}))
                .unwrap();
            trie.prove(db, &key.bytes(), j, |_, _| Ok(())).unwrap();
        }
    }
}

#[test]
fn test_zktrie_statedb() {
    type H = PoseidonHash;
    let mut db = <MemDB<H>>::default();
    let db = &mut db;

    for bts in EXAMPLE {
        let buf = hex::decode(bts.get(2..).unwrap()).unwrap();
        let node = decode_smt_proofs::<H>(&buf).unwrap().unwrap();
        db.update_node(node).unwrap();
    }

    let root =
        Hash::from_hex("194cfd0c3cce58ac79c5bab34b149927e0cd9280c6d61870bfb621d45533ddbc").unwrap();
    let mut zktrie = <ZkTrie<H>>::new(248, root);
    assert_eq!(zktrie.hash(), &root);

    let acc_key = hex::decode("1C5A77d9FA7eF466951B2F01F724BCa3A5820b63").unwrap();
    let acc_data = zktrie.get_data(db, &acc_key).unwrap();
    let acc_data = acc_data.get();
    let nonce_code = &acc_data[..32];
    assert_eq!(
        nonce_code,
        hex::decode("0000000000000000000000000000000000000000000000000000000000000011").unwrap()
    );
    let balance = &acc_data[32..64];
    assert_eq!(
        balance,
        hex::decode(b"01ffffffffffffffffffffffffffffffffffffffffffd5a5fa65e20465da88bf").unwrap()
    );
    let code_hash = &acc_data[96..128];
    assert_eq!(
        code_hash,
        hex::decode(b"c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap()
    );

    let mut acc_data = acc_data.to_vec();
    acc_data[31] += 1;
    let newacc = Byte32::from_vec_bytes(&acc_data);

    zktrie.update(db, &acc_key, 8, newacc).unwrap();

    let new_acc_data = zktrie.get_data(db, &acc_key).unwrap();
    assert_eq!(new_acc_data.get(), &acc_data);

    let root =
        hex_root("9a88bda22f50dc0fda6c355fd93c025df7f7ce6e3d0b979942ebd981c1c6c71c").unwrap();
    assert_eq!(&root, zktrie.hash());

    let mut newacc = new_acc_data.get().to_vec();
    newacc[32..64].copy_from_slice(
        &hex::decode(b"01ffffffffffffffffffffffffffffffffffffffffffd5a5fa65b10989405cd7").unwrap(),
    );
    let newacc = Byte32::from_vec_bytes(&newacc);
    zktrie.update(db, &acc_key, 8, newacc.clone()).unwrap();
    let root =
        hex_root("7f787ee24805a9e5f69dc3a91ce68ef86d9358ce9c35729bd68660ccf6f9d909").unwrap();
    assert_eq!(&root, zktrie.hash());

    let proof = zktrie.proof(db, &acc_key).unwrap();

    assert_eq!(proof.len(), 8);
    assert_eq!(proof[7], hex::decode("5448495320495320534f4d45204d4147494320425954455320464f5220534d54206d3172525867503278704449").unwrap());
    assert_eq!(proof[3], hex::decode("0810b051b9facdd51b7fd1a1cf8e9a62facef17c80c7be0db1f15f3cda95982e34233b07e4b000250359a56ef55485036e6d4dbca7c71bf82812790ac3f4a5238e").unwrap());

    let node = <Node<H>>::from_bytes(&proof[6]).unwrap();
    assert_eq!(
        &node.hash().bytes(),
        hex::decode("272f093df377b234e179b70dc1a04a1543072be3c7d3a47f6e59004c84639907")
            .unwrap()
            .as_slice()
    );
    assert_eq!(
        &node
            .leaf()
            .map(|n| n.value_hash::<H>().unwrap())
            .unwrap()
            .bytes(),
        hex::decode("06c7c55f4d38fa2c6f6e0e655038ae7e1b3bb9dfa8954bdec0f9708e6e6b7d72")
            .unwrap()
            .as_slice()
    );
    assert!(node.is_terminal());
    let node_data = node.data();
    assert_eq!(
        node_data[32..64].to_vec(),
        hex::decode("01ffffffffffffffffffffffffffffffffffffffffffd5a5fa65b10989405cd7")
            .unwrap()
            .as_slice(),
    );

    zktrie.delete(db, &acc_key).unwrap();
    assert!(zktrie.get_data(db, &acc_key).unwrap().get().is_empty());

    zktrie.update(db, &acc_key, 8, newacc).unwrap();
    assert_eq!(zktrie.hash(), &root);
}

fn hex_root(d: &str) -> Result<Hash, String> {
    let mut data = hex::decode(d.as_bytes())
        .map_err(|d| format!("{d:?}"))?
        .clone();
    data.reverse();
    Ok(Hash::from_bytes(&data))
}

static EXAMPLE : [&str;41] = [
    "0x09218bcaf094949451aaea2273a4092c7116839ad69df7597df06c7bf741a9477f01020df75837d8a760bfb941f3465f63812b205ac7e1fff5d310a2a3295e60c8",
    "0x0913e957fbc8585b40175129d3547a76b9fc3a1c3b16a6ca4de468879bb08fcbb6104a71f54260a0430906c4a0c3cc5eb459dd132b637c944ea92b769a98dba762",
    "0x092c2eae4f5273c398709da3e317c86a3a817008c98269bf2766405259c488306628c0c92eb1f16fc59b8b99e0a8abee3f88afb477c4d36be3571d537b076e0f83",
    "0x0800100f66e758c81427817699eeed67308bc9a7ee8054f2cbd463b7bf252610af233b07e4b000250359a56ef55485036e6d4dbca7c71bf82812790ac3f4a5238e",
    "0x08088158f4dfd26b06688c646a453c1b52710139a064b0394b47a0693c2bee46a4159d39c4d2776406bca63dfba405861d669f6220a087ad4b204e1cca52c7be5f",
    "0x062b2d9de4b02c2bab78264918866524e44e6efdc24bf0be2d4a8aa6f9b232a7781cb2c64090d483dbe3795eea941f808f7eda30de68190976a36f856f2a824bdd",
    "0x040a30b5d71d70991519167c5314323d2d69b02b7c501070ec7f34f4f24d89b5860508000000000000000000000000000000000000000000000000119b000000000000000100000000000000000000000000000000000000000000000000000000000000001a99ce3a54bcc9f4d7f61c67286f0ffc6a5ddab4a94c1f6fc6741a5ef196145b16fc66d15010e6213d2a009f57ed8e847717ea0b83eeb37cd322e9ad1b018a3e0d85b09a93d5ed99a87d27dcf6d50e4459d16bb694e70f89eefcb745ea1c85e7200c64e6f8d51bb1ae0e4ad62b9a1b996e1b2675d3000000000000000000000000",
    "0x09218bcaf094949451aaea2273a4092c7116839ad69df7597df06c7bf741a9477f01020df75837d8a760bfb941f3465f63812b205ac7e1fff5d310a2a3295e60c8",
    "0x0913e957fbc8585b40175129d3547a76b9fc3a1c3b16a6ca4de468879bb08fcbb6104a71f54260a0430906c4a0c3cc5eb459dd132b637c944ea92b769a98dba762",
    "0x092c2eae4f5273c398709da3e317c86a3a817008c98269bf2766405259c488306628c0c92eb1f16fc59b8b99e0a8abee3f88afb477c4d36be3571d537b076e0f83",
    "0x0800100f66e758c81427817699eeed67308bc9a7ee8054f2cbd463b7bf252610af233b07e4b000250359a56ef55485036e6d4dbca7c71bf82812790ac3f4a5238e",
    "0x08088158f4dfd26b06688c646a453c1b52710139a064b0394b47a0693c2bee46a4159d39c4d2776406bca63dfba405861d669f6220a087ad4b204e1cca52c7be5f",
    "0x062b2d9de4b02c2bab78264918866524e44e6efdc24bf0be2d4a8aa6f9b232a7781cb2c64090d483dbe3795eea941f808f7eda30de68190976a36f856f2a824bdd",
    "0x041822829dca763241624d1f8dd4cf59018fc5f69931d579f8e8a4c3addd6633e605080000000000000000000000000000000000000000000000000000000000000000001101ffffffffffffffffffffffffffffffffffffffffffd5a5fa65e20465da88bf0000000000000000000000000000000000000000000000000000000000000000c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a4702098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864201c5a77d9fa7ef466951b2f01f724bca3a5820b63000000000000000000000000",
    "0x092df5ac113a2c9174aea818559d63df596efdd925bcc14028e901ba605dc030e101ebd1fa8391b5fa5b805444d74896d14cfac9519260e94ab9ef25ee4461f737",
    "0x070000000000000000000000000000000000000000000000000000000000000000104736bbf00e9ab6f74b9e366c28b4f21c4a273cbd1f7e3dff3d68d4dbfe6d76",
    "0x060a9837791a40c9befa2ebdbbe99fdb8d8d7a7bb9fe3a4c14f1581a76809bf2b21323d7866288f9d670672215af41d0d610303b7e5f6ba97b5e54080960974580",
    "0x041aed9d52b6e3489c0ea97983a6dc4fbad57507090547dc83b8830c2ddb88577701010000000000000000000000001c5a77d9fa7ef466951b2f01f724bca3a5820b630012200000000000000000000000000000000000000000000000000000000000000005",
    "0x09218bcaf094949451aaea2273a4092c7116839ad69df7597df06c7bf741a9477f01020df75837d8a760bfb941f3465f63812b205ac7e1fff5d310a2a3295e60c8",
    "0x0913e957fbc8585b40175129d3547a76b9fc3a1c3b16a6ca4de468879bb08fcbb6104a71f54260a0430906c4a0c3cc5eb459dd132b637c944ea92b769a98dba762",
    "0x092c2eae4f5273c398709da3e317c86a3a817008c98269bf2766405259c488306628c0c92eb1f16fc59b8b99e0a8abee3f88afb477c4d36be3571d537b076e0f83",
    "0x0800100f66e758c81427817699eeed67308bc9a7ee8054f2cbd463b7bf252610af233b07e4b000250359a56ef55485036e6d4dbca7c71bf82812790ac3f4a5238e",
    "0x04113060bdeae1240b8b2f272e35848ac6b0c401bdc3a9ec20186da2a6a9d4607e05080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000152d02c7e14af60000000000000000000000000000000000000000000000000000000000000000000000c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a4702098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b6486420c0c4c8baea3f6acb49b6e1fb9e2adeceeacb0ca2000000000000000000000000",
    "0x09218bcaf094949451aaea2273a4092c7116839ad69df7597df06c7bf741a9477f01020df75837d8a760bfb941f3465f63812b205ac7e1fff5d310a2a3295e60c8",
    "0x0908e49a63f6ecd17ace446bd1e684b6cdd29f31faae528a9f058aefa76551068228eeef32a81cf40e295ad9c1de7e53a5180e6f1727521a209e0e2913250941fe",
    "0x082e4e1a6f0a26fe354020a569325d45d9b63e11f769a79620da6c84c053d88733252f02bd2a45416d5076e363e6172f941774eb184ce75ba0803264362958e2ef",
    "0x08020cc627de460d025af928a8a847b8d7475ff44bcaadce1667cfab122c8f3ea6301dc3e787d41a3db0710353073f18eaebab31ac37d69e25983caf72f6c08178",
    "0x04139a6815e4d1fb05c969e6a8036aa5cc06b88751d713326d681bd90448ea64c905080000000000000000000000000000000000000000000000000874000000000000000000000000000000000000000000000000000000000000000000000000000000002c3c54d9c8b2d411ccd6458eaea5c644392b097de2ee416f5c923f3b01c7b8b80fabb5b0f58ec2922e2969f4dadb6d1395b49ecd40feff93e01212ae848355d410e77cae1c507f967948c6cd114e74ed65f662e365c7d6993e97f78ce898252800",
    "0x09218bcaf094949451aaea2273a4092c7116839ad69df7597df06c7bf741a9477f01020df75837d8a760bfb941f3465f63812b205ac7e1fff5d310a2a3295e60c8",
    "0x0908e49a63f6ecd17ace446bd1e684b6cdd29f31faae528a9f058aefa76551068228eeef32a81cf40e295ad9c1de7e53a5180e6f1727521a209e0e2913250941fe",
    "0x082e4e1a6f0a26fe354020a569325d45d9b63e11f769a79620da6c84c053d88733252f02bd2a45416d5076e363e6172f941774eb184ce75ba0803264362958e2ef",
    "0x08020cc627de460d025af928a8a847b8d7475ff44bcaadce1667cfab122c8f3ea6301dc3e787d41a3db0710353073f18eaebab31ac37d69e25983caf72f6c08178",
    "0x0700000000000000000000000000000000000000000000000000000000000000000d652d6e2cc697970d24bfec9c84b720481a080eeb3a039277d5dfa90c634a02",
    "0x060b262fa2cc2bcdf4083a6b4b45956ebcf85003d697780351a24398b7df39985a096c33b369382285822d8f0acf8097ca6f095334750a42f869e513c8ec3779a7",
    "0x04287b801ba8950befe82147f88e71eff6b85eb921845d754c9c2a165a4ec86791050800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a5b65ae2577410000000000000000000000000000000000000000000000000000000000000000c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a4702098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864205300000000000000000000000000000000000005000000000000000000000000",
    "0x092e757f7cfb7c618a89bef428d6f043efb7913959793a525d3e6dc2265aa2e0362c9e569b67ba72d58e6f56454481607aee49523e3da63072e2cb4e0b37453e8a",
    "0x091868286870969b61281e49af8860d1bc74b558a4014da7433cb7e99a88aa56bc2d58daf89ed4b660018c081b11785924bd129ce58535350bd66c23eddf591e2b",
    "0x0900d86fc3cea9f88796671391157d8433f92be74473b01876ef9b6a75632c225d159af6801572801dfd6e17b00de85fcf0dae392c520440b763ecfc3936970af5",
    "0x0911b101680f5f11b4cccdcde4115c3f8e8af523fa76dd52de98c468cc0502dd642fd7d2a38e36d5a616485e21c93edb5798618e0e0e2003b979d05a94b29b2b29",
    "0x070000000000000000000000000000000000000000000000000000000000000000240aaaaee47745183d4820fe7384efe4a3fb93461aecea38b0a7d7bee64784a5",
    "0x05",
];

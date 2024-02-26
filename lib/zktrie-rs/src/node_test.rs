use crate::{byte32_test::TestHash, node::*, Byte32, Hash, ZERO_HASH};

fn gen_key(n: char) -> Vec<u8> {
    let mut tmp = vec![n as u8; 32];
    tmp[0] = 0;
    tmp[1] = 1;
    tmp
}

#[test]
fn test_new_node() {
    type H = TestHash;
    type Node = crate::Node<TestHash>;
    {
        let node = Node::new_empty();
        assert_eq!(node.hash(), &ZERO_HASH);
    }

    {
        // newLeafNode
        let tmp = gen_key('a');
        let k = Hash::from_bytes(&tmp);
        let vp = Byte32::from_vec_bytes(&tmp);
        let node = Node::new_leaf(k, 1, vp.clone(), None).unwrap();
        match node.value() {
            NodeValue::Leaf(node) => {
                assert_eq!(node.compressed_flags, 1);
                assert_eq!(node.value_preimage, vp);
                let hash_from_vp = vp[0].hash::<H>().unwrap();
                let hash_from_vp_hash: Hash = hash_from_vp.into();
                assert_eq!(hash_from_vp_hash, node.value_hash::<H>().unwrap());
            }
            _ => unreachable!(),
        }
        assert_eq!(
            node.hash(),
            &"1cf0dfce3d72cce73d40668bb1f67c26b9f4fd9e5d91e88fd61e8e77f3cbf9bd".into()
        );
    }

    {
        // test branch node
        let k = Hash::from_bytes(&gen_key('a'));
        let node = Node::new_branch_ty(BranchType::BothBranch, k, k).unwrap();
        match node.value() {
            NodeValue::Branch(node) => {
                assert_eq!(node.left.hash(), &k);
                assert_eq!(node.right.hash(), &k);
                assert_eq!(node.ty(), BranchType::BothBranch);
            }
            _ => unreachable!(),
        }
        assert_eq!(
            node.hash(),
            &"2a734a18a728b090a9c0692c70421c1fe1f1d2996965b4a852cb45ccd24a3af9".into()
        );
    }

    {
        // Test NewParentNodeWithEmptyChild
        let k = Hash::from_bytes(&gen_key('a'));
        let r = *Node::new_empty().hash();
        let node = Node::new_branch_ty(BranchType::RightTerminal, k, r).unwrap();
        match node.value() {
            NodeValue::Branch(node) => {
                assert_eq!(BranchHash::Branch(k), node.left);
                assert_eq!(BranchHash::Terminal(r), node.right);
            }
            _ => unreachable!(),
        }
        assert_eq!(
            node.hash(),
            &"115f16981bd884be59935291890adee837ebd14593e4c486b68ca358a330ea4c".into(),
        );
    }
}

#[test]
fn test_new_node_from_bytes() {
    type H = TestHash;
    type Node = crate::Node<TestHash>;
    {
        // ParentNode
        let k1 = Hash::from_bytes(&gen_key('a'));
        let k2 = Hash::from_bytes(&gen_key('b'));
        let node = Node::new_branch_ty(BranchType::BothTerminal, k1, k2).unwrap();
        let b = node.bytes();

        let node = Node::from_bytes(&b).unwrap();
        assert!(node.is_branch());
        match node.value() {
            NodeValue::Branch(node) => {
                assert_eq!(node.left, BranchHash::Terminal(k1));
                assert_eq!(node.right, BranchHash::Terminal(k2));
            }
            _ => unreachable!(),
        }
        assert_eq!(
            node.hash(),
            &"2156268e5220bbbd2e832d120f693ed8b42c87bf9bfb871d44886c61503f0c4e".into(),
        );
    }
    {
        // LeafNode
        let k = Hash::from_bytes(&gen_key('a'));
        let vp = vec![Byte32::default()];
        let node =
            Node::new_leaf(k, 1, vp.clone(), Some(Byte32::from_bytes(&gen_key('b')))).unwrap();
        assert_eq!(
            node.bytes(),
            &[
                4, 0, 1, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97,
                97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 1,
                98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98,
                98, 98, 98, 98, 98, 98, 98, 98, 98,
            ]
        );

        let new_node = Node::from_bytes(&node.bytes()).unwrap();
        assert_eq!(node, new_node);

        assert_eq!(
            node.hash(),
            &"1cd03939ac78031d321c938f64fd1f66486d291348500f17fe55ef6e9da19529".into()
        );

        let hash = node.leaf().unwrap().value_hash::<H>().unwrap();
        let hash_from_vp = vp[0].hash::<H>().unwrap();

        assert_eq!(hash, hash_from_vp.into());
    }
    {
        // EmptyNode
        let node = Node::new_empty();
        let b = node.bytes();

        let node = Node::from_bytes(&b).unwrap();
        assert!(node.is_empty());

        assert_eq!(node.hash(), &ZERO_HASH);
    }

    {
        // BadSize
    }
}

#[test]
fn test_node_value_and_data() {
    type Node = crate::Node<TestHash>;
    let k = Hash::from_bytes(&gen_key('a'));
    let vp = vec![Byte32::from_bytes(&gen_key('b'))];

    let node = Node::new_leaf(k, 1, vp, None).unwrap();
    let canonical_value = node.canonical_value();
    assert_eq!(
        &canonical_value,
        &[
            0x4, 0x0, 0x1, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x1, 0x1, 0x0, 0x0, 0x0, 0x1, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x0
        ]
    );

    assert_eq!(
        &node.bytes(),
        &[
            0x4, 0x0, 0x1, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x1, 0x1, 0x0, 0x0, 0x0, 0x1, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x0
        ]
    );
    let new_value = Byte32::from_bytes(&gen_key('c'));
    let node = node.leaf_set_keypreimage(Some(new_value)).unwrap();
    assert_eq!(
        &node.bytes(),
        &[
            0x4, 0x0, 0x1, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x1, 0x1, 0x0, 0x0, 0x0, 0x1, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x20, 0x0, 0x1, 0x63,
            0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63,
            0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63,
            0x63
        ]
    );
    assert_eq!(
        node.data(),
        &[
            0x0, 0x1, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62,
            0x62, 0x62, 0x62
        ]
    );

    let parent_node = Node::new_branch_ty(BranchType::BothBranch, k, k).unwrap();
    assert_eq!(
        &parent_node.canonical_value(),
        &[
            0x9, 0x0, 0x1, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x0, 0x1, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61
        ]
    );
    assert!(parent_node.data().is_empty());

    let empty = Node::empty();
    assert_eq!(empty.canonical_value(), &[5]);
    assert!(empty.data().is_empty());
}

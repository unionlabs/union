use borsh::maybestd::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
#[borsh_init(init)]
struct A<'a> {
    x: u64,
    b: B,
    y: f32,
    z: String,
    t: (String, u64),
    m: HashMap<String, String>,
    s: HashSet<u64>,
    btree_map_string: BTreeMap<String, String>,
    btree_set_u64: BTreeSet<u64>,
    linked_list_string: LinkedList<String>,
    vec_deque_u64: VecDeque<u64>,
    v: Vec<String>,
    w: Box<[u8]>,
    box_str: Box<str>,
    i: [u8; 32],
    u: std::result::Result<String, String>,
    lazy: Option<u64>,
    c: std::borrow::Cow<'a, str>,
    cow_arr: std::borrow::Cow<'a, [std::borrow::Cow<'a, str>]>,
    range_u32: std::ops::Range<u32>,
    #[borsh_skip]
    skipped: Option<u64>,
}

impl A<'_> {
    pub fn init(&mut self) {
        if let Some(v) = self.lazy.as_mut() {
            *v *= 10;
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct B {
    x: u64,
    y: i32,
    c: C,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
enum C {
    C1,
    C2(u64),
    C3(u64, u64),
    C4 { x: u64, y: u64 },
    C5(D),
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct D {
    x: u64,
}

#[derive(BorshSerialize)]
struct E<'a, 'b> {
    a: &'a A<'b>,
}

#[derive(BorshSerialize)]
struct F1<'a, 'b> {
    aa: &'a [&'a A<'b>],
}

#[derive(BorshDeserialize)]
struct F2<'b> {
    aa: Vec<A<'b>>,
}

#[test]
fn test_simple_struct() {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("test".into(), "test".into());
    let mut set: HashSet<u64> = HashSet::new();
    set.insert(std::u64::MAX);
    let cow_arr = [
        std::borrow::Cow::Borrowed("Hello1"),
        std::borrow::Cow::Owned("Hello2".to_string()),
    ];
    let a = A {
        x: 1,
        b: B {
            x: 2,
            y: 3,
            c: C::C5(D { x: 1 }),
        },
        y: 4.0,
        z: "123".to_string(),
        t: ("Hello".to_string(), 10),
        m: map.clone(),
        s: set.clone(),
        btree_map_string: map.clone().into_iter().collect(),
        btree_set_u64: set.clone().into_iter().collect(),
        linked_list_string: vec!["a".to_string(), "b".to_string()].into_iter().collect(),
        vec_deque_u64: vec![1, 2, 3].into_iter().collect(),
        v: vec!["qwe".to_string(), "zxc".to_string()],
        w: vec![0].into_boxed_slice(),
        box_str: Box::from("asd"),
        i: [4u8; 32],
        u: Ok("Hello".to_string()),
        lazy: Some(5),
        c: std::borrow::Cow::Borrowed("Hello"),
        cow_arr: std::borrow::Cow::Borrowed(&cow_arr),
        range_u32: 12..71,
        skipped: Some(6),
    };
    let encoded_a = a.try_to_vec().unwrap();
    let e = E { a: &a };
    let encoded_ref_a = e.try_to_vec().unwrap();
    assert_eq!(encoded_ref_a, encoded_a);

    let decoded_a = A::try_from_slice(&encoded_a).unwrap();
    let expected_a = A {
        x: 1,
        b: B {
            x: 2,
            y: 3,
            c: C::C5(D { x: 1 }),
        },
        y: 4.0,
        z: a.z.clone(),
        t: ("Hello".to_string(), 10),
        m: map.clone(),
        s: set.clone(),
        btree_map_string: map.clone().into_iter().collect(),
        btree_set_u64: set.clone().into_iter().collect(),
        linked_list_string: vec!["a".to_string(), "b".to_string()].into_iter().collect(),
        vec_deque_u64: vec![1, 2, 3].into_iter().collect(),
        v: a.v.clone(),
        w: a.w.clone(),
        box_str: Box::from("asd"),
        i: a.i,
        u: Ok("Hello".to_string()),
        lazy: Some(50),
        c: std::borrow::Cow::Owned("Hello".to_string()),
        cow_arr: std::borrow::Cow::Owned(vec![
            std::borrow::Cow::Borrowed("Hello1"),
            std::borrow::Cow::Owned("Hello2".to_string()),
        ]),
        range_u32: 12..71,
        skipped: None,
    };

    assert_eq!(expected_a, decoded_a);

    let f1 = F1 { aa: &[&a, &a] };
    let encoded_f1 = f1.try_to_vec().unwrap();
    let decoded_f2 = F2::try_from_slice(&encoded_f1).unwrap();
    assert_eq!(decoded_f2.aa.len(), 2);
    assert!(decoded_f2.aa.iter().all(|f2_a| f2_a == &expected_a));
}

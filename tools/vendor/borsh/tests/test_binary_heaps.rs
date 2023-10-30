use borsh::maybestd::collections::BinaryHeap;
use borsh::{BorshDeserialize, BorshSerialize};

macro_rules! test_binary_heap {
    ($v: expr, $t: ty) => {
        let buf = $v.try_to_vec().unwrap();
        let actual_v: BinaryHeap<$t> =
            BorshDeserialize::try_from_slice(&buf).expect("failed to deserialize");
        assert_eq!(actual_v.into_vec(), $v.into_vec());
    };
}

macro_rules! test_binary_heaps {
    ($test_name: ident, $el: expr, $t: ty) => {
        #[test]
        fn $test_name() {
            test_binary_heap!(BinaryHeap::<$t>::new(), $t);
            test_binary_heap!(vec![$el].into_iter().collect::<BinaryHeap<_>>(), $t);
            test_binary_heap!(vec![$el; 10].into_iter().collect::<BinaryHeap<_>>(), $t);
            test_binary_heap!(vec![$el; 100].into_iter().collect::<BinaryHeap<_>>(), $t);
            test_binary_heap!(vec![$el; 1000].into_iter().collect::<BinaryHeap<_>>(), $t);
            test_binary_heap!(vec![$el; 10000].into_iter().collect::<BinaryHeap<_>>(), $t);
        }
    };
}

test_binary_heaps!(test_binary_heap_u8, 100u8, u8);
test_binary_heaps!(test_binary_heap_i8, 100i8, i8);
test_binary_heaps!(test_binary_heap_u32, 1000000000u32, u32);
test_binary_heaps!(test_binary_heap_string, "a".to_string(), String);

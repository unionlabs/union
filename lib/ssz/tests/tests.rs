use ssz::{Decode, DecodeError, Encode};

macro_rules! list {
    ($($tt:tt)*) => {
        vec![$($tt)*].try_into().unwrap()
    };
}

mod round_trip {
    use ssz::types::{
        typenum::{U10, U4, U8},
        VariableList,
    };

    use super::*;

    fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(
        items: impl IntoIterator<Item = T>,
    ) {
        for item in items {
            let encoded = &item.as_ssz_bytes();
            assert_eq!(item.ssz_bytes_len(), encoded.len());
            assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
        }
    }

    #[test]
    fn bool() {
        round_trip([true, false]);
    }

    #[test]
    fn option_u16() {
        round_trip([None, Some(2u16)]);
    }

    #[test]
    fn u8_array_4() {
        round_trip([[0_u8, 0, 0, 0], [1, 0, 0, 0], [1, 2, 3, 4], [1, 2, 0, 4]]);
    }

    // #[test]
    // fn h256() {
    //     round_trip([H256::zero(), H256::from([1; 32]), H256::random()]);
    // }

    // #[test]
    // fn vec_of_h256() {
    //     round_trip::<VariableList<H256, U4>>([
    //         list![],
    //         list![H256::zero(), H256::from([1; 32]), H256::random()],
    //     ]);
    // }

    // #[test]
    // fn option_vec_h256() {
    //     round_trip::<Option<VariableList<H256, U3>>>([
    //         None,
    //         Some(list![]),
    //         Some(list![H256::zero(), H256::from([1; 32]), H256::random()]),
    //     ]);
    // }

    #[test]
    fn vec_u16() {
        round_trip::<VariableList<u16, typenum::U64>>(vec![
            list![],
            list![255],
            list![0, 1, 2],
            list![100; 64],
            list![255, 0, 255],
        ]);
    }

    #[test]
    fn vec_of_vec_u16() {
        round_trip::<VariableList<VariableList<u16, U10>, U10>>(vec![
            list![],
            list![list![]],
            list![list![1, 2, 3]],
            list![list![], list![]],
            list![list![], list![1, 2, 3]],
            list![list![1, 2, 3], list![1, 2, 3]],
            list![list![1, 2, 3], list![], list![1, 2, 3]],
            list![list![], list![], list![1, 2, 3]],
            list![list![], list![1], list![1, 2, 3]],
            list![list![], list![1], list![1, 2, 3]],
        ]);
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct FixedLen {
        a: u16,
        b: u64,
        c: u32,
    }

    #[test]
    #[allow(clippy::zero_prefixed_literal)]
    fn fixed_len_struct_encoding() {
        let items: Vec<FixedLen> = vec![
            FixedLen { a: 0, b: 0, c: 0 },
            FixedLen { a: 1, b: 1, c: 1 },
            FixedLen { a: 1, b: 0, c: 1 },
        ];

        let expected_encodings = [
            //  | u16--| u64----------------------------| u32----------|
            vec![00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00],
            vec![01, 00, 01, 00, 00, 00, 00, 00, 00, 00, 01, 00, 00, 00],
            vec![01, 00, 00, 00, 00, 00, 00, 00, 00, 00, 01, 00, 00, 00],
        ];

        for i in 0..items.len() {
            assert_eq!(
                items[i].as_ssz_bytes(),
                expected_encodings[i],
                "Failed on {}",
                i
            );
        }
    }

    #[test]
    fn fixed_len_excess_bytes() {
        let fixed = FixedLen { a: 1, b: 2, c: 3 };

        let mut bytes = fixed.as_ssz_bytes();
        bytes.append(&mut vec![0]);

        assert_eq!(
            FixedLen::from_ssz_bytes(&bytes),
            Err(DecodeError::InvalidByteLength {
                len: 15,
                expected: 14,
            })
        );
    }

    #[test]
    fn vec_of_fixed_len_struct() {
        let items: Vec<FixedLen> = vec![
            FixedLen { a: 0, b: 0, c: 0 },
            FixedLen { a: 1, b: 1, c: 1 },
            FixedLen { a: 1, b: 0, c: 1 },
        ];

        round_trip(items);
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct VariableLen {
        a: u16,
        b: VariableList<u16, U4>,
        c: u32,
    }

    #[test]
    #[allow(clippy::zero_prefixed_literal)]
    fn offset_into_fixed_bytes() {
        let bytes = vec![
            //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15
            //      | offset        | u32           | variable
            01, 00, 09, 00, 00, 00, 01, 00, 00, 00, 00, 00, 01, 00, 02, 00,
        ];

        assert_eq!(
            VariableLen::from_ssz_bytes(&bytes),
            Err(DecodeError::OffsetIntoFixedPortion(9))
        );
    }

    #[test]
    fn variable_len_excess_bytes() {
        let variable = VariableLen {
            a: 1,
            b: list![2],
            c: 3,
        };

        let mut bytes = variable.as_ssz_bytes();
        bytes.append(&mut vec![0]);

        // The error message triggered is not so helpful, it's caught by a side-effect. Just
        // checking there is _some_ error is fine.
        assert!(VariableLen::from_ssz_bytes(&bytes).is_err());
    }

    #[test]
    #[allow(clippy::zero_prefixed_literal)]
    fn first_offset_skips_byte() {
        let bytes = vec![
            //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15
            //      | offset        | u32           | variable
            01, 00, 11, 00, 00, 00, 01, 00, 00, 00, 00, 00, 01, 00, 02, 00,
        ];

        assert_eq!(
            VariableLen::from_ssz_bytes(&bytes),
            Err(DecodeError::OffsetSkipsVariableBytes(11))
        );
    }

    #[test]
    #[allow(clippy::zero_prefixed_literal)]
    fn variable_len_struct_encoding() {
        let items: Vec<VariableLen> = vec![
            VariableLen {
                a: 0,
                b: list![],
                c: 0,
            },
            VariableLen {
                a: 1,
                b: list![0],
                c: 1,
            },
            VariableLen {
                a: 1,
                b: list![0, 1, 2],
                c: 1,
            },
        ];

        let expected_encodings = [
            //   00..................................09
            //  | u16--| vec offset-----| u32------------| vec payload --------|
            vec![00, 00, 10, 00, 00, 00, 00, 00, 00, 00],
            vec![01, 00, 10, 00, 00, 00, 01, 00, 00, 00, 00, 00],
            vec![
                01, 00, 10, 00, 00, 00, 01, 00, 00, 00, 00, 00, 01, 00, 02, 00,
            ],
        ];

        for i in 0..items.len() {
            assert_eq!(
                items[i].as_ssz_bytes(),
                expected_encodings[i],
                "Failed on {}",
                i
            );
        }
    }

    #[test]
    fn vec_of_variable_len_struct() {
        let items: Vec<VariableLen> = vec![
            VariableLen {
                a: 0,
                b: list![],
                c: 0,
            },
            VariableLen {
                a: 255,
                b: list![0, 1, 2, 3],
                c: 99,
            },
            VariableLen {
                a: 255,
                b: list![0],
                c: 99,
            },
            VariableLen {
                a: 50,
                b: list![0],
                c: 0,
            },
        ];

        round_trip(items);
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct ThreeVariableLen {
        a: u16,
        b: VariableList<u16, U4>,
        c: VariableList<u16, U4>,
        d: VariableList<u16, U4>,
    }

    #[test]
    fn three_variable_len() {
        let vec: Vec<ThreeVariableLen> = vec![ThreeVariableLen {
            a: 42,
            b: list![0],
            c: list![1],
            d: list![2],
        }];

        round_trip(vec);
    }

    #[test]
    #[allow(clippy::zero_prefixed_literal)]
    fn offsets_decreasing() {
        let bytes = vec![
            //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15
            //      | offset        | offset        | offset        | variable
            01, 00, 14, 00, 00, 00, 15, 00, 00, 00, 14, 00, 00, 00, 00, 00,
        ];

        assert_eq!(
            ThreeVariableLen::from_ssz_bytes(&bytes),
            Err(DecodeError::OffsetsAreDecreasing(14))
        );
    }

    #[test]
    fn tuple_u8_u16() {
        let vec: Vec<(u8, u16)> = vec![
            (0, 0),
            (0, 1),
            (1, 0),
            (u8::MAX, u16::MAX),
            (0, u16::MAX),
            (u8::MAX, 0),
            (42, 12301),
        ];

        round_trip(vec);
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn tuple_vec_vec() {
        let vec: Vec<(
            u64,
            VariableList<u8, U8>,
            VariableList<VariableList<u16, U8>, U8>,
        )> = vec![
            (0, list![], list![list![]]),
            (99, list![101], list![list![], list![]]),
            (
                42,
                list![12, 13, 14],
                list![list![99, 98, 97, 96], list![42, 44, 46, 48, 50]],
            ),
        ];

        round_trip(vec);
    }

    // #[test]
    // fn btree_map_fixed() {
    //     let data = vec![
    //         BTreeMap::new(),
    //         BTreeMap::from_iter(vec![(0u8, 0u16), (1, 2), (2, 4), (4, 6)]),
    //     ];
    //     round_trip(data);
    // }

    // #[test]
    // fn btree_map_variable_value() {
    //     let data = vec![
    //         BTreeMap::new(),
    //         BTreeMap::from_iter(vec![
    //             (
    //                 0u64,
    //                 ThreeVariableLen {
    //                     a: 1,
    //                     b: vec![3, 5, 7],
    //                     c: vec![],
    //                     d: vec![0, 0],
    //                 },
    //             ),
    //             (
    //                 1,
    //                 ThreeVariableLen {
    //                     a: 99,
    //                     b: vec![1],
    //                     c: vec![2, 3, 4, 5, 6, 7, 8, 9, 10],
    //                     d: vec![4, 5, 6, 7, 8],
    //                 },
    //             ),
    //             (
    //                 2,
    //                 ThreeVariableLen {
    //                     a: 0,
    //                     b: vec![],
    //                     c: vec![],
    //                     d: vec![],
    //                 },
    //             ),
    //         ]),
    //     ];
    //     round_trip(data);
    // }
}

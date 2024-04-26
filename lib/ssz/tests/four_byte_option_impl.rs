use ssz::{
    types::{
        typenum::{U32, U8},
        VariableList,
    },
    Decode, Encode,
};

type VariableListU16 = VariableList<u16, U32>;

ssz::four_byte_option_impl!(impl_u16, u16);
ssz::four_byte_option_impl!(impl_var_list_u16, VariableListU16);

#[test]
fn ssz_encode_option_u16() {
    let item = Some(65535_u16);
    let bytes = vec![1, 0, 0, 0, 255, 255];
    assert_eq!(impl_u16::encode::as_ssz_bytes(&item), bytes);
    assert_eq!(impl_u16::decode::from_ssz_bytes(&bytes).unwrap(), item);

    let item = None;
    let bytes = vec![0, 0, 0, 0];
    assert_eq!(impl_u16::encode::as_ssz_bytes(&item), bytes);
    assert_eq!(impl_u16::decode::from_ssz_bytes(&bytes).unwrap(), None);
}

#[test]
fn ssz_encode_option_vec_u16() {
    let item = Some(vec![0_u16, 1].try_into().unwrap());
    let bytes = vec![1, 0, 0, 0, 0, 0, 1, 0];
    assert_eq!(impl_var_list_u16::encode::as_ssz_bytes(&item), bytes);
    assert_eq!(
        impl_var_list_u16::decode::from_ssz_bytes(&bytes).unwrap(),
        item
    );

    let item = None;
    let bytes = vec![0, 0, 0, 0];
    assert_eq!(impl_var_list_u16::encode::as_ssz_bytes(&item), bytes);
    assert_eq!(
        impl_var_list_u16::decode::from_ssz_bytes(&bytes).unwrap(),
        item
    );
}

fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(items: Vec<T>) {
    for item in items {
        let encoded = &item.as_ssz_bytes();
        dbg!(&item);
        println!("{:?}", &encoded);
        // assert_eq!(item.ssz_bytes_len(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct TwoVariableLenOptions {
    a: u16,
    #[ssz(with = "impl_u16")]
    b: Option<u16>,
    #[ssz(with = "impl_var_list_u16")]
    c: Option<VariableListU16>,
    #[ssz(with = "impl_var_list_u16")]
    d: Option<VariableListU16>,
}

#[allow(clippy::zero_prefixed_literal)]
#[test]
fn two_variable_len_options_encoding() {
    let s = TwoVariableLenOptions {
        a: 42,
        b: None,
        c: Some(vec![0].try_into().unwrap()),
        d: None,
    };

    let bytes = vec![
        //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15  16  17  18  19  20  21
        //      | option<u16>   | offset        | offset        | option<u16    | 1st list
        42, 00, 14, 00, 00, 00, 18, 00, 00, 00, 24, 00, 00, 00, 00, 00, 00, 00, 01, 00, 00, 00,
        //  23  24  25  26  27
        //      | 2nd list
        00, 00, 00, 00, 00, 00,
    ];

    assert_eq!(s.as_ssz_bytes(), bytes);
}

#[test]
fn two_variable_len_options_round_trip() {
    let vec: Vec<TwoVariableLenOptions> = vec![
        TwoVariableLenOptions {
            a: 42,
            b: Some(12),
            c: Some(vec![0].try_into().unwrap()),
            d: Some(vec![1].try_into().unwrap()),
        },
        TwoVariableLenOptions {
            a: 42,
            b: Some(12),
            c: Some(vec![0].try_into().unwrap()),
            d: None,
        },
        TwoVariableLenOptions {
            a: 42,
            b: None,
            c: Some(vec![0].try_into().unwrap()),
            d: None,
        },
        TwoVariableLenOptions {
            a: 42,
            b: None,
            c: None,
            d: None,
        },
    ];

    round_trip(vec);
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
        (
            0,
            vec![].try_into().unwrap(),
            vec![vec![].try_into().unwrap()].try_into().unwrap(),
        ),
        (
            99,
            vec![101].try_into().unwrap(),
            vec![vec![].try_into().unwrap(), vec![].try_into().unwrap()]
                .try_into()
                .unwrap(),
        ),
        (
            42,
            vec![12, 13, 14].try_into().unwrap(),
            vec![
                vec![99, 98, 97, 96].try_into().unwrap(),
                vec![42, 44, 46, 48, 50].try_into().unwrap(),
            ]
            .try_into()
            .unwrap(),
        ),
    ];

    round_trip(vec);
}

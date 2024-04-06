use ssz::{
    types::{typenum::U4, VariableList},
    Decode, DecodeError, Encode, SszDecoderBuilder, SszEncoder,
};

#[derive(Debug, PartialEq)]
pub struct Foo {
    a: u16,
    b: VariableList<u16, U4>,
    c: u16,
}

impl Encode for Foo {
    fn is_ssz_fixed_len() -> bool {
        <u16 as Encode>::is_ssz_fixed_len() && <VariableList<u16, U4> as Encode>::is_ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        <u16 as Encode>::ssz_fixed_len()
            + ssz::BYTES_PER_LENGTH_OFFSET
            + <u16 as Encode>::ssz_fixed_len()
            + self.b.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let offset = <u16 as Encode>::ssz_fixed_len()
            + <VariableList<u16, U4> as Encode>::ssz_fixed_len()
            + <u16 as Encode>::ssz_fixed_len();

        let mut encoder = SszEncoder::container(buf, offset);

        encoder.append(&self.a);
        encoder.append(&self.b);
        encoder.append(&self.c);

        encoder.finalize();
    }
}

impl Decode for Foo {
    fn is_ssz_fixed_len() -> bool {
        <u16 as Decode>::is_ssz_fixed_len() && <VariableList<u16, U4> as Decode>::is_ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut builder = SszDecoderBuilder::new(bytes);

        builder.register_type::<u16>()?;
        builder.register_type::<VariableList<u16, U4>>()?;
        builder.register_type::<u16>()?;

        let mut decoder = builder.build()?;

        Ok(Self {
            a: decoder.decode_next()?,
            b: decoder.decode_next()?,
            c: decoder.decode_next()?,
        })
    }
}

fn main() {
    let my_foo = Foo {
        a: 42,
        b: vec![0, 1, 2, 3].try_into().unwrap(),
        c: 11,
    };

    #[rustfmt::skip]
    let bytes = vec![
        // a
        42, 0,
        // offset for b
        8, 0, 0, 0,
        // c
        11, 0,
        // b
        0, 0, 1, 0, 2, 0, 3, 0 
    ];

    assert_eq!(my_foo.as_ssz_bytes(), bytes);

    let decoded_foo = Foo::from_ssz_bytes(&bytes).unwrap();

    assert_eq!(my_foo, decoded_foo);
}

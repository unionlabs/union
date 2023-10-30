use enumset::*;

#[derive(EnumSetType)]
#[repr(usize)]
enum BadRepr {
    Variant,
}

#[derive(EnumSetType)]
#[repr(usize)]
enum GenericEnum<T> {
    Variant,
    FieldBlah(T),
}

#[derive(EnumSetType)]
struct BadItemType {

}

#[derive(EnumSetType)]
#[enumset(repr = "u8", repr = "u16")]
enum MultipleReprs {
    Variant,
}

#[derive(EnumSetType)]
#[enumset(repr = "abcdef")]
enum InvalidRepr {
    Variant,
}

#[derive(EnumSetType)]
#[enumset(serialize_repr = "abcdef")]
enum InvalidSerdeRepr {
    Variant,
}

fn main() {}
use enumset::*;

#[derive(EnumSetType)]
enum NegativeVariant {
    Variant = -1,
}

#[derive(EnumSetType)]
enum HasFields {
    Variant(u32),
}

#[derive(EnumSetType)]
#[enumset(repr = "u128")]
enum BadExplicitRepr {
    Variant = 128,
}

#[derive(EnumSetType)]
#[enumset(serialize_repr = "u8")]
enum BadSerializationRepr {
    Variant = 8,
}

#[derive(EnumSetType)]
enum NonLiteralRepr {
    Variant = 1 + 1,
}

#[derive(EnumSetType)]
#[enumset(repr = "u16")]
enum BadMemRepr {
    Variant = 16,
}

#[derive(EnumSetType)]
enum ExcessiveReprSize {
    Variant = 0xFFFFFFD0,
}

fn main() {}

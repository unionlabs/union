use enumset::*;

#[derive(EnumSetType)]
enum OkayEnumButCantUseFromRepr {
    Variant,
}

#[derive(EnumSetType)]
#[enumset(repr = "array")]
enum OkayEnumButCantUseFromReprArray {
    Variant,
}

fn main() {
    EnumSet::<OkayEnumButCantUseFromRepr>::from_repr(1);
    EnumSet::<OkayEnumButCantUseFromReprArray>::from_repr([1]);
}

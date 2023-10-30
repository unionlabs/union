use enumset::*;

#[derive(EnumSetType, Debug)]
#[enumset(repr = "u16")]
enum ReprEnum {
    A, B, C, D, E, F, G, H,
}

#[test]
fn test() {
    let mut set = EnumSet::<ReprEnum>::new();
    set.insert(ReprEnum::B);
    set.insert(ReprEnum::F);

    let repr: u16 = set.as_repr();
    assert_eq!(
        (1 << 1) | (1 << 5),
        repr,
    );

    let set2 = unsafe { EnumSet::<ReprEnum>::from_repr_unchecked(repr) };
    assert_eq!(set, set2);
}

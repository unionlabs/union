/// Crate internal trait for all our signed and unsigned number types
pub(crate) trait NumConsts {
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;
    const ONE: Self;
}

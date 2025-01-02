#[cfg(feature = "primitive-types-compat")]
impl From<crate::H256> for primitive_types::H256 {
    fn from(value: crate::H256) -> Self {
        Self(*value.get())
    }
}

#[cfg(feature = "primitive-types-compat")]
impl From<primitive_types::H256> for crate::H256 {
    fn from(value: primitive_types::H256) -> Self {
        Self::new(value.0)
    }
}

#[cfg(feature = "primitive-types-compat")]
impl From<crate::H160> for primitive_types::H160 {
    fn from(value: crate::H160) -> Self {
        Self(*value.get())
    }
}

#[cfg(feature = "primitive-types-compat")]
impl From<primitive_types::H160> for crate::H160 {
    fn from(value: primitive_types::H160) -> Self {
        Self::new(value.0)
    }
}

#[cfg(feature = "generic-array-compat")]
impl<E: crate::encoding::Encoding, const BYTES: usize>
    From<generic_array::GenericArray<u8, typenum::U<BYTES>>> for crate::Hash<BYTES, E>
where
    typenum::Const<BYTES>: typenum::ToUInt<Output: generic_array::ArrayLength<u8>>,
{
    fn from(arr: generic_array::GenericArray<u8, typenum::U<BYTES>>) -> Self {
        Self::new(
            arr.to_vec()
                .try_into()
                .expect("GenericArray has the correct length; qed;"),
        )
    }
}

#[cfg(feature = "generic-array-compat")]
impl<E: crate::encoding::Encoding, const BYTES: usize> From<crate::Hash<BYTES, E>>
    for generic_array::GenericArray<u8, typenum::U<BYTES>>
where
    typenum::Const<BYTES>: typenum::ToUInt<Output: generic_array::ArrayLength<u8>>,
{
    fn from(arr: crate::Hash<BYTES, E>) -> Self {
        generic_array::GenericArray::<u8, typenum::U<BYTES>>::from_slice(arr.get()).to_owned()
    }
}

#[cfg(feature = "alloy-primitives-compat")]
impl<EBytes: Encoding> TryFrom<crate::Bytes<EBytes>> for alloy::core::primitives::Address {
    type Error = crate::hash::FixedBytesError;

    fn try_from(value: Bytes<EBytes>) -> Result<Self, Self::Error> {
        <crate::hash::H160>::try_from(value).map(Self::from)
    }
}

#[cfg(feature = "alloy-primitives-compat")]
impl<EBytes: Encoding> From<alloy::core::primitives::Address> for crate::Bytes<EBytes> {
    fn from(value: alloy::core::primitives::Address) -> Self {
        value.0 .0.into()
    }
}

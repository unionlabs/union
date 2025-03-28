#![allow(clippy::module_name_repetitions)]

#[cfg(feature = "primitive-types-compat")]
pub mod primitive_types_compat {
    use crate::{H160, H256};

    impl From<H256> for primitive_types::H256 {
        fn from(value: H256) -> Self {
            Self(*value.get())
        }
    }

    impl From<primitive_types::H256> for H256 {
        fn from(value: primitive_types::H256) -> Self {
            Self::new(value.0)
        }
    }

    impl From<H160> for primitive_types::H160 {
        fn from(value: H160) -> Self {
            Self(*value.get())
        }
    }

    impl From<primitive_types::H160> for H160 {
        fn from(value: primitive_types::H160) -> Self {
            Self::new(value.0)
        }
    }
}

#[cfg(feature = "generic-array-compat")]
pub mod generic_array_compat {
    use generic_array::{ArrayLength, GenericArray};
    use typenum::{Const, ToUInt, U};

    use crate::encoding::Encoding;

    impl<E: Encoding, const BYTES: usize> From<GenericArray<u8, U<BYTES>>>
        for crate::FixedBytes<BYTES, E>
    where
        Const<BYTES>: ToUInt<Output: ArrayLength<u8>>,
    {
        fn from(arr: GenericArray<u8, U<BYTES>>) -> Self {
            Self::new(
                arr.to_vec()
                    .try_into()
                    .expect("GenericArray has the correct length; qed;"),
            )
        }
    }

    impl<E: Encoding, const BYTES: usize> From<crate::FixedBytes<BYTES, E>>
        for GenericArray<u8, U<BYTES>>
    where
        Const<BYTES>: ToUInt<Output: ArrayLength<u8>>,
    {
        fn from(arr: crate::FixedBytes<BYTES, E>) -> Self {
            GenericArray::<u8, U<BYTES>>::from_slice(arr.get()).to_owned()
        }
    }
}

#[cfg(feature = "alloy-primitives-compat")]
pub mod alloy_primitives_compat {
    use crate::{encoding::Encoding, Bytes, FixedBytes, H160, U256};

    impl<EBytes: Encoding> TryFrom<Bytes<EBytes>> for alloy_primitives::Address {
        type Error = crate::fixed_bytes::FixedBytesError;

        fn try_from(value: Bytes<EBytes>) -> Result<Self, Self::Error> {
            <H160>::try_from(value).map(Self::from)
        }
    }

    impl<E: Encoding> From<alloy_primitives::Address> for Bytes<E> {
        fn from(value: alloy_primitives::Address) -> Self {
            value.0 .0.into()
        }
    }

    impl<E: Encoding> From<alloy_primitives::Address> for H160<E> {
        fn from(value: alloy_primitives::Address) -> Self {
            value.0 .0.into()
        }
    }

    impl<E: Encoding> From<H160<E>> for alloy_primitives::Address {
        fn from(value: H160<E>) -> Self {
            value.get().into()
        }
    }

    impl<E: Encoding, const BYTES: usize> From<alloy_primitives::FixedBytes<BYTES>>
        for FixedBytes<BYTES, E>
    {
        fn from(value: alloy_primitives::FixedBytes<BYTES>) -> Self {
            value.0.into()
        }
    }

    impl<E: Encoding, const BYTES: usize> From<FixedBytes<BYTES, E>>
        for alloy_primitives::FixedBytes<BYTES>
    {
        fn from(value: FixedBytes<BYTES, E>) -> Self {
            value.get().into()
        }
    }

    impl<E: Encoding> From<alloy_primitives::Bytes> for Bytes<E> {
        fn from(value: alloy_primitives::Bytes) -> Self {
            value.to_vec().into()
        }
    }

    impl<E: Encoding> From<Bytes<E>> for alloy_primitives::Bytes {
        fn from(value: Bytes<E>) -> Self {
            value.to_vec().into()
        }
    }

    impl From<alloy_primitives::U256> for U256 {
        fn from(value: alloy_primitives::U256) -> Self {
            U256::from_be_bytes(value.to_be_bytes())
        }
    }

    impl From<U256> for alloy_primitives::U256 {
        fn from(value: U256) -> Self {
            alloy_primitives::U256::from_be_bytes(value.to_be_bytes())
        }
    }
}

#[cfg(feature = "alloy-sol-types-compat")]
pub mod alloy_sol_types_compat {
    use alloy_sol_types::{
        abi::token::WordToken,
        private::SolTypeValue,
        sol_data::{ByteCount, SupportedFixedBytes},
        SolType, SolValue, Word,
    };

    use crate::{encoding::Encoding, Bytes, FixedBytes};

    impl<E: Encoding> SolValue for Bytes<E> {
        type SolType = <alloy_primitives::Bytes as SolValue>::SolType;
    }

    impl<E: Encoding, const BYTES: usize> SolValue for FixedBytes<BYTES, E>
    where
        ByteCount<BYTES>: SupportedFixedBytes,
    {
        type SolType = SolFixedBytes<BYTES>;
    }

    #[derive(Clone, Copy, Debug)]
    pub struct SolFixedBytes<const BYTES: usize>;

    impl<E: Encoding, const BYTES: usize> SolTypeValue<SolFixedBytes<BYTES>> for FixedBytes<BYTES, E>
    where
        ByteCount<BYTES>: SupportedFixedBytes,
    {
        #[inline]
        fn stv_to_tokens(&self) -> <SolFixedBytes<BYTES> as SolType>::Token<'_> {
            let mut word = Word::ZERO;
            word[..BYTES].copy_from_slice(self.get());
            word.into()
        }

        #[inline]
        fn stv_eip712_data_word(&self) -> Word {
            SolTypeValue::<SolFixedBytes<BYTES>>::stv_to_tokens(self).0
        }

        #[inline]
        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            out.extend_from_slice(self.get().as_slice());
        }
    }

    impl<E: Encoding, const BYTES: usize> SolTypeValue<SolFixedBytes<BYTES>> for &FixedBytes<BYTES, E>
    where
        ByteCount<BYTES>: SupportedFixedBytes,
    {
        #[inline]
        fn stv_to_tokens(&self) -> <SolFixedBytes<BYTES> as SolType>::Token<'_> {
            <FixedBytes<BYTES, E> as SolTypeValue<SolFixedBytes<BYTES>>>::stv_to_tokens(self)
        }

        #[inline]
        fn stv_eip712_data_word(&self) -> Word {
            <FixedBytes<BYTES, E> as SolTypeValue<SolFixedBytes<BYTES>>>::stv_eip712_data_word(self)
        }

        #[inline]
        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            <FixedBytes<BYTES, E> as SolTypeValue<SolFixedBytes<BYTES>>>::stv_abi_encode_packed_to(
                self, out,
            );
        }
    }

    impl<const N: usize> SolType for SolFixedBytes<N>
    where
        ByteCount<N>: SupportedFixedBytes,
    {
        type RustType = FixedBytes<N>;
        type Token<'a> = WordToken;

        const SOL_NAME: &'static str = <ByteCount<N>>::NAME;
        const ENCODED_SIZE: Option<usize> = Some(32);
        const PACKED_ENCODED_SIZE: Option<usize> = Some(N);

        #[inline]
        fn valid_token(token: &Self::Token<'_>) -> bool {
            alloy_sol_types::sol_data::FixedBytes::valid_token(token)
        }

        #[inline]
        fn detokenize(token: Self::Token<'_>) -> Self::RustType {
            token.0[..N].try_into().unwrap()
        }
    }
}

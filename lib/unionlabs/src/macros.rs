macro_rules! hex_string_array_wrapper {
    (
        $(
            $(#[doc = $doc:literal])*
            pub struct $Struct:ident(pub [u8; $N:expr]);
        )+
    ) => {
        $(
            $(#[doc = $doc])*
            #[derive(
                Copy,
                Clone,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                ::ssz::Ssz,
                ::serde::Serialize,
                ::serde::Deserialize,
                Hash
            )]
            #[ssz(transparent)]
            #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
            pub struct $Struct(#[serde(with = "::serde_utils::hex_string")] pub [u8; $N]);

            impl $Struct {
                pub const BYTES_LEN: usize = $N;

                #[doc = concat!("The [`Display`](core::fmt::Display) impl for [`", stringify!($Struct), "`]")]
                /// prefixes the output with `0x`, which may not be desirable in all contexts.
                /// This fn serves as a convenience around [`hex::encode(&self)`](hex::encode).
                #[must_use]
                pub fn to_string_unprefixed(&self) -> String {
                    hex::encode(&self)
                }

                pub fn iter(&self) -> core::slice::Iter<u8> {
                    (&self).into_iter()
                }
            }

            impl core::str::FromStr for $Struct {
                type Err = serde_utils::FromHexStringError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    serde_utils::parse_hex(s)
                }
            }

            impl Default for $Struct {
                fn default() -> Self {
                    Self([0_u8; $N])
                }
            }

            impl<'a> IntoIterator for &'a $Struct {
                type Item = &'a u8;
                type IntoIter = core::slice::Iter<'a, u8>;

                fn into_iter(self) -> core::slice::Iter<'a, u8> {
                    self.0.iter()
                }
            }

            impl IntoIterator for $Struct {
                type Item = u8;
                type IntoIter = core::array::IntoIter<u8, $N>;

                fn into_iter(self) -> Self::IntoIter {
                    self.0.into_iter()
                }
            }

            impl TryFrom<Vec<u8>> for $Struct {
                type Error = crate::errors::InvalidLength;

                fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
                    value
                        .try_into()
                        .map(Self)
                        .map_err(|invalid| crate::errors::InvalidLength {
                            expected: crate::errors::ExpectedLength::Exact($N),
                            found: invalid.len(),
                        })
                }
            }

            impl TryFrom<&Vec<u8>> for $Struct {
                type Error = crate::errors::InvalidLength;

                fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
                    value
                        .as_slice()
                        .try_into()
                        .map(Self)
                        .map_err(|_| crate::errors::InvalidLength {
                            expected: crate::errors::ExpectedLength::Exact($N),
                            found: value.len(),
                        })
                }
            }

            impl TryFrom<&[u8]> for $Struct {
                type Error = crate::errors::InvalidLength;

                fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                    value
                        .try_into()
                        .map(Self)
                        .map_err(|_| crate::errors::InvalidLength {
                            expected: crate::errors::ExpectedLength::Exact($N),
                            found: value.len(),
                        })
                }
            }

            impl From<$Struct> for Vec<u8> {
                fn from(value: $Struct) -> Self {
                    value.0.into()
                }
            }

            impl From<$Struct> for [u8; $N] {
                fn from(value: $Struct) -> Self {
                    value.0
                }
            }

            impl From<[u8; $N]> for $Struct {
                fn from(value: [u8; $N]) -> Self {
                    Self(value)
                }
            }

            impl ::core::fmt::Debug for $Struct {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{}({self})", stringify!($Struct))
                }
            }

            impl ::core::fmt::Display for $Struct {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "0x{}", hex::encode(self.0).as_str())
                }
            }

            impl From<::generic_array::GenericArray<u8, ::typenum::U<$N>>> for $Struct {
                fn from(arr: ::generic_array::GenericArray<u8, ::typenum::U<$N>>) -> Self {
                    Self(arr.to_vec().try_into().expect("GenericArray has the correct length; qed;"))
                }
            }

            impl From<$Struct> for ::generic_array::GenericArray<u8, ::typenum::U<$N>> {
                fn from(arr: $Struct) -> Self {
                    ::generic_array::GenericArray::<u8, ::typenum::U<$N>>::from_slice(&arr.0).to_owned()
                }
            }

            #[cfg(feature = "ethabi")]
            impl From<$Struct> for ::ethers_core::types::Bytes {
                fn from(value: $Struct) -> Self {
                    ::ethers_core::types::Bytes::from(value.0)
                }
            }

            #[cfg(feature = "ethabi")]
            impl TryFrom<::ethers_core::types::Bytes> for $Struct {
                type Error = <Self as TryFrom<Vec<u8>>>::Error;

                fn try_from(value: ::ethers_core::types::Bytes) -> Result<Self, Self::Error> {
                    Self::try_from(&value.0[..])
                }
            }

            #[cfg(feature = "ethabi")]
            impl TryFrom<&'_ ::ethers_core::types::Bytes> for $Struct {
                type Error = <Self as TryFrom<Vec<u8>>>::Error;

                fn try_from(value: &::ethers_core::types::Bytes) -> Result<Self, Self::Error> {
                    Self::try_from(&value.0[..])
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiType for $Struct {
                fn param_type() -> ::ethers_core::abi::ParamType {
                    ::ethers_core::abi::ParamType::FixedBytes($N)
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiArrayType for $Struct {}

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiEncode for $Struct {
                fn encode(self) -> Vec<u8> {
                    self.0.encode()
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiDecode for $Struct {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ::ethers_core::abi::AbiError> {
                    <[u8; $N]>::decode(bytes).map(Self)
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::Tokenizable for $Struct {
                fn from_token(token: ::ethers_core::abi::Token) -> Result<Self, ::ethers_core::abi::InvalidOutputType> {
                    <[u8; $N]>::from_token(token).map(Self)
                }

                fn into_token(self) -> ::ethers_core::abi::Token {
                    self.0.into_token()
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::TokenizableItem for $Struct {}

            impl AsRef<[u8]> for $Struct {
                fn as_ref(&self) -> &[u8] {
                    &self.0
                }
            }

            impl ::rlp::Encodable for $Struct {
                fn rlp_append(&self, s: &mut ::rlp::RlpStream) {
                    s.encoder().encode_value(self.as_ref());
                }
            }

            impl ::rlp::Decodable for $Struct {
                fn decode(rlp: &rlp::Rlp) -> Result<Self, ::rlp::DecoderError> {
                    rlp.decoder()
                        .decode_value(|bytes| match bytes.len().cmp(&$N) {
                            ::core::cmp::Ordering::Less => Err(::rlp::DecoderError::RlpIsTooShort),
                            ::core::cmp::Ordering::Greater => Err(::rlp::DecoderError::RlpIsTooBig),
                            ::core::cmp::Ordering::Equal => {
                                Ok($Struct(bytes.try_into().expect("size is checked; qed;")))
                            }
                        })
                }
            }
        )+
    };
}

pub(crate) use hex_string_array_wrapper;

macro_rules! wrapper_enum {
    (
        #[model(proto($Proto:ty))]
        $(#[$meta:meta])*
        // NOTE: Currently there are no ethabi generated enums; use this if/when there are any.
        // $(#[ethabi($EthAbi:ty)])?
        pub enum $Enum:ident {
            $(
                $(#[$inner_meta:meta])*
                $Variant:ident = $discriminant:literal,
            )+
        }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        $(#[$meta])*
        #[cfg_attr(feature = "arbitrary", derive(::arbitrary::Arbitrary))]
        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        pub enum $Enum {
            $(
                $(#[$inner_meta])*
                $Variant = $discriminant,
            )+
        }

        mod ensure_enum_values_are_same_as_proto {
            $(
                #[allow(non_upper_case_globals, dead_code)]
                const $Variant: () = assert!(
                    super::$Enum::$Variant as i32 == <$Proto>::$Variant as i32,
                );
            )+
        }

        impl core::str::FromStr for $Enum {
            type Err = crate::errors::UnknownEnumVariant<String>;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if s == <$Proto>::$Variant.as_str_name() {
                        Ok($Enum::$Variant)
                    } else
                )+
                {
                    Err(crate::errors::UnknownEnumVariant(s.to_string()))
                }
            }
        }

        impl From<$Enum> for &'static str {
            fn from(value: $Enum) -> Self {
                match value {
                    $(
                        $Enum::$Variant => <$Proto>::$Variant.as_str_name(),
                    )+
                }
            }
        }

        impl core::fmt::Display for $Enum {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(<&'static str>::from(*self))
            }
        }

        impl From<$Enum> for u8 {
            fn from(value: $Enum) -> Self {
                match value {
                    $(
                        $Enum::$Variant => $discriminant as u8,
                    )+
                }
            }
        }

        impl From<$Enum> for i32 {
            fn from(value: $Enum) -> Self {
                u8::from(value).into()
            }
        }

        impl TryFrom<u8> for $Enum {
            type Error = crate::errors::UnknownEnumVariant<u8>;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                i32::from(value)
                    .try_into()
                    .map_err(|_| crate::errors::UnknownEnumVariant(value))
            }
        }

        impl TryFrom<i32> for $Enum {
            type Error = crate::errors::UnknownEnumVariant<i32>;

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                <$Proto>::try_from(value)
                    .map_err(|_| crate::errors::UnknownEnumVariant(value))
                    .map(Into::into)
            }
        }

        impl From<$Proto> for $Enum {
            fn from(value: $Proto) -> Self {
                match value {
                    $(
                        <$Proto>::$Variant => $Enum::$Variant,
                    )+
                }
            }
        }

        impl From<$Enum> for $Proto {
            fn from(value: $Enum) -> Self {
                match value {
                    $(
                        $Enum::$Variant => Self::$Variant,
                    )+
                }
            }
        }
    };
}

pub(crate) use wrapper_enum;

// Useful in const contexts in place of `?`. Uncomment if you need it!
// macro_rules! option_try {
//     ($expr:expr) => {
//         match $expr {
//             Some(some) => some,
//             None => return None,
//         }
//     };
// }

// pub(crate) use option_try;

macro_rules! result_try {
    ($expr:expr$(, $map_err:expr)?) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => return Err($($map_err)?(err)),
        }
    };
}

pub(crate) use result_try;

// Useful in const contexts in place of `.unwrap()`
#[macro_export]
macro_rules! result_unwrap {
    ($expr:expr) => {{
        // assign to a const here so this can't be called in non-const contexts
        const _: () = match $expr {
            Ok(_) => {}
            Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
        };

        match $expr {
            Ok(ok) => ok,
            Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
        }
    }};
}

// Useful in const contexts in place of `.unwrap()`
#[macro_export]
macro_rules! option_unwrap {
    ($expr:expr) => {{
        // assign to a const here so this can't be called in non-const contexts
        const _: () = match $expr {
            Some(_) => {}
            None => panic!("called `Option::unwrap()` on an `None` value"),
        };

        match $expr {
            Some(some) => some,
            None => panic!("called `Option::unwrap()` on an `None` value"),
        }
    }};
}

/// Minimal version of <https://github.com/benluelo/advent-of-code/blob/18684af90a06eb594b07dee9b99593ce3f872641/rust/src/const_helpers.rs#L299-L369>. Check there if any other features are required.
#[macro_export]
macro_rules! iter {
    ($($label:lifetime:)? for ($i:ident, $item:pat) in enumerate($slice:expr)
        $body:block
    ) => {{
        let __slice = $slice;
        let mut __i = 0;
        $($label:)? while __i < __slice.len() {
            #[allow(clippy::toplevel_ref_arg)]
            let $item = __slice[__i];
            __i += 1;
            let $i = __i - 1;
            $body;
        }
    }};
}

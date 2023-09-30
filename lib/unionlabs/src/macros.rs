macro_rules! wrapper_enum {
    (
        #[proto($Proto:ty)]
        // NOTE: Currently there are no ethabi generated enums; use this if/when there are any.
        // $(#[ethabi($EthAbi:ty)])?
        pub enum $Enum:ident {
            $(
                $(#[doc = $doc:literal])*
                $Variant:ident = $discriminant:literal,
            )+
        }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub enum $Enum {
            $(
                $(#[doc = $doc])*
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

        impl std::str::FromStr for $Enum {
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

        impl std::fmt::Display for $Enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                <$Proto>::from_i32(value)
                    .ok_or(crate::errors::UnknownEnumVariant(value))
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

// Useful in const contexts in place of `.unwrap()`. Uncomment if you need it!
// macro_rules! result_unwrap {
//     ($expr:expr) => {{
//         // assign to a const here so this can't be called in non-const contexts
//         const _: () = match $expr {
//             Ok(_) => {}
//             Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
//         };

//         match $expr {
//             Ok(ok) => ok,
//             Err(err) => panic!("called `Result::unwrap()` on an `Err` value: {err:?}"),
//         }
//     }};
// }

// pub(crate) use result_unwrap;

/// Given an enum bound by one generic parameter, generate [`Debug`], [`PartialEq`], [`Clone`], [`Serialize`], and
/// [`Deserialize`] implementations without bounding the generic parameter.
#[macro_export]
macro_rules! generic_enum {
    (
        $(#[doc = $outer_doc:literal])*
        pub enum $Enum:ident<$generics:ident: $bound:ident> {
            $(
                $(#[doc = $doc:literal])*
                $Variant:ident$((
                    $(#[$variant_inner_meta:meta])*
                    $VariantInner:ty
                ))?,
            )+
        }
    ) => {
        $(#[doc = $outer_doc])*
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #[serde(bound(serialize = "", deserialize = ""))]
        #[allow(clippy::type_complexity)]
        pub enum $Enum<$generics: $bound> {
            $(
                $(#[doc = $doc])*
                $Variant$((
                    $(#[$variant_inner_meta])*
                    $VariantInner
                ))?,
            )+
        }

        impl<$generics: $bound> PartialEq for $Enum<$generics> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    $(
                        (Self::$Variant(this), Self::$Variant(other)) => this == other,
                    )+
                    _ => false,
                }
            }
        }

        impl<$generics: $bound> ::std::fmt::Debug for $Enum<$generics> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    $(
                        Self::$Variant(this) => f.debug_tuple(stringify!($Variant)).field(&this).finish(),
                    )+
                }
            }
        }

        impl<$generics: $bound> Clone for $Enum<$generics> {
            fn clone(&self) -> Self {
                match self {
                    $(
                        Self::$Variant(this) => Self::$Variant(this.clone()),
                    )+
                }
            }
        }
    };
}

pub use generic_enum;

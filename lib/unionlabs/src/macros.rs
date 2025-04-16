macro_rules! wrapper_enum {
    (
        #[model(proto($Proto:ty))]
        $(#[$meta:meta])*
        // NOTE: Currently there are no ethabi generated enums; use this if/when there are any.
        // $(#[ethabi($EthAbi:ty)])?
        pub enum $Enum:ident {
            $(
                $(#[$inner_meta:meta])*
                $Variant:ident = $discriminant:tt,
            )+
        }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        $(#[$meta])*
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

        impl $Enum {
            pub fn from_proto_str(s: &str) -> Option<Self> {
                <$Proto>::from_str_name(s).map(Into::into)
            }
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

#[macro_export]
macro_rules! assert_all_eq (
    ($a:expr, $b:expr) => {
        assert_eq!($a, $b);
    };
    ($a:expr, $b:expr, $c:expr) => {
        assert_eq!($a, $b);
        assert_eq!($b, $c);
    };
    ($a:expr, $b:expr, $c:expr, $($rest:expr),*$(,)?) => {
        assert_eq!($a, $b);
        assert_all_eq!($b, $c, $($rest),*);
    }
);

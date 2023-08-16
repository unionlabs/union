macro_rules! wrapper_enum {
    (
        #[proto($Proto:ty)]
        // NOTE: Currently there are no ethabi generated enums; use this if/when there are any.
        // $(#[ethabi($EthAbi:ty)])?
        pub enum $Enum:ident {
            $($Variant:ident = $discriminant:literal,)+
        }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub enum $Enum {
            $($Variant = $discriminant),+
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

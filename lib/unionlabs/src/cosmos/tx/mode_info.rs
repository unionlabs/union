use serde::{Deserialize, Serialize};

use crate::cosmos::{
    crypto::multisig::compact_bit_array::CompactBitArray, tx::signing::sign_info::SignMode,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeInfo {
    Single {
        mode: SignMode,
    },
    Multi {
        bitarray: CompactBitArray,
        mode_infos: Vec<ModeInfo>,
    },
}

#[cfg(feature = "proto")]
pub mod proto {
    use super::ModeInfo;
    use crate::{
        errors::{MissingField, UnknownEnumVariant},
        impl_proto_via_try_from_into, required,
    };

    impl_proto_via_try_from_into!(ModeInfo => protos::cosmos::tx::v1beta1::ModeInfo);

    impl From<ModeInfo> for protos::cosmos::tx::v1beta1::ModeInfo {
        fn from(value: ModeInfo) -> Self {
            protos::cosmos::tx::v1beta1::ModeInfo {
                sum: Some(match value {
                    ModeInfo::Single { mode } => {
                        protos::cosmos::tx::v1beta1::mode_info::Sum::Single(
                            protos::cosmos::tx::v1beta1::mode_info::Single { mode: mode.into() },
                        )
                    }
                    ModeInfo::Multi {
                        bitarray,
                        mode_infos,
                    } => protos::cosmos::tx::v1beta1::mode_info::Sum::Multi(
                        protos::cosmos::tx::v1beta1::mode_info::Multi {
                            bitarray: Some(bitarray.into()),
                            mode_infos: mode_infos.into_iter().map(Into::into).collect(),
                        },
                    ),
                }),
            }
        }
    }

    impl TryFrom<protos::cosmos::tx::v1beta1::ModeInfo> for ModeInfo {
        type Error = TryFromModeInfoError;

        fn try_from(value: protos::cosmos::tx::v1beta1::ModeInfo) -> Result<Self, Self::Error> {
            Ok(match required!(value.sum)? {
                protos::cosmos::tx::v1beta1::mode_info::Sum::Single(
                    protos::cosmos::tx::v1beta1::mode_info::Single { mode },
                ) => Self::Single {
                    mode: mode.try_into().map_err(TryFromModeInfoError::SingleMode)?,
                },
                protos::cosmos::tx::v1beta1::mode_info::Sum::Multi(
                    protos::cosmos::tx::v1beta1::mode_info::Multi {
                        bitarray,
                        mode_infos,
                    },
                ) => Self::Multi {
                    bitarray: bitarray.ok_or(MissingField("bitarray"))?.into(),
                    mode_infos: mode_infos
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_, _>>()
                        .map_err(Box::new)
                        .map_err(TryFromModeInfoError::MultiModeInfo)?,
                },
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromModeInfoError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid single mode")]
        SingleMode(#[source] UnknownEnumVariant<i32>),
        #[error("invalid multi mode info")]
        MultiModeInfo(#[source] Box<TryFromModeInfoError>),
    }
}

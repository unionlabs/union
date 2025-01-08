use alloc::borrow::Cow;

use macros::model;

use crate::{
    bounded::{BoundedIntError, BoundedUsize},
    cosmos::ics23::hash_op::HashOp,
    errors::UnknownEnumVariant,
};

pub type PositiveI32AsUsize = BoundedUsize<0, { i32::MAX as usize }>;

#[model(proto(raw(protos::cosmos::ics23::v1::InnerSpec), into, from))]
pub struct InnerSpec {
    pub child_order: Cow<'static, [PositiveI32AsUsize]>,
    pub child_size: PositiveI32AsUsize,
    pub min_prefix_length: PositiveI32AsUsize,
    pub max_prefix_length: PositiveI32AsUsize,
    #[serde(with = "::serde_utils::hex_string")]
    pub empty_child: Cow<'static, [u8]>,
    pub hash: HashOp,
}

impl From<InnerSpec> for protos::cosmos::ics23::v1::InnerSpec {
    fn from(value: InnerSpec) -> Self {
        Self {
            child_order: value
                .child_order
                .iter()
                .map(|x| {
                    x.inner()
                        .try_into()
                        .expect("value is bounded between 0..=i32::MAX")
                })
                .collect(),
            child_size: value
                .child_size
                .inner()
                .try_into()
                .expect("value is bounded between 1..=i32::MAX"),
            min_prefix_length: value
                .min_prefix_length
                .inner()
                .try_into()
                .expect("value is bounded between 0..=i32::MAX"),
            max_prefix_length: value
                .max_prefix_length
                .inner()
                .try_into()
                .expect("value is bounded between 0..=i32::MAX"),
            empty_child: value.empty_child.into(),
            hash: value.hash.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromInnerSpecError {
    #[error("invalid hash")]
    Hash(#[from] UnknownEnumVariant<i32>),
    #[error("invalid child order")]
    ChildOrder(#[source] BoundedIntError<usize>),
    #[error("invalid child size")]
    ChildSize(#[source] BoundedIntError<usize>),
    #[error("invalid min prefix length")]
    MinPrefixLength(#[source] BoundedIntError<usize>),
    #[error("invalid max prefix length")]
    MaxPrefixLength(#[source] BoundedIntError<usize>),
    #[error("negative child order")]
    NegativeChildOrder,
    #[error("negative child size")]
    NegativeChildSize,
    #[error("negative min prefix length")]
    NegativeMinPrefixLength,
    #[error("negative max prefix length")]
    NegativeMaxPrefixLength,
}

impl TryFrom<protos::cosmos::ics23::v1::InnerSpec> for InnerSpec {
    type Error = TryFromInnerSpecError;

    fn try_from(value: protos::cosmos::ics23::v1::InnerSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            child_order: value
                .child_order
                .into_iter()
                .map(|order| {
                    usize::try_from(order)
                        .map_err(|_| TryFromInnerSpecError::NegativeChildOrder)?
                        .try_into()
                        .map_err(TryFromInnerSpecError::ChildOrder)
                })
                .collect::<Result<_, _>>()?,
            child_size: usize::try_from(value.child_size)
                .map_err(|_| TryFromInnerSpecError::NegativeChildSize)?
                .try_into()
                .map_err(TryFromInnerSpecError::ChildSize)?,
            min_prefix_length: usize::try_from(value.min_prefix_length)
                .map_err(|_| TryFromInnerSpecError::NegativeMinPrefixLength)?
                .try_into()
                .map_err(TryFromInnerSpecError::MinPrefixLength)?,
            max_prefix_length: usize::try_from(value.max_prefix_length)
                .map_err(|_| TryFromInnerSpecError::NegativeMaxPrefixLength)?
                .try_into()
                .map_err(TryFromInnerSpecError::MaxPrefixLength)?,
            empty_child: value.empty_child.into(),
            hash: value.hash.try_into().map_err(TryFromInnerSpecError::Hash)?,
        })
    }
}

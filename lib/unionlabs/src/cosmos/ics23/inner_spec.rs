use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::hash_op::HashOp;
use crate::{
    bounded::{BoundedIntError, BoundedUsize},
    errors::UnknownEnumVariant,
    Proto, TypeUrl,
};

pub type PositiveI32AsUsize = BoundedUsize<0, { i32::MAX as usize }>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerSpec {
    pub child_order: Cow<'static, [PositiveI32AsUsize]>,
    pub child_size: PositiveI32AsUsize,
    pub min_prefix_length: PositiveI32AsUsize,
    pub max_prefix_length: PositiveI32AsUsize,
    #[serde(with = "::serde_utils::hex_string")]
    pub empty_child: Cow<'static, [u8]>,
    pub hash: HashOp,
}

impl TypeUrl for protos::cosmos::ics23::v1::InnerSpec {
    const TYPE_URL: &'static str = "/cosmos.ics23.v1.InnerSpec";
}

impl Proto for InnerSpec {
    type Proto = protos::cosmos::ics23::v1::InnerSpec;
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

#[derive(Debug)]
pub enum TryFromInnerSpecError {
    Hash(UnknownEnumVariant<i32>),
    ChildOrder(BoundedIntError<usize>),
    ChildSize(BoundedIntError<usize>),
    MinPrefixLength(BoundedIntError<usize>),
    MaxPrefixLength(BoundedIntError<usize>),
    NegativeChildOrder,
    NegativeChildSize,
    NegativeMinPrefixLength,
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

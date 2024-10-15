use macros::model;

use crate::{
    bounded::BoundedUsize,
    cosmos::ics23::{inner_spec::InnerSpec, leaf_op::LeafOp},
};

#[model(proto(raw(protos::cosmos::ics23::v1::ProofSpec), into, from))]
pub struct ProofSpec {
    pub leaf_spec: LeafOp,
    pub inner_spec: InnerSpec,
    // TODO: Merge these fields into a single range type to ensure `min? <= max?`
    pub max_depth: Option<BoundedUsize<1, { i32::MAX as usize }>>,
    pub min_depth: Option<BoundedUsize<1, { i32::MAX as usize }>>,
    pub prehash_key_before_comparison: bool,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        bounded::BoundedUsize,
        cosmos::ics23::{
            inner_spec::proto::TryFromInnerSpecError, leaf_op::proto::TryFromLeafOpError,
            proof_spec::ProofSpec,
        },
        errors::{required, MissingField},
    };

    impl From<ProofSpec> for protos::cosmos::ics23::v1::ProofSpec {
        fn from(value: ProofSpec) -> Self {
            Self {
                leaf_spec: Some(value.leaf_spec.into()),
                inner_spec: Some(value.inner_spec.into()),
                max_depth: value.max_depth.map_or(0, |md| {
                    md.inner()
                        .try_into()
                        .expect("value is bounded between 1..=i32::MAX")
                }),
                min_depth: value.min_depth.map_or(0, |md| {
                    md.inner()
                        .try_into()
                        .expect("value is bounded between 1..=i32::MAX")
                }),
                prehash_key_before_comparison: value.prehash_key_before_comparison,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromProofSpecError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid leaf spec")]
        LeafSpec(#[from] TryFromLeafOpError),
        #[error("invalid inner spec")]
        InnerSpec(#[from] TryFromInnerSpecError),
        #[error("negative max depth")]
        NegativeMinDepth,
        #[error("negative min depth")]
        NegativeMaxDepth,
    }

    impl TryFrom<protos::cosmos::ics23::v1::ProofSpec> for ProofSpec {
        type Error = TryFromProofSpecError;

        fn try_from(value: protos::cosmos::ics23::v1::ProofSpec) -> Result<Self, Self::Error> {
            Ok(Self {
                leaf_spec: required!(value.leaf_spec)?
                    .try_into()
                    .map_err(TryFromProofSpecError::LeafSpec)?,
                inner_spec: required!(value.inner_spec)?
                    .try_into()
                    .map_err(TryFromProofSpecError::InnerSpec)?,
                // x is between 0..i32::MAX here, expected type is between 1..=u32::MAX, we want the
                // behaviour of NonZero* here; so if the ctor fails, then we know the value is zero.
                // see test below for edge case handling
                max_depth: usize::try_from(value.max_depth)
                    .map_err(|_| TryFromProofSpecError::NegativeMaxDepth)
                    .map(|x| BoundedUsize::new(x).ok())?,
                min_depth: usize::try_from(value.min_depth)
                    .map_err(|_| TryFromProofSpecError::NegativeMinDepth)
                    .map(|x| BoundedUsize::new(x).ok())?,
                prehash_key_before_comparison: value.prehash_key_before_comparison,
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use super::*;

        #[test]
        fn min_max_depth_conversion_works() {
            let proto = protos::cosmos::ics23::v1::ProofSpec {
                leaf_spec: Some(protos::cosmos::ics23::v1::LeafOp {
                    hash: 1,
                    prehash_key: 1,
                    prehash_value: 1,
                    length: 1,
                    prefix: [].into(),
                }),
                inner_spec: Some(protos::cosmos::ics23::v1::InnerSpec {
                    child_order: [].into(),
                    child_size: 1,
                    min_prefix_length: 1,
                    max_prefix_length: 1,
                    empty_child: [].into(),
                    hash: 1,
                }),
                max_depth: 1,
                min_depth: 1,
                prehash_key_before_comparison: false,
            };

            let cvt = ProofSpec::try_from(proto.clone()).unwrap();
            assert_eq!(cvt.max_depth, Some(BoundedUsize::new(1).unwrap()));
            assert_eq!(cvt.min_depth, Some(BoundedUsize::new(1).unwrap()));

            let proto = protos::cosmos::ics23::v1::ProofSpec {
                max_depth: 0,
                min_depth: 0,
                ..proto
            };

            let cvt = ProofSpec::try_from(proto.clone()).unwrap();
            assert_eq!(cvt.max_depth, None);
            assert_eq!(cvt.min_depth, None);

            let proto = protos::cosmos::ics23::v1::ProofSpec {
                max_depth: i32::MAX,
                min_depth: i32::MAX,
                ..proto
            };

            let cvt = ProofSpec::try_from(proto.clone()).unwrap();
            assert_eq!(
                cvt.max_depth,
                Some(BoundedUsize::new_const(i32::MAX.try_into().unwrap()).unwrap())
            );
            assert_eq!(
                cvt.min_depth,
                Some(BoundedUsize::new_const(i32::MAX.try_into().unwrap()).unwrap())
            );
        }
    }
}

use std::{collections::VecDeque, fmt::Display, ops::ControlFlow};

use frunk::{HCons, HNil};

use crate::{data::AnyData, AnyLightClientIdentified, RelayerMsg};

pub trait IsAggregateData = TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>>
    + Into<AnyLightClientIdentified<AnyData>>;

pub fn do_aggregate<T: UseAggregate>(
    event: T,
    data: VecDeque<AnyLightClientIdentified<AnyData>>,
) -> RelayerMsg {
    // let data_json = serde_json::to_string(&data).expect("serialization should not fail");

    // tracing::info!(%data_json, "aggregating data");

    let data = match HListTryFromIterator::try_from_iter(data) {
        Ok(ok) => ok,
        Err(_) => {
            panic!(
                "could not aggregate data into {}",
                std::any::type_name::<T>()
            )
        }
    };
    T::aggregate(event, data)
}

pub(crate) fn pluck<T: TryFrom<U, Error = U>, U>(
    mut vec: VecDeque<U>,
) -> ControlFlow<(VecDeque<U>, T), VecDeque<U>> {
    let mut new_vec = VecDeque::new();

    while let Some(data) = vec.pop_front() {
        match T::try_from(data) {
            Ok(t) => {
                new_vec.extend(vec);
                return ControlFlow::Break((new_vec, t));
            }
            Err(value) => new_vec.push_back(value),
        }
    }

    ControlFlow::Continue(new_vec)
}

// TODO: Figure out how to return the entire source list on error
pub trait HListTryFromIterator<U>: Sized {
    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>>;
}

impl<U, T, Tail> HListTryFromIterator<U> for HCons<T, Tail>
where
    T: TryFrom<U, Error = U> + Into<U>,
    Tail: HListTryFromIterator<U>,
    // REVIEW: Should debug be used instead?
    U: Display,
{
    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>> {
        match pluck::<T, U>(vec) {
            ControlFlow::Continue(invalid) => {
                let invalid_str = invalid
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                tracing::error!(
                    %invalid_str,
                    "type didn't match"
                );

                Err(invalid)
            }
            ControlFlow::Break((vec, u)) => Ok(HCons {
                head: u,
                tail: Tail::try_from_iter(vec)?,
            }),
        }
    }
}

impl<U> HListTryFromIterator<U> for HNil {
    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>> {
        if vec.is_empty() {
            Ok(Self)
        } else {
            Err(vec)
        }
    }
}

pub trait UseAggregate {
    type AggregatedData: HListTryFromIterator<AnyLightClientIdentified<AnyData>>;

    fn aggregate(this: Self, data: Self::AggregatedData) -> RelayerMsg;
}

#[cfg(test)]
pub(crate) mod tests {
    use frunk::HList;

    use super::*;
    use crate::enum_variants_conversions;

    #[test]
    fn hlist_try_from_iter() {
        enum_variants_conversions! {
            #[derive(Debug, PartialEq, derive_more::Display)]
            #[display(fmt = "{}")]
            pub enum A {
                #[display(fmt = "{_0}")]
                B(B),
                #[display(fmt = "{_0}")]
                C(C),
                #[display(fmt = "{_0}")]
                D(D),
            }
        }

        #[derive(Debug, PartialEq, derive_more::Display)]
        #[display(fmt = "B")]
        pub struct B;

        #[derive(Debug, PartialEq, derive_more::Display)]
        #[display(fmt = "C")]
        pub struct C;

        #[derive(Debug, PartialEq, derive_more::Display)]
        #[display(fmt = "D")]
        pub struct D;

        // correct items, correct order
        assert_eq!(
            <HList![B, C]>::try_from_iter(VecDeque::from_iter([A::B(B), A::C(C)])),
            Ok(frunk::hlist![B, C])
        );

        // correct items, wrong order
        assert_eq!(
            <HList![B, C]>::try_from_iter(VecDeque::from_iter([A::C(C), A::B(B)])),
            Ok(frunk::hlist![B, C])
        );

        // extra items
        assert_eq!(
            <HList![B, C]>::try_from_iter(VecDeque::from_iter([A::C(C), A::B(B), A::D(D)])),
            Err(VecDeque::from_iter([A::D(D)]))
        );

        // missing items
        assert_eq!(
            <HList![B, C]>::try_from_iter(VecDeque::from_iter([A::C(C)])),
            Err(VecDeque::from_iter([A::C(C)]))
        );

        // missing items, extra items present
        assert_eq!(
            <HList![B, C]>::try_from_iter(VecDeque::from_iter([A::C(C), A::D(D)])),
            Err(VecDeque::from_iter([A::C(C), A::D(D)]))
        );
    }
}

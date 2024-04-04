use std::{collections::VecDeque, fmt::Debug, ops::ControlFlow};

use frunk::{HCons, HNil};

use crate::{QueueMsg, QueueMsgTypes};

pub fn do_aggregate<T: QueueMsgTypes, A: UseAggregate<T>>(
    event: A,
    data: VecDeque<T::Data>,
) -> QueueMsg<T> {
    let data = match HListTryFromIterator::try_from_iter(data) {
        Ok(ok) => ok,
        Err(_) => {
            panic!(
                "could not aggregate data into {}",
                std::any::type_name::<A>()
            )
        }
    };
    A::aggregate(event, data)
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
    const LEN: usize;
    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>>;
}

impl<U, T, Tail> HListTryFromIterator<U> for HCons<T, Tail>
where
    T: TryFrom<U, Error = U> + Into<U>,
    Tail: HListTryFromIterator<U>,
    U: Debug,
{
    const LEN: usize = 1 + Tail::LEN;

    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>> {
        match pluck::<T, U>(vec) {
            ControlFlow::Continue(invalid) => {
                tracing::error!(?invalid, "type didn't match");

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
    const LEN: usize = 0;

    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>> {
        if vec.is_empty() {
            Ok(Self)
        } else {
            Err(vec)
        }
    }
}

pub trait UseAggregate<T: QueueMsgTypes, R = QueueMsg<T>> {
    type AggregatedData: HListTryFromIterator<T::Data>;

    fn aggregate(this: Self, data: Self::AggregatedData) -> R;
}

#[cfg(test)]
mod tests {
    use frunk::HList;

    use super::*;

    #[test]
    fn hlist_try_from_iter() {
        #[derive(Debug, PartialEq, enumorph::Enumorph)]
        pub enum A {
            B(B),
            C(C),
            D(D),
        }

        #[derive(Debug, PartialEq)]
        pub struct B;

        #[derive(Debug, PartialEq)]
        pub struct C;

        #[derive(Debug, PartialEq)]
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

use std::{collections::VecDeque, fmt::Debug, ops::ControlFlow};

use frunk::{HCons, HList, HNil};
use tracing::error;

use crate::{Op, QueueMessage};

pub fn do_aggregate<T: QueueMessage, A: UseAggregate<T>>(
    event: A,
    data: VecDeque<T::Data>,
) -> Op<T> {
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
                error!(?invalid, "type didn't match");

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

pub trait UseAggregate<T: QueueMessage, R = Op<T>> {
    type AggregatedData: HListTryFromIterator<T::Data>;

    fn aggregate(this: Self, data: Self::AggregatedData) -> R;
}

pub struct TupleAggregator;

pub trait IsAggregateData<T: QueueMessage> = TryFrom<<T as QueueMessage>::Data, Error = <T as QueueMessage>::Data>
    + Into<<T as QueueMessage>::Data>;

impl<T> UseAggregate<T, ()> for TupleAggregator
where
    T: QueueMessage,
{
    type AggregatedData = HList![];

    fn aggregate(_: TupleAggregator, _data: Self::AggregatedData) {}
}

impl<T, U, Tail> UseAggregate<T, (U, Tail)> for TupleAggregator
where
    T: QueueMessage,
    U: IsAggregateData<T>,
    TupleAggregator: UseAggregate<T, Tail>,
    HList![U, ...<TupleAggregator as UseAggregate<T, Tail>>::AggregatedData]:
        HListAsTuple<Tuple = (U, Tail)>,
{
    type AggregatedData = HList![U, ...<TupleAggregator as UseAggregate<T, Tail>>::AggregatedData];

    fn aggregate(_: TupleAggregator, data: Self::AggregatedData) -> (U, Tail) {
        data.into_tuple()
    }
}

trait HListAsTuple {
    type Tuple;

    fn into_tuple(self) -> Self::Tuple;
}

impl HListAsTuple for HNil {
    type Tuple = ();

    fn into_tuple(self) {}
}

impl<H, Tail: HListAsTuple> HListAsTuple for HCons<H, Tail> {
    type Tuple = (H, Tail::Tuple);

    fn into_tuple(self) -> Self::Tuple {
        (self.head, self.tail.into_tuple())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use frunk::HList;
    use tracing_subscriber::EnvFilter;

    use super::*;
    use crate::{
        fetch,
        optimize::{passes::NormalizeFinal, Pure},
        run_to_completion,
        test_utils::{DataA, DataB, DataC, FetchA, FetchB, FetchC, SimpleMessage},
        InMemoryQueue,
    };

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

    #[tokio::test]
    async fn tuple_aggregate() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init();

        let _: () = run_to_completion::<
            TupleAggregator,
            SimpleMessage,
            (),
            InMemoryQueue<SimpleMessage>,
            NormalizeFinal,
            Pure<NormalizeFinal>,
        >(
            TupleAggregator,
            Arc::new(()),
            (),
            [],
            NormalizeFinal::default(),
            Pure(NormalizeFinal::default()),
        )
        .await;

        let _: (DataA, ()) = run_to_completion::<
            TupleAggregator,
            SimpleMessage,
            (DataA, ()),
            InMemoryQueue<SimpleMessage>,
            NormalizeFinal,
            Pure<NormalizeFinal>,
        >(
            TupleAggregator,
            Arc::new(()),
            (),
            [fetch(FetchA {})],
            NormalizeFinal::default(),
            Pure(NormalizeFinal::default()),
        )
        .await;

        let _: (DataC, (DataB, (DataA, ()))) = run_to_completion::<
            TupleAggregator,
            SimpleMessage,
            (DataC, (DataB, (DataA, ()))),
            InMemoryQueue<SimpleMessage>,
            NormalizeFinal,
            Pure<NormalizeFinal>,
        >(
            TupleAggregator,
            Arc::new(()),
            (),
            [fetch(FetchA {}), fetch(FetchC {}), fetch(FetchB {})],
            NormalizeFinal::default(),
            Pure(NormalizeFinal::default()),
        )
        .await;
    }
}

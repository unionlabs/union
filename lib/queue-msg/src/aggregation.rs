use std::{collections::VecDeque, fmt::Debug, ops::ControlFlow};

use frunk::{HCons, HList, HNil};

use crate::{QueueMessageTypes, QueueMsg};

pub fn do_aggregate<T: QueueMessageTypes, A: UseAggregate<T>>(
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

pub trait UseAggregate<T: QueueMessageTypes, R = QueueMsg<T>> {
    type AggregatedData: HListTryFromIterator<T::Data>;

    fn aggregate(this: Self, data: Self::AggregatedData) -> R;
}

pub struct TupleAggregator;

pub trait IsAggregateData<T: QueueMessageTypes> = TryFrom<<T as QueueMessageTypes>::Data, Error = <T as QueueMessageTypes>::Data>
    + Into<<T as QueueMessageTypes>::Data>;

impl<T> UseAggregate<T, ()> for TupleAggregator
where
    T: QueueMessageTypes,
{
    type AggregatedData = HList![];

    fn aggregate(_: TupleAggregator, _data: Self::AggregatedData) {}
}

impl<T, U, Tail> UseAggregate<T, (U, Tail)> for TupleAggregator
where
    T: QueueMessageTypes,
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

    use enumorph::Enumorph;
    use frunk::HList;
    use queue_msg_macro::queue_msg;

    use super::*;
    use crate::{
        data, fetch, run_to_completion, HandleAggregate, HandleData, HandleEffect, HandleEvent,
        HandleFetch, HandleWait, InMemoryQueue, QueueError,
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

    pub enum SimpleMessageTypes {}

    impl QueueMessageTypes for SimpleMessageTypes {
        type Event = SimpleEvent;
        type Data = SimpleData;
        type Fetch = SimpleFetch;
        type Effect = SimpleEffect;
        type Wait = SimpleWait;

        type Aggregate = SimpleAggregate;

        type Store = ();
    }

    impl HandleEffect<SimpleMessageTypes> for SimpleEffect {
        async fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessageTypes>, QueueError> {
            Ok(QueueMsg::Noop)
        }
    }

    impl HandleEvent<SimpleMessageTypes> for SimpleEvent {
        fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessageTypes>, QueueError> {
            Ok(QueueMsg::Noop)
        }
    }

    impl HandleData<SimpleMessageTypes> for SimpleData {
        fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessageTypes>, QueueError> {
            Ok(data(self))
        }
    }

    impl HandleFetch<SimpleMessageTypes> for SimpleFetch {
        async fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessageTypes>, QueueError> {
            Ok(match self {
                SimpleFetch::A(_) => data(DataA {}),
                SimpleFetch::B(_) => data(DataB {}),
                SimpleFetch::C(_) => data(DataC {}),
                SimpleFetch::D(_) => data(DataD {}),
                SimpleFetch::E(_) => data(DataE {}),
            })
        }
    }

    impl HandleWait<SimpleMessageTypes> for SimpleWait {
        async fn handle(self, _: &()) -> Result<QueueMsg<SimpleMessageTypes>, QueueError> {
            Ok(QueueMsg::Noop)
        }
    }

    impl HandleAggregate<SimpleMessageTypes> for SimpleAggregate {
        fn handle(
            self,
            _: VecDeque<SimpleData>,
        ) -> Result<QueueMsg<SimpleMessageTypes>, QueueError> {
            Ok(QueueMsg::Noop)
        }
    }

    #[queue_msg]
    pub struct SimpleEvent {}
    #[queue_msg]
    #[derive(Enumorph)]
    pub enum SimpleData {
        A(DataA),
        B(DataB),
        C(DataC),
        D(DataD),
        E(DataE),
    }
    #[queue_msg]
    pub struct DataA {}
    #[queue_msg]
    pub struct DataB {}
    #[queue_msg]
    pub struct DataC {}
    #[queue_msg]
    pub struct DataD {}
    #[queue_msg]
    pub struct DataE {}

    #[queue_msg]
    #[derive(Enumorph)]
    pub enum SimpleFetch {
        A(FetchA),
        B(FetchB),
        C(FetchC),
        D(FetchD),
        E(FetchE),
    }
    #[queue_msg]
    pub struct FetchA {}
    #[queue_msg]
    pub struct FetchB {}
    #[queue_msg]
    pub struct FetchC {}
    #[queue_msg]
    pub struct FetchD {}
    #[queue_msg]
    pub struct FetchE {}

    #[queue_msg]
    pub struct SimpleEffect {}
    #[queue_msg]
    pub struct SimpleWait {}

    #[queue_msg]
    pub struct SimpleAggregate {}

    #[tokio::test]
    async fn tuple_aggregate() {
        let _: () = run_to_completion::<
            TupleAggregator,
            SimpleMessageTypes,
            (),
            InMemoryQueue<SimpleMessageTypes>,
        >(TupleAggregator, Arc::new(()), (), [])
        .await;

        let _: (DataA, ()) = run_to_completion::<
            TupleAggregator,
            SimpleMessageTypes,
            (DataA, ()),
            InMemoryQueue<SimpleMessageTypes>,
        >(TupleAggregator, Arc::new(()), (), [fetch(FetchA {})])
        .await;

        let _: (DataC, (DataB, (DataA, ()))) = run_to_completion::<
            TupleAggregator,
            SimpleMessageTypes,
            (DataC, (DataB, (DataA, ()))),
            InMemoryQueue<SimpleMessageTypes>,
        >(
            TupleAggregator,
            Arc::new(()),
            (),
            [fetch(FetchA {}), fetch(FetchC {}), fetch(FetchB {})],
        )
        .await;
    }
}

use std::{collections::VecDeque, fmt::Debug};

use frunk::{HCons, HList, HNil};
use subset_of::SubsetOf;
use tracing::error;

use crate::{Op, QueueMessage};

pub fn do_callback<T: QueueMessage, Cb: DoCallback<T>>(
    event: Cb,
    data: VecDeque<T::Data>,
) -> Op<T> {
    let data = match HListTryFromIterator::try_from_iter(data) {
        Ok(ok) => ok,
        Err(_) => {
            error!(
                "could not aggregate data into {}",
                std::any::type_name::<Cb>()
            );
            panic!(
                "could not aggregate data into {}",
                std::any::type_name::<Cb>()
            )
        }
    };
    Cb::call(event, data)
}

pub fn pluck<T: SubsetOf<U>, U>(mut vec: VecDeque<U>) -> PluckResult<T, U> {
    let mut new_vec = VecDeque::new();

    while let Some(data) = vec.pop_front() {
        match T::try_from_super(data) {
            Ok(t) => {
                new_vec.extend(vec);
                return PluckResult::Found(t, new_vec);
            }
            Err(value) => new_vec.push_back(value),
        }
    }

    PluckResult::NotFound(new_vec)
}

pub enum PluckResult<T, U> {
    /// `(item, rest)`
    Found(T, VecDeque<U>),
    NotFound(VecDeque<U>),
}

// TODO: Figure out how to return the entire source list on error
pub trait HListTryFromIterator<U>: Sized {
    const LEN: usize;
    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>>;
}

impl<U, T, Tail> HListTryFromIterator<U> for HCons<T, Tail>
where
    T: SubsetOf<U>,
    Tail: HListTryFromIterator<U>,
    U: Debug,
{
    const LEN: usize = 1 + Tail::LEN;

    fn try_from_iter(vec: VecDeque<U>) -> Result<Self, VecDeque<U>> {
        match pluck::<T, U>(vec) {
            PluckResult::NotFound(invalid) => {
                error!(?invalid, "type didn't match");

                Err(invalid)
            }
            PluckResult::Found(t, vec) => Ok(HCons {
                head: t,
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

pub trait DoCallback<T: QueueMessage, R = Op<T>> {
    /// Unordered params for this callback. If there are multiple params of the same type, their ordering is not guaranteed or deterministic.
    type Params: HListTryFromIterator<T::Data>;

    fn call(this: Self, data: Self::Params) -> R;
}

pub struct TupleCallback;

// pub trait IsAggregateData<T: QueueMessage> = TryFrom<<T as QueueMessage>::Data, Error = <T as QueueMessage>::Data>
//     + Into<<T as QueueMessage>::Data>;

impl<T> DoCallback<T, ()> for TupleCallback
where
    T: QueueMessage,
{
    type Params = HList![];

    fn call(_: TupleCallback, _data: Self::Params) {}
}

impl<T, U, Tail> DoCallback<T, (U, Tail)> for TupleCallback
where
    T: QueueMessage,
    U: SubsetOf<T::Data>,
    TupleCallback: DoCallback<T, Tail>,
    HList![U, ...<TupleCallback as DoCallback<T, Tail>>::Params]: HListAsTuple<Tuple = (U, Tail)>,
{
    type Params = HList![U, ...<TupleCallback as DoCallback<T, Tail>>::Params];

    fn call(_: TupleCallback, data: Self::Params) -> (U, Tail) {
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
    use std::collections::VecDeque;

    use enumorph::Enumorph;
    use frunk::HList;
    use subset_of::SubsetOf;

    use crate::aggregation::HListTryFromIterator;

    #[test]
    fn hlist_try_from_iter() {
        #[derive(Debug, PartialEq, Enumorph, SubsetOf)]
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

    // #[tokio::test]
    // async fn tuple_aggregate() {
    //     let _ = tracing_subscriber::fmt()
    //         .with_env_filter(EnvFilter::from_default_env())
    //         .try_init();

    //     let _: () = run_to_completion::<
    //         TupleCallback,
    //         SimpleMessage,
    //         (),
    //         InMemoryQueue<SimpleMessage>,
    //         NoopPass,
    //         Pure<NoopPass>,
    //     >(TupleCallback, (), (), [], NoopPass, Pure(NoopPass))
    //     .await;

    //     let _: (DataA, ()) = run_to_completion::<
    //         TupleCallback,
    //         SimpleMessage,
    //         (DataA, ()),
    //         InMemoryQueue<SimpleMessage>,
    //         NoopPass,
    //         Pure<NoopPass>,
    //     >(
    //         TupleCallback,
    //         (),
    //         (),
    //         [call(FetchA {})],
    //         NoopPass,
    //         Pure(NoopPass),
    //     )
    //     .await;

    //     let _: (DataC, (DataB, (DataA, ()))) = run_to_completion::<
    //         TupleCallback,
    //         SimpleMessage,
    //         (DataC, (DataB, (DataA, ()))),
    //         InMemoryQueue<SimpleMessage>,
    //         NoopPass,
    //         Pure<NoopPass>,
    //     >(
    //         TupleCallback,
    //         (),
    //         (),
    //         [call(FetchA {}), call(FetchC {}), call(FetchB {})],
    //         NoopPass,
    //         Pure(NoopPass),
    //     )
    //     .await;
    // }
}

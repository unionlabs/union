use std::collections::VecDeque;

use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;

use crate::{Handler, Op, QueueError, QueueMessage, call, data, noop};

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleMessage {}

impl QueueMessage for SimpleMessage {
    type Data = SimpleData;
    type Call = SimpleCall;
    type Callback = SimpleCallback;
}

impl Handler<SimpleMessage> for () {
    async fn call(&self, call: SimpleCall) -> Result<Op<SimpleMessage>, QueueError> {
        Ok(match call {
            SimpleCall::A(FetchA {}) => data(DataA {}),
            SimpleCall::B(FetchB {}) => data(DataB {}),
            SimpleCall::C(FetchC {}) => data(DataC {}),
            SimpleCall::D(FetchD {}) => data(DataD {}),
            SimpleCall::E(FetchE {}) => data(DataE {}),
            SimpleCall::PrintAbc(PrintAbc { a, b, c }) => {
                println!("a = {a:?}, b = {b:?}, c = {c:?}");
                noop()
            }
        })
    }

    async fn callback(
        &self,
        callback: SimpleCallback,
        data: VecDeque<SimpleData>,
    ) -> Result<Op<SimpleMessage>, QueueError> {
        Ok(match callback {
            SimpleCallback::BuildPrintAbc(BuildPrintAbc {}) => {
                let mut data = data.into_iter().collect();

                let op = call(PrintAbc {
                    a: find_in_vec(&mut data, |d| d.clone().try_into().ok()).unwrap(),
                    b: find_in_vec(&mut data, |d| d.clone().try_into().ok()).unwrap(),
                    c: find_in_vec(&mut data, |d| d.clone().try_into().ok()).unwrap(),
                });

                assert!(data.is_empty());

                op
            }
        })
    }
}

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum SimpleData {
    A(DataA),
    B(DataB),
    C(DataC),
    D(DataD),
    E(DataE),
}
#[model]
pub struct DataA {}
#[model]
pub struct DataB {}
#[model]
pub struct DataC {}
#[model]
pub struct DataD {}
#[model]
pub struct DataE {}

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum SimpleCall {
    A(FetchA),
    B(FetchB),
    C(FetchC),
    D(FetchD),
    E(FetchE),
    PrintAbc(PrintAbc),
}
#[model]
pub struct FetchA {}
#[model]
pub struct FetchB {}
#[model]
pub struct FetchC {}
#[model]
pub struct FetchD {}
#[model]
pub struct FetchE {}

#[model]
pub struct PrintAbc {
    pub a: DataA,
    pub b: DataB,
    pub c: DataC,
}

#[model]
#[derive(Enumorph)]
pub enum SimpleCallback {
    BuildPrintAbc(BuildPrintAbc),
}

#[model]
pub struct BuildPrintAbc {}

fn find_in_vec<T, U>(v: &mut Vec<T>, mut predicate: impl FnMut(&T) -> Option<U>) -> Option<U> {
    v.iter()
        .enumerate()
        .find_map(|(i, t)| predicate(t).map(|u| (i, u)))
        .map(|(i, u)| {
            v.remove(i);
            u
        })
}

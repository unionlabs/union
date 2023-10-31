#![cfg_attr(not(feature = "full"), allow(dead_code))]

use crate::park::{Park, Unpark};

use std::fmt;
use std::time::Duration;

pub(crate) enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> Park for Either<A, B>
where
    A: Park,
    B: Park,
{
    type Unpark = Either<A::Unpark, B::Unpark>;
    type Error = Either<A::Error, B::Error>;

    fn unpark(&self) -> Self::Unpark {
        match self {
            Either::A(a) => Either::A(a.unpark()),
            Either::B(b) => Either::B(b.unpark()),
        }
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        match self {
            Either::A(a) => a.park().map_err(Either::A),
            Either::B(b) => b.park().map_err(Either::B),
        }
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        match self {
            Either::A(a) => a.park_timeout(duration).map_err(Either::A),
            Either::B(b) => b.park_timeout(duration).map_err(Either::B),
        }
    }

    fn shutdown(&mut self) {
        match self {
            Either::A(a) => a.shutdown(),
            Either::B(b) => b.shutdown(),
        }
    }
}

impl<A, B> Unpark for Either<A, B>
where
    A: Unpark,
    B: Unpark,
{
    fn unpark(&self) {
        match self {
            Either::A(a) => a.unpark(),
            Either::B(b) => b.unpark(),
        }
    }
}

impl<A, B> fmt::Debug for Either<A, B>
where
    A: fmt::Debug,
    B: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::A(a) => a.fmt(fmt),
            Either::B(b) => b.fmt(fmt),
        }
    }
}

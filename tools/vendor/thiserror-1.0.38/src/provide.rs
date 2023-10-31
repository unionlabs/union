use std::any::{Demand, Provider};

pub trait ThiserrorProvide: Sealed {
    fn thiserror_provide<'a>(&'a self, demand: &mut Demand<'a>);
}

impl<T: Provider + ?Sized> ThiserrorProvide for T {
    #[inline]
    fn thiserror_provide<'a>(&'a self, demand: &mut Demand<'a>) {
        self.provide(demand);
    }
}

pub trait Sealed {}
impl<T: Provider + ?Sized> Sealed for T {}

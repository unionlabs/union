use std::marker::PhantomData;

use crate::query::CustomQuery;
use crate::results::Empty;
use crate::traits::{Api, Querier, Storage};
use crate::QuerierWrapper;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct OwnedDeps<S: Storage, A: Api, Q: Querier, C: CustomQuery = Empty> {
    pub storage: S,
    pub api: A,
    pub querier: Q,
    pub custom_query_type: PhantomData<C>,
}

pub struct DepsMut<'a, C: CustomQuery = Empty> {
    pub storage: &'a mut dyn Storage,
    pub api: &'a dyn Api,
    pub querier: QuerierWrapper<'a, C>,
}

#[derive(Clone)]
pub struct Deps<'a, C: CustomQuery = Empty> {
    pub storage: &'a dyn Storage,
    pub api: &'a dyn Api,
    pub querier: QuerierWrapper<'a, C>,
}

// Use custom implementation on order to implement Copy in case `C` is not `Copy`.
// See "There is a small difference between the two: the derive strategy will also
// place a Copy bound on type parameters, which isn’t always desired."
// https://doc.rust-lang.org/std/marker/trait.Copy.html
impl<'a, C: CustomQuery> Copy for Deps<'a, C> {}

impl<S: Storage, A: Api, Q: Querier, C: CustomQuery> OwnedDeps<S, A, Q, C> {
    pub fn as_ref(&'_ self) -> Deps<'_, C> {
        Deps {
            storage: &self.storage,
            api: &self.api,
            querier: QuerierWrapper::new(&self.querier),
        }
    }

    pub fn as_mut(&'_ mut self) -> DepsMut<'_, C> {
        DepsMut {
            storage: &mut self.storage,
            api: &self.api,
            querier: QuerierWrapper::new(&self.querier),
        }
    }
}

impl<'a, C: CustomQuery> DepsMut<'a, C> {
    pub fn as_ref(&'_ self) -> Deps<'_, C> {
        Deps {
            storage: self.storage,
            api: self.api,
            querier: self.querier,
        }
    }

    pub fn branch(&'_ mut self) -> DepsMut<'_, C> {
        DepsMut {
            storage: self.storage,
            api: self.api,
            querier: self.querier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
    use serde::{Deserialize, Serialize};

    // ensure we can call these many times, eg. as sub-calls
    fn execute(mut deps: DepsMut) {
        execute2(deps.branch());
        query(deps.as_ref());
        execute2(deps.branch());
    }
    fn execute2(_deps: DepsMut) {}

    fn query(deps: Deps) {
        query2(deps);
        query2(deps);
    }
    fn query2(_deps: Deps) {}

    #[test]
    fn ensure_easy_reuse() {
        let mut deps = mock_dependencies();
        execute(deps.as_mut());
        query(deps.as_ref())
    }

    #[test]
    fn deps_implements_copy() {
        impl CustomQuery for u64 {}
        #[derive(Clone, Serialize, Deserialize)]
        struct MyQuery;
        impl CustomQuery for MyQuery {}

        // With C: Copy
        let owned = OwnedDeps::<_, _, _, u64> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<u64>::new(&[]),
            custom_query_type: PhantomData,
        };
        let deps: Deps<u64> = owned.as_ref();
        let _copy1 = deps;
        let _copy2 = deps;

        // Without C: Copy
        let owned = OwnedDeps::<_, _, _, MyQuery> {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::<MyQuery>::new(&[]),
            custom_query_type: PhantomData,
        };
        let deps: Deps<MyQuery> = owned.as_ref();
        let _copy1 = deps;
        let _copy2 = deps;
    }
}

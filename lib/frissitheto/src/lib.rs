#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs)]

use std::num::NonZeroU32;

use cosmwasm_std::{CustomMsg, DepsMut, Response, StdError};
use serde::{Deserialize, Serialize};

/// The storage prefix that the current state version is stored under.
pub const STATE_VERSION: &[u8] = b"state_version";

/// The migrate message to be used for contracts using `frissitheto`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum UpgradeMsg<Init, Migrate> {
    /// Initiate the contract, migrating it from the existing bytecode.
    ///
    /// This can only be called once. Any subsequent migrations called with this entrypoint will
    /// fail with [`UpgradeError::AlreadyInitiated`].
    Init(Init),
    /// Migrate the contract.
    Migrate(Migrate),
}

impl<Init, Migrate> UpgradeMsg<Init, Migrate> {
    /// Run the migration.
    ///
    /// If the contract has not yet been initiated, `init_f` will be called, otherwise `migrate_f`
    /// will be called. This will error if `self` is not the correct variant, respectively.
    ///
    /// `init_f` takes the `Init` message, returning a `Response` and an optional state version. If
    /// `None` is returned, the version will be set to `1`.
    ///
    /// `migrate_f` takes the `Migrate` message and the current state version, returning a
    /// `Response` and an optional state version. If `None` is returned, the version will not be
    /// modified, however if a new version is returned it MUST be > the current version. An error
    /// will be returned if this is not the case.
    ///
    /// State is stored under [`STATE_VERSION`] at the contract root. Consumers of this library
    /// MUST ensure to not overwrite this key.
    ///
    /// # Errors
    ///
    /// This function will error if either `init_f` or `migrate_f` error. See [`UpgradeError`] for
    /// additional failure modes.
    ///
    /// # Panics
    ///
    /// This function will panic if the state version cannot be decoded.
    pub fn run<E: From<UpgradeError> + From<StdError>, T: CustomMsg>(
        self,
        mut deps: DepsMut,
        init_f: impl FnOnce(DepsMut, Init) -> Result<(Response<T>, Option<NonZeroU32>), E>,
        migrate_f: impl FnOnce(
            DepsMut,
            Migrate,
            NonZeroU32,
        ) -> Result<(Response<T>, Option<NonZeroU32>), E>,
    ) -> Result<Response<T>, E> {
        let state_version = deps.storage.get(STATE_VERSION);
        match state_version {
            Some(state_version) => match self {
                UpgradeMsg::Init(_) => Err(UpgradeError::AlreadyInitiated.into()),
                UpgradeMsg::Migrate(migrate) => {
                    let current_state_version =
                        NonZeroU32::new(u32::from_be_bytes(state_version.try_into().unwrap()))
                            .unwrap();

                    let (res, new_version) =
                        migrate_f(deps.branch(), migrate, current_state_version)?;

                    if let Some(new_version) = new_version
                        && new_version <= current_state_version
                    {
                        return Err(UpgradeError::StateVersionMustIncreaseIfModified {
                            current: current_state_version,
                            new: new_version,
                        }
                        .into());
                    }

                    deps.storage.set(
                        STATE_VERSION,
                        &(new_version.unwrap_or(current_state_version))
                            .get()
                            .to_be_bytes(),
                    );
                    Ok(res)
                }
            },
            None => match self {
                UpgradeMsg::Init(init) => {
                    let (res, version) = init_f(deps.branch(), init)?;

                    deps.storage.set(
                        STATE_VERSION,
                        &version.map_or(1, NonZeroU32::get).to_be_bytes(),
                    );

                    Ok(res)
                }
                UpgradeMsg::Migrate(_) => Err(UpgradeError::NotInitiated.into()),
            },
        }
    }
}

/// Possible errors that can occur while executing [`UpgradeMsg::run()`].
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[expect(missing_docs, reason = "#[error] attributes provide documentation")]
pub enum UpgradeError {
    #[error("attempted to initiate a contract that was already initiated")]
    AlreadyInitiated,
    #[error("attempted to migrate a contract that was not yet initiated")]
    NotInitiated,
    #[error("unknown state version {0}")]
    UnknownStateVersion(NonZeroU32),
    #[error(
        "the state version must increase if it is modified, attempted to migrate from {current} to {new}"
    )]
    StateVersionMustIncreaseIfModified {
        current: NonZeroU32,
        new: NonZeroU32,
    },
}

/// Initiate the state version in a contract that was not previously using `frissitheto`.
///
/// # Errors
///
/// This function will return an error if the state version has already been set.
pub fn init_state_version(
    deps: &mut DepsMut<'_>,
    version: NonZeroU32,
) -> Result<(), InitStateVersionError> {
    let state_version = deps.storage.get(STATE_VERSION);

    if state_version.is_some() {
        Err(InitStateVersionError::AlreadyInitiated)
    } else {
        deps.storage
            .set(STATE_VERSION, &version.get().to_be_bytes());

        Ok(())
    }
}

/// Possible errors that can occur while executing [`init_state_version()`].
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[expect(missing_docs, reason = "#[error] attributes provide documentation")]
pub enum InitStateVersionError {
    #[error("attempted to init the state version in a contract that was already initiated")]
    AlreadyInitiated,
}

// TODO: Add tests for version changes as well
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cosmwasm_std::{Empty, MemoryStorage, Storage, testing::mock_dependencies};

    use super::*;

    #[track_caller]
    fn assert_storage_eq(
        storage: &MemoryStorage,
        expected: impl IntoIterator<Item = (Vec<u8>, Vec<u8>)>,
    ) {
        let found = storage
            .range(None, None, cosmwasm_std::Order::Ascending)
            .collect::<BTreeMap<_, _>>();

        let expected = expected.into_iter().collect::<BTreeMap<_, _>>();

        assert_eq!(expected, found);
    }

    #[derive(Debug, PartialEq, thiserror::Error)]
    enum Error {
        #[error(transparent)]
        Std(#[from] StdError),
        #[error(transparent)]
        Upgrade(#[from] UpgradeError),
    }

    #[test]
    fn init_state_version_unset() {
        let mut deps = mock_dependencies();

        init_state_version(&mut deps.as_mut(), 1.try_into().unwrap()).unwrap();

        assert_storage_eq(
            &deps.storage,
            [(STATE_VERSION.to_vec(), [0, 0, 0, 1].to_vec())],
        );
    }

    #[test]
    fn init_state_version_set() {
        let mut deps = mock_dependencies();

        init_state_version(&mut deps.as_mut(), 1.try_into().unwrap()).unwrap();

        let err = init_state_version(&mut deps.as_mut(), 1.try_into().unwrap()).unwrap_err();

        assert_eq!(err, InitStateVersionError::AlreadyInitiated);
    }

    #[test]
    fn init_state_version_after_init() {
        let mut deps = mock_dependencies();

        UpgradeMsg::<(), ()>::Init(())
            .run::<_, Empty>(
                deps.as_mut(),
                |deps, ()| {
                    deps.storage.set(&[], &[1]);

                    Ok::<_, Error>((Response::new(), None))
                },
                |_, (), _| unreachable!(),
            )
            .unwrap();

        let err = init_state_version(&mut deps.as_mut(), 1.try_into().unwrap()).unwrap_err();

        assert_eq!(err, InitStateVersionError::AlreadyInitiated);
    }

    #[test]
    fn run_init() {
        let mut deps = mock_dependencies();

        let res = UpgradeMsg::<(), ()>::Init(())
            .run::<_, Empty>(
                deps.as_mut(),
                |deps, ()| {
                    deps.storage.set(&[], &[1]);

                    Ok::<_, Error>((Response::new(), None))
                },
                |_, (), _| unreachable!(),
            )
            .unwrap();

        assert_eq!(res, Response::new());

        assert_storage_eq(
            &deps.storage,
            [
                // ensure state version is set
                (STATE_VERSION.to_vec(), [0, 0, 0, 1].to_vec()),
                // ensure init was called
                ([].to_vec(), [1].to_vec()),
            ],
        );
    }

    #[test]
    fn run_migrate() {
        let mut deps = mock_dependencies();

        UpgradeMsg::<(), ()>::Init(())
            .run::<_, Empty>(
                deps.as_mut(),
                |deps, ()| {
                    deps.storage.set(&[], &[1]);

                    Ok::<_, Error>((Response::new(), None))
                },
                |_, (), _| unreachable!(),
            )
            .unwrap();

        let res = UpgradeMsg::<(), ()>::Migrate(())
            .run::<_, Empty>(
                deps.as_mut(),
                |_, ()| unreachable!(),
                |deps, (), _version| {
                    deps.storage.set(&[1], &[1]);

                    Ok::<_, Error>((Response::new(), None))
                },
            )
            .unwrap();

        assert_eq!(res, Response::new());

        assert_storage_eq(
            &deps.storage,
            [
                // ensure state version is set
                (STATE_VERSION.to_vec(), [0, 0, 0, 1].to_vec()),
                // ensure init was called
                ([].to_vec(), [1].to_vec()),
                // ensure migrate was called
                ([1].to_vec(), [1].to_vec()),
            ],
        );
    }

    #[test]
    fn run_init_already_initiated() {
        let mut deps = mock_dependencies();

        UpgradeMsg::<(), ()>::Init(())
            .run::<_, Empty>(
                deps.as_mut(),
                |deps, ()| {
                    deps.storage.set(&[], &[1]);

                    Ok::<_, Error>((Response::new(), None))
                },
                |_, (), _| unreachable!(),
            )
            .unwrap();

        let err = UpgradeMsg::<(), ()>::Init(())
            .run::<Error, Empty>(
                deps.as_mut(),
                |_, ()| unreachable!(),
                |_, (), _| unreachable!(),
            )
            .unwrap_err();

        assert_eq!(err, Error::Upgrade(UpgradeError::AlreadyInitiated));
    }

    #[test]
    fn run_migrate_not_initiated() {
        let mut deps = mock_dependencies();

        let err = UpgradeMsg::<(), ()>::Migrate(())
            .run::<_, Empty>(
                deps.as_mut(),
                |_, ()| unreachable!(),
                |deps, (), _version| {
                    deps.storage.set(&[1], &[1]);

                    Ok::<_, Error>((Response::new(), None))
                },
            )
            .unwrap_err();

        assert_eq!(err, Error::Upgrade(UpgradeError::NotInitiated));
    }
}

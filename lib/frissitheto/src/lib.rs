#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs)]

use std::num::NonZeroU32;

use cosmwasm_std::{DepsMut, Response, StdError};
use serde::{Deserialize, Serialize};

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
    /// State is stored under `b"state_version"` at the contract root. Consumers of this library
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
    pub fn run<E: From<UpgradeError> + From<StdError>>(
        self,
        mut deps: DepsMut,
        init_f: impl FnOnce(DepsMut, Init) -> Result<(Response, Option<NonZeroU32>), E>,
        migrate_f: impl FnOnce(
            DepsMut,
            Migrate,
            NonZeroU32,
        ) -> Result<(Response, Option<NonZeroU32>), E>,
    ) -> Result<Response, E> {
        let state_version = deps.storage.get(b"state_version");
        match state_version {
            Some(state_version) => match self {
                UpgradeMsg::Init(_) => Err(UpgradeError::AlreadyInitiated.into()),
                UpgradeMsg::Migrate(migrate) => {
                    let current_state_version =
                        NonZeroU32::new(u32::from_be_bytes(state_version.try_into().unwrap()))
                            .unwrap();

                    let (res, new_version) =
                        migrate_f(deps.branch(), migrate, current_state_version)?;

                    if let Some(new_version) = new_version {
                        if new_version <= current_state_version {
                            return Err(UpgradeError::StateVersionMustIncreaseIfModified {
                                current: current_state_version,
                                new: new_version,
                            }
                            .into());
                        }
                    }

                    deps.storage.set(
                        b"state_version",
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
                        b"state_version",
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
#[allow(missing_docs, reason = "#[error] attributes provide documentation")]
pub enum UpgradeError {
    #[error("attempted to initiate a contract that was already initiated")]
    AlreadyInitiated,
    #[error("attempted to migrate a contract that was not yet initiated")]
    NotInitiated,
    #[error("unknown state version {0}")]
    UnknownStateVersion(NonZeroU32),
    #[error("the state version must increase if it is modified, attempted to migrate from {current} to {new}")]
    StateVersionMustIncreaseIfModified {
        current: NonZeroU32,
        new: NonZeroU32,
    },
}

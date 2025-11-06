use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError, StdResult, ensure,
    entry_point,
};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use ibc_union_light_client::{IbcClientError, msg::QueryMsg};
use serde::{Deserialize, Serialize};
use unionlabs::encoding::{Bincode, EncodeAs};

use crate::{
    client::AttestedLightClient,
    errors::Error,
    msg::{ExecuteMsg, InitMsg},
    state::{
        AttestationAttestors, Attestations, Attestors, HeightTimestamps, PendingAttestations,
        Quorum,
    },
    types::AttestationKey,
};

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::Attest {
            attestation,
            attestor,
            signature,
        } => {
            deps.storage
                .maybe_read::<Attestors>(&attestor)?
                .ok_or(Error::InvalidAttestor { attestor })?;

            if let Some(previously_attested_timestamp) = deps
                .storage
                .maybe_read::<HeightTimestamps>(&attestation.height)?
                && previously_attested_timestamp != attestation.timestamp
            {
                return Err(Error::InconsistentTimestamp {
                    height: attestation.height,
                    timestamp: attestation.timestamp,
                    previously_attested_timestamp,
                });
            }

            let attestation_key = AttestationKey {
                height: attestation.height,
                key: attestation.key.clone(),
            };

            if let Some(value) = deps.storage.maybe_read::<Attestations>(&attestation_key)? {
                return Err(Error::AlreadyAttested {
                    height: attestation.height,
                    timestamp: attestation.timestamp,
                    key: attestation.key,
                    value,
                });
            }

            ensure!(
                deps.api
                    .ed25519_verify(
                        &(&attestation).encode_as::<Bincode>(),
                        signature.as_ref(),
                        attestor.as_ref()
                    )
                    .map_err(StdError::from)?,
                Error::InvalidSignature
            );

            let mut signatures = deps
                .storage
                .maybe_read::<PendingAttestations>(&attestation)?
                .unwrap_or_default();

            if signatures.insert(attestor, signature).is_some() {
                return Err(Error::AttestationAlreadyReceived);
            }

            let quorum = deps.storage.read_item::<Quorum>()?;

            let mut res = Response::new().add_event(
                Event::new("attestation_submitted")
                    .add_attribute("height", attestation.height.to_string())
                    .add_attribute("timestamp", attestation.timestamp.to_string())
                    .add_attribute("key", attestation.key.to_string())
                    .add_attribute("value", attestation.value.to_string())
                    .add_attribute("attestor", attestor.to_string())
                    .add_attribute("signature", signature.to_string()),
            );

            if signatures.len() >= quorum.get().into() {
                deps.storage.delete::<PendingAttestations>(&attestation);

                deps.storage
                    .write::<Attestations>(&attestation_key, &attestation.value);
                deps.storage
                    .write::<AttestationAttestors>(&attestation, &signatures);

                deps.storage
                    .upsert::<HeightTimestamps, Error>(&attestation.height, |maybe_timestamp| {
                        Ok(maybe_timestamp.unwrap_or(attestation.timestamp))
                    })?;

                res = res.add_event(
                    Event::new("quorum_reached")
                        .add_attribute("height", attestation.height.to_string())
                        .add_attribute("timestamp", attestation.timestamp.to_string())
                        .add_attribute("key", attestation.key.to_string())
                        .add_attribute("value", attestation.value.to_string())
                        .add_attribute("quorum", quorum.to_string()),
                );
            } else {
                deps.storage
                    .write::<PendingAttestations>(&attestation, &signatures);
            }

            Ok(res)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    ibc_union_light_client::query::<AttestedLightClient>(deps, env, msg).map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, IbcClientError<AttestedLightClient>> {
    msg.run(
        deps,
        |deps, init_msg| {
            for key in init_msg.attestors {
                deps.storage.write::<Attestors>(&key, &());
            }

            deps.storage.write_item::<Quorum>(&init_msg.quorum);

            let res = ibc_union_light_client::init(deps, init_msg.ibc_union_light_client_init_msg)?;

            Ok((res, None))
        },
        |_deps, _migrate_msg, _current_version| Ok((Response::default(), None)),
    )
}

use std::{collections::BTreeMap, num::NonZero};

use cosmwasm_std::{DepsMut, Event, Response, StdError, ensure};
use depolama::StorageExt;
use unionlabs::{
    encoding::{Bincode, EncodeAs},
    primitives::{H256, H512},
};

#[cfg(doc)]
use crate::msg::{ExecuteMsg, RestrictedExecuteMsg};
use crate::{
    errors::Error,
    state::{
        AttestationAttestors, Attestations, Attestors, HeightTimestamps, PendingAttestations,
        Quorum,
    },
    types::{Attestation, AttestationKey},
};

/// See [`ExecuteMsg::Attest`].
pub fn attest(
    mut deps: DepsMut,
    attestation: Attestation,
    attestor: H256,
    signature: H512,
) -> Result<Response, Error> {
    deps.storage
        .maybe_read::<Attestors>(&(attestation.chain_id.clone(), attestor))?
        .ok_or_else(|| Error::InvalidAttestor {
            chain_id: attestation.chain_id.clone(),
            attestor,
        })?;

    if let Some(previously_attested_timestamp) = deps
        .storage
        .maybe_read::<HeightTimestamps>(&(attestation.chain_id.clone(), attestation.height))?
        && previously_attested_timestamp != attestation.timestamp
    {
        return Err(Error::InconsistentTimestamp {
            chain_id: attestation.chain_id.clone(),
            height: attestation.height,
            timestamp: attestation.timestamp,
            previously_attested_timestamp,
        });
    }

    let attestation_key = AttestationKey {
        chain_id: attestation.chain_id.clone(),
        height: attestation.height,
        key: attestation.key.clone(),
    };

    if let Some(value) = deps.storage.maybe_read::<Attestations>(&attestation_key)? {
        return Err(Error::AlreadyAttested {
            chain_id: attestation.chain_id.clone(),
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
        return Err(Error::AttestationAlreadyReceived {
            chain_id: attestation.chain_id.clone(),
        });
    }

    let mut res = Response::new().add_event(
        Event::new("attestation_submitted")
            .add_attribute("chain_id", attestation.chain_id.clone())
            .add_attribute("height", attestation.height.to_string())
            .add_attribute("timestamp", attestation.timestamp.to_string())
            .add_attribute("key", attestation.key.to_string())
            .add_attribute("value", attestation.value.to_string())
            .add_attribute("attestor", attestor.to_string())
            .add_attribute("signature", signature.to_string()),
    );

    if let Ok(event) = check_quorum(deps.branch(), &signatures, &attestation)? {
        res = res.add_event(event);
    } else {
        deps.storage
            .write::<PendingAttestations>(&attestation, &signatures);
    }

    Ok(res)
}

pub fn confirm_attestation(deps: DepsMut, attestation: Attestation) -> Result<Response, Error> {
    let signatures = deps
        .storage
        .maybe_read::<PendingAttestations>(&attestation)?
        .unwrap_or_default();

    let event = check_quorum(deps, &signatures, &attestation)?.map_err(|(quorum, current)| {
        Error::QuorumNotReached {
            chain_id: attestation.chain_id,
            quorum,
            current,
        }
    })?;

    Ok(Response::new().add_event(event))
}

/// Check if the quorum has been reached for the provided `attestation` with the provided `signatures`.
///
/// This function will return `Ok(Ok(Event))` if the quorum was reached, containing the `quorum_reached` event. Additionally, the contract state will be updated:
/// - The confirmed attestation will be removed from [`PendingAttestations`], and will be written to [`Attestations`]
/// - [`AttestationAttestors`] will be updated to contain `signatures`
/// - [`HeightTimestamps`] will be updated to contain the attestation timestamp at the attestation height, if it is not already present.
///
/// In the case that the quorum is not reached, `Ok(Err(quorum, total_valid_signatures))` will be returned. This can be safely ignored if the quorum is not expected to be reached.
///
/// # Errors
///
/// This function will return an error if any storage reads fail.
fn check_quorum(
    deps: DepsMut,
    signatures: &BTreeMap<H256, H512>,
    attestation: &Attestation,
) -> Result<Result<Event, (NonZero<u8>, u8)>, Error> {
    let quorum = deps.storage.read::<Quorum>(&attestation.chain_id)?;

    // TODO: Add a test that checks for wrapping behaviour
    let total_valid_signatures = signatures.iter().try_fold(0_u8, |total, (attestor, _)| {
        deps.storage
            .maybe_read::<Attestors>(&(attestation.chain_id.clone(), *attestor))
            .map(|exists| total.wrapping_add(exists.is_some() as u8))
    })?;

    if total_valid_signatures >= quorum.get() {
        deps.storage.delete::<PendingAttestations>(attestation);

        deps.storage.write::<Attestations>(
            &AttestationKey {
                chain_id: attestation.chain_id.clone(),
                height: attestation.height,
                key: attestation.key.clone(),
            },
            &attestation.value,
        );
        deps.storage
            .write::<AttestationAttestors>(attestation, signatures);

        deps.storage.upsert::<HeightTimestamps, Error>(
            &(attestation.chain_id.clone(), attestation.height),
            |maybe_timestamp| match maybe_timestamp {
                Some(timestamp) => {
                    assert_eq!(
                        timestamp, attestation.timestamp,
                        "invariant: attestations with inconsistent heights cannot be created"
                    );
                    Ok(timestamp)
                }
                None => Ok(attestation.timestamp),
            },
        )?;

        Ok(Ok(Event::new("quorum_reached")
            .add_attribute("chain_id", attestation.chain_id.clone())
            .add_attribute("height", attestation.height.to_string())
            .add_attribute("timestamp", attestation.timestamp.to_string())
            .add_attribute("key", attestation.key.to_string())
            .add_attribute("value", attestation.value.to_string())
            .add_attribute("quorum", quorum.to_string())))
    } else {
        Ok(Err((quorum, total_valid_signatures)))
    }
}

/// See [`RestrictedExecuteMsg::SetQuorum`].
///
/// Note that permissions are not checked in this function, and must be checked by the caller.
pub fn set_quorum(
    deps: DepsMut,
    chain_id: String,
    new_quorum: NonZero<u8>,
) -> Result<Response, Error> {
    deps.storage.write::<Quorum>(&chain_id, &new_quorum);

    Ok(Response::new().add_event(
        Event::new("quorum_updated")
            .add_attribute("chain_id", chain_id)
            .add_attribute("quorum", new_quorum.to_string()),
    ))
}

/// See [`RestrictedExecuteMsg::AddAttestor`].
///
/// Note that permissions are not checked in this function, and must be checked by the caller.
pub fn add_attestor(
    deps: DepsMut,
    chain_id: String,
    new_attestor: H256,
) -> Result<Response, Error> {
    let attestor_key = (chain_id.clone(), new_attestor);

    if deps
        .storage
        .maybe_read::<Attestors>(&attestor_key)?
        .is_some()
    {
        Err(Error::AttestorAlreadyExists {
            chain_id,
            attestor: new_attestor,
        })
    } else {
        deps.storage.write::<Attestors>(&attestor_key, &());

        Ok(Response::new().add_event(
            Event::new("attestor_added")
                .add_attribute("chain_id", chain_id)
                .add_attribute("attestor", new_attestor.to_string()),
        ))
    }
}

/// See [`RestrictedExecuteMsg::RemoveAttestor`].
///
/// Note that permissions are not checked in this function, and must be checked by the caller.
pub fn remove_attestor(
    deps: DepsMut,
    chain_id: String,
    old_attestor: H256,
) -> Result<Response, Error> {
    if deps
        .storage
        .take::<Attestors>(&(chain_id.clone(), old_attestor))?
        .is_some()
    {
        Ok(Response::new().add_event(
            Event::new("attestor_removed")
                .add_attribute("chain_id", chain_id)
                .add_attribute("attestor", old_attestor.to_string()),
        ))
    } else {
        Err(Error::InvalidAttestor {
            chain_id,
            attestor: old_attestor,
        })
    }
}

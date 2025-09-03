// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cosmwasm/lst subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use cosmwasm_std::{Addr, StdError, StdResult};
use depolama::{value::ValueCodecViaEncoding, KeyCodec, Prefix, Store, ValueCodec};
use unionlabs_encoding::Bincode;
use unionlabs_primitives::{ByteArrayExt, Bytes};

use crate::types::{
    AccountingState, BatchId, Config, PendingBatch, PendingOwner, ProtocolFeeConfig, ReceivedBatch,
    SubmittedBatch, UnstakeRequest, UnstakeRequestKey,
};

pub enum Stopped {}

impl Store for Stopped {
    const PREFIX: Prefix = Prefix::new(b"stopped");
    type Key = ();
    type Value = bool;
}

impl ValueCodecViaEncoding for Stopped {
    type Encoding = Bincode;
}

pub enum ConfigStore {}

impl Store for ConfigStore {
    const PREFIX: Prefix = Prefix::new(b"config");
    type Key = ();
    type Value = Config;
}

impl ValueCodecViaEncoding for ConfigStore {
    type Encoding = Bincode;
}

pub enum ProtocolFeeConfigStore {}

impl Store for ProtocolFeeConfigStore {
    const PREFIX: Prefix = Prefix::new(b"protocol_fee_config");
    type Key = ();
    type Value = ProtocolFeeConfig;
}

impl ValueCodecViaEncoding for ProtocolFeeConfigStore {
    type Encoding = Bincode;
}

pub enum AccountingStateStore {}

impl Store for AccountingStateStore {
    const PREFIX: Prefix = Prefix::new(b"state");
    type Key = ();
    type Value = AccountingState;
}

impl ValueCodecViaEncoding for AccountingStateStore {
    type Encoding = Bincode;
}

pub enum Monitors {}

impl Store for Monitors {
    const PREFIX: Prefix = Prefix::new(b"monitors");
    type Key = ();
    type Value = Vec<String>;
}

impl ValueCodecViaEncoding for Monitors {
    type Encoding = Bincode;
}

/// The address of the [`UCS03-ZKGM`] contract on this chain.
///
/// [`UCS03-ZKGM`]: https://docs.union.build/ucs/03
pub enum Zkgm {}

impl Store for Zkgm {
    const PREFIX: Prefix = Prefix::new(b"zkgm");
    type Key = ();
    type Value = Addr;
}

impl ValueCodec<Addr> for Zkgm {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

pub enum Admin {}

impl Store for Admin {
    const PREFIX: Prefix = Prefix::new(b"admin");
    type Key = ();
    type Value = Addr;
}

impl ValueCodec<Addr> for Admin {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

/// The address of the [`on-zkgm-call-proxy`] contract. This is checked when executing the
/// [`OnProxyOnZkgmCall`](on_zkgm_call_proxy::OnProxyOnZkgmCall) message.
///
/// [`on-zkgm-call-proxy`]: https://github.com/unionlabs/union/tree/758d66edd45a47861773a1ca74ef9e8a2ea24961/cosmwasm/on-zkgm-call-proxy
pub enum OnZkgmCallProxy {}

impl Store for OnZkgmCallProxy {
    const PREFIX: Prefix = Prefix::new(b"on_zkgm_call_proxy");
    type Key = ();
    type Value = Addr;
}

impl ValueCodec<Addr> for OnZkgmCallProxy {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

/// Address of the account that is performing the delegation.
///
/// This contract is an implementation of [TODO: LINK CONTRACT CODE HERE]
pub enum StakerAddress {}

impl Store for StakerAddress {
    const PREFIX: Prefix = Prefix::new(b"staker_address");
    type Key = ();
    type Value = Addr;
}

impl ValueCodec<Addr> for StakerAddress {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

/// The address of the LST CW20 contract.
pub enum LstAddress {}

impl Store for LstAddress {
    const PREFIX: Prefix = Prefix::new(b"lst_address");
    type Key = ();
    type Value = Addr;
}

impl ValueCodec<Addr> for LstAddress {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

pub enum SubmittedBatches {}

impl Store for SubmittedBatches {
    const PREFIX: Prefix = Prefix::new(b"submitted_batches");
    type Key = BatchId;
    type Value = SubmittedBatch;
}

// big endian for iteration to work correctly
impl KeyCodec<BatchId> for SubmittedBatches {
    fn encode_key(key: &BatchId) -> Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<BatchId> {
        BatchId::try_from_be_bytes(raw)
    }
}

impl ValueCodecViaEncoding for SubmittedBatches {
    type Encoding = Bincode;
}

pub enum ReceivedBatches {}

impl Store for ReceivedBatches {
    const PREFIX: Prefix = Prefix::new(b"received_batches");
    type Key = BatchId;
    type Value = ReceivedBatch;
}

// big endian for iteration to work correctly
impl KeyCodec<BatchId> for ReceivedBatches {
    fn encode_key(key: &BatchId) -> Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<BatchId> {
        BatchId::try_from_be_bytes(raw)
    }
}

impl ValueCodecViaEncoding for ReceivedBatches {
    type Encoding = Bincode;
}

pub enum CurrentPendingBatch {}

impl Store for CurrentPendingBatch {
    const PREFIX: Prefix = Prefix::new(b"pending_batch");
    type Key = ();
    type Value = PendingBatch;
}

// big endian for iteration to work correctly
impl ValueCodec<BatchId> for CurrentPendingBatch {
    fn encode_value(value: &BatchId) -> Bytes {
        value.to_be_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<BatchId> {
        BatchId::try_from_be_bytes(raw)
    }
}

impl ValueCodecViaEncoding for CurrentPendingBatch {
    type Encoding = Bincode;
}

pub enum UnstakeRequests {}

impl Store for UnstakeRequests {
    const PREFIX: Prefix = Prefix::new(b"unstake_requests");

    type Key = UnstakeRequestKey;

    type Value = UnstakeRequest;
}

impl KeyCodec<UnstakeRequestKey> for UnstakeRequests {
    fn encode_key(key: &UnstakeRequestKey) -> Bytes {
        [
            key.batch_id.get().get().to_be_bytes().as_slice(),
            key.staker_hash.get().as_slice(),
        ]
        .concat()
        .into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<UnstakeRequestKey> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected 40 bytes, found {}: {raw}",
                    raw.len(),
                ))
            })
            .and_then(|arr: [u8; 40]| {
                Ok(UnstakeRequestKey {
                    batch_id: BatchId::from_be_bytes(arr.array_slice::<0, 8>())?,
                    staker_hash: arr.array_slice::<8, 32>().into(),
                })
            })
    }
}

impl ValueCodecViaEncoding for UnstakeRequests {
    type Encoding = Bincode;
}

/// Compliment to [`UnstakeRequests`], but keyed by the staker hash.
pub enum UnstakeRequestsByStakerHash {}

impl Store for UnstakeRequestsByStakerHash {
    const PREFIX: Prefix = Prefix::new(b"unstake_requests_by_staker_hash");

    type Key = UnstakeRequestKey;

    type Value = UnstakeRequest;
}

impl KeyCodec<UnstakeRequestKey> for UnstakeRequestsByStakerHash {
    fn encode_key(key: &UnstakeRequestKey) -> Bytes {
        [
            key.staker_hash.get().as_slice(),
            key.batch_id.get().get().to_be_bytes().as_slice(),
        ]
        .concat()
        .into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<UnstakeRequestKey> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected 40 bytes, found {}: {raw}",
                    raw.len(),
                ))
            })
            .and_then(|arr: [u8; 40]| {
                Ok(UnstakeRequestKey {
                    batch_id: BatchId::from_be_bytes(arr.array_slice::<32, 8>())?,
                    staker_hash: arr.array_slice::<0, 32>().into(),
                })
            })
    }
}

impl ValueCodecViaEncoding for UnstakeRequestsByStakerHash {
    type Encoding = Bincode;
}

pub enum PendingOwnerStore {}

impl Store for PendingOwnerStore {
    const PREFIX: Prefix = Prefix::new(b"pending_owner");
    type Key = ();
    type Value = PendingOwner;
}

impl ValueCodecViaEncoding for PendingOwnerStore {
    type Encoding = Bincode;
}

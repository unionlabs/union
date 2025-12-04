// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use alexandria_math::bitmap::Bitmap;
use alexandria_math::opt_math::OptBitShift;
use core::hash::{Hash, HashStateTrait};
use core::pedersen::PedersenTrait;
use ibc::types::{ChannelId, Id};
use starknet::ContractAddress;
use types::{TokenMetadata, ethabi_encode};

pub mod contract;

pub mod event;
pub mod isolver;
pub mod izkgmerc20;
pub mod types;

const TOKEN_ORDER_KIND_INITIALIZE: u8 = 0x00;
const TOKEN_ORDER_KIND_ESCROW: u8 = 0x01;
const TOKEN_ORDER_KIND_UNESCROW: u8 = 0x02;
const TOKEN_ORDER_KIND_SOLVE: u8 = 0x03;

const WRAPPED_TOKEN_KIND_PROTOCOL: u8 = 0x00;
const WRAPPED_TOKEN_KIND_THIRD_PARTY: u8 = 0x01;

const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;

const INSTR_VERSION_0: u8 = 0x00;
const INSTR_VERSION_1: u8 = 0x01;
const INSTR_VERSION_2: u8 = 0x02;

// TODO(aeryz): do we need to hash this?
const ACK_ERR_ONLYMAKER: u256 = 0xdeadc0de;


pub fn predict_wrapped_token(
    path: u256, channel: ChannelId, token: ByteArray, metadata: TokenMetadata, with_calldata: bool,
) -> (ContractAddress, felt252, Array<felt252>) {
    let metadata_image = ethabi_encode(@metadata).keccak_be();

    predict_wrapped_token_from_metadata_and_image(
        path, channel, token, metadata, metadata_image, with_calldata,
    )
}

pub fn predict_wrapped_token_from_metadata_and_image(
    path: u256,
    channel: ChannelId,
    token: ByteArray,
    metadata: TokenMetadata,
    metadata_image: u256,
    with_calldata: bool,
) -> (ContractAddress, felt252, Array<felt252>) {
    const PATRICIA_KEY_UPPER_BOUND: u256 =
        0x800000000000000000000000000000000000000000000000000000000000000;
    const CONTRACT_ADDRESS_DOMAIN_SIZE: u256 = PATRICIA_KEY_UPPER_BOUND;
    const L2_ADDRESS_UPPER_BOUND: u256 = CONTRACT_ADDRESS_DOMAIN_SIZE - 256;

    let (_, class_hash) = metadata.implementation.read_felt252(0);

    let mut offset = 0;
    let mut calldata_hash = PedersenTrait::new(0);
    let mut calldata: Array<felt252> = Default::default();
    while offset < metadata.initializer.len() {
        let (o, i) = metadata.initializer.read_felt252(offset);
        offset = o;
        calldata_hash = calldata_hash.update(i);
        if (with_calldata) {
            calldata.append(i);
        }
    }
    // NOTE(aeryz): `pedersen_hash_array` function always postfixes the values with
    // the given array length.
    // https://docs.rs/crate/starknet-types-core/latest/source/src/hash/pedersen.rs#23
    let calldata_hash = calldata_hash.update((offset / 32).try_into().unwrap()).finalize();

    let mut salt = PedersenTrait::new(0)
        .update(path.low.into())
        .update(path.high.into())
        .update(channel.raw().into())
        .update(metadata_image.low.try_into().unwrap())
        .update(metadata_image.high.try_into().unwrap());

    for i in token {
        salt = salt.update(i.into());
    }

    let salt = salt.finalize();

    let mut address = PedersenTrait::new(0)
        .update(0x535441524b4e45545f434f4e54524143545f41444452455353)
        .update(0) // deployer is empty
        .update(salt)
        .update(class_hash) // class hash
        .update(calldata_hash)
        // Postfix with the array length
        .update(5)
        .finalize();

    // TODO(aeryz): check this logic one more time
    // safu because its guaranteed to be smaller than the felt252::Max
    address = (address.into() % L2_ADDRESS_UPPER_BOUND).try_into().unwrap();

    (address.try_into().unwrap(), salt, calldata)
}

pub fn reverse_channel_path(mut path: u256) -> Result<u256, ()> {
    let mut reversed_path = 0;

    loop {
        let (tail, head) = pop_channel_from_path(path);
        reversed_path = update_channel_path(reversed_path, head)?;
        path = tail;

        if path == 0 {
            return Ok(reversed_path);
        }
    }
}

pub fn pop_channel_from_path(path: u256) -> (u256, u32) {
    if path == 0 {
        return (0, 0);
    }

    let current_hop_index = Bitmap::most_significant_bit(path).unwrap() / 32;
    let clear_shift = (8 - current_hop_index) * 32;

    return (
        OptBitShift::shr(OptBitShift::shl(path, clear_shift), clear_shift),
        OptBitShift::shr(path, current_hop_index * 32).try_into().unwrap(),
    );
}


pub fn update_channel_path(path: u256, next_channel_id: u32) -> Result<u256, ()> {
    let next_channel_id = next_channel_id.into();
    if path == 0 {
        return Ok(next_channel_id);
    }

    // unwrap since this is guaranteed to be nonzero
    let next_hop_index = Bitmap::most_significant_bit(path).unwrap() / 32 + 1;
    if next_hop_index > 7 {
        // InvalidHops
        return Err(());
    }

    Ok((OptBitShift::shl(next_channel_id, 32) & next_hop_index.into()) | path)
}


#[cfg(test)]
mod tests {
    use snforge_std::{DeclareResultTrait, declare};
    use starknet::syscalls::deploy_syscall;
    use super::{ByteArrayTraitExt, Id, TokenMetadata};

    #[derive(Serde, Drop)]
    struct ConstructorArgs {
        name: ByteArray,
        symbol: ByteArray,
        decimals: u8,
    }

    #[test]
    fn test_address_prediction_works() {
        let contract = declare("Ucs03Zkgm").unwrap().contract_class();

        let args = ConstructorArgs { name: "Union Token", symbol: "U", decimals: 18 };

        let mut out = array![];
        args.serialize(ref out);

        let mut implementation: ByteArray = Default::default();
        implementation.append_felt252((*contract.class_hash).try_into().unwrap());

        let mut initializer = Default::default();
        for o in @out {
            initializer.append_felt252(*o);
        }

        let metadata = TokenMetadata { initializer, implementation };

        let (address, salt, _) = super::predict_wrapped_token_from_metadata_and_image(
            0, Id::new(1_u32.try_into().unwrap()), "AAAAAA", metadata, 100, false,
        );

        let (got, _) = deploy_syscall(*contract.class_hash, salt, out.into(), true).unwrap();

        assert!(got == address);
    }
}

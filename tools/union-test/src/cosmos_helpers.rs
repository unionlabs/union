use std::str::FromStr;

use alloy_sol_types::SolValue as _;
use cosmwasm_std::{instantiate2_address, Addr, CanonicalAddr};
use hex_literal::hex;
use unionlabs::{
    ethereum::keccak256,
    primitives::{Bech32, FixedBytes},
};
use voyager_sdk::anyhow;

pub const COSMOS_BASE_CONTRACT_HASH: [u8; 32] =
    hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1");

pub const SALT_IBC_CORE: &[u8] = b"ibc-is-based";
pub const SALT_ZKGM: &[u8] = b"protocols/ucs03";
pub const SALT_ESCROW_VAULT: &[u8] =
    &hex!("50bbead29d10abe51a7c32bbc02a9b00ff4a7db57c050b7a0ff61d6173c33965");
pub const SALT_LST_HUB: &[u8] = b"apps/lst";
pub const SALT_EU: &[u8] = b"tokens/eu";
pub const SALT_LST_STAKER: &[u8] = b"apps/lst-staker";

pub fn calculate_cosmos_contract_address(creator: &str, salt: &[u8]) -> anyhow::Result<Addr> {
    let bech_addr: Bech32 = Bech32::from_str(creator).unwrap();

    let addr = instantiate2_address(
        &COSMOS_BASE_CONTRACT_HASH,
        &bech_addr.data().as_ref().into(),
        salt,
    )?;

    let bech_addr = Bech32::new(bech_addr.hrp(), addr.as_slice());

    Ok(Addr::unchecked(bech_addr.to_string()))
}

pub fn calculate_proxy_address(
    zkgm_address: &Addr,
    path: alloy::primitives::U256,
    channel_id: u32,
    sender: &[u8],
) -> Addr {
    let addr = Bech32::<FixedBytes<32>>::from_str(zkgm_address.as_str()).unwrap();
    let canonical_addr = instantiate2_address(
        &COSMOS_BASE_CONTRACT_HASH,
        &CanonicalAddr::from(addr.data().as_ref()),
        keccak256((path, channel_id, sender.to_vec()).abi_encode_params()).as_ref(),
    )
    .unwrap();

    Addr::unchecked(
        Bech32::<FixedBytes<32>>::new(
            addr.hrp().clone(),
            canonical_addr.as_slice().try_into().unwrap(),
        )
        .to_string(),
    )
}

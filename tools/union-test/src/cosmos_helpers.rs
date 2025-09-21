use std::str::FromStr;

use cosmwasm_std::{instantiate2_address, Addr};
use hex_literal::hex;
use unionlabs::primitives::Bech32;
use voyager_sdk::anyhow;

pub const COSMOS_BASE_CONTRACT_HASH: [u8; 32] =
    hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1");

pub const SALT_IBC_CORE: &[u8] = b"ibc-is-based";
pub const SALT_ZKGM: &[u8] = b"protocols/ucs03";
pub const SALT_ESCROW_VAULT: &[u8] =
    &hex!("50bbead29d10abe51a7c32bbc02a9b00ff4a7db57c050b7a0ff61d6173c33965");
pub const SALT_LST_HUB: &[u8] = b"apps/lst";
pub const SALT_EU: &[u8] = b"tokens/eu";

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

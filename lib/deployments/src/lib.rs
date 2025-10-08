#![warn(clippy::pedantic)]

use std::{collections::BTreeMap, fmt::Display, sync::LazyLock};

use serde::{Deserialize, Serialize};
use ucs04::UniversalChainId;
use unionlabs_primitives::{Bech32, H160, H256, encoding::HexUnprefixed};
use voyager_primitives::ClientType;

pub type Deployments<'a> = BTreeMap<UniversalChainId<'a>, Deployment>;

pub static DEPLOYMENTS: LazyLock<Deployments<'static>> = LazyLock::new(|| {
    serde_json::from_slice(include_bytes!("../../../deployments/deployments.json")).unwrap()
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "ibc_interface")]
#[allow(clippy::large_enum_variant)]
pub enum Deployment {
    #[serde(rename = "ibc-cosmwasm")]
    IbcCosmwasm {
        deployer: Bech32<H160>,
        core: DeployedContract<Bech32<H256>, IbcCosmwasmDeployedContractExtra>,
        lightclient:
            BTreeMap<ClientType, DeployedContract<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
        app: App<Bech32<H256>, IbcCosmwasmDeployedContractExtra, IbcCosmwasmUcs03Extra>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        u: Option<DeployedContract<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eu: Option<DeployedContract<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lst: Option<DeployedContract<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        on_zkgm_call_proxy:
            Option<DeployedContract<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
    },
    #[serde(rename = "ibc-solidity")]
    IbcSolidity {
        deployer: H160,
        sender: H160,
        manager: H160,
        multicall: DeployedContract<H160>,
        core: DeployedContract<H160>,
        lightclient: BTreeMap<ClientType, DeployedContract<H160>>,
        app: App<H160>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        u: Option<DeployedContract<H160>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eu: Option<DeployedContract<H160>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployedContract<Address, Extra = ()> {
    pub address: Address,
    pub height: u64,
    pub commit: Commit,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Commit {
    Dirty,
    Unknown,
    #[serde(untagged)]
    Hash(H160<HexUnprefixed>),
}

impl Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Commit::Dirty => f.write_str("dirty"),
            Commit::Unknown => f.write_str("unknown"),
            Commit::Hash(hash) => Display::fmt(hash, f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct App<Address, Ucs00Extra = (), Ucs03Extra = ()> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucs00: Option<DeployedContract<Address, Ucs00Extra>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucs03: Option<DeployedContract<Address, Ucs03Extra>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IbcCosmwasmDeployedContractExtra {
    #[serde(default)]
    pub code_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IbcCosmwasmUcs03Extra {
    #[serde(default)]
    pub code_id: u64,
    pub minter: Minter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Minter {
    #[serde(rename = "cw20")]
    Cw20 {
        address: Bech32<H256>,
        commit: Commit,
        #[serde(default)]
        code_id: u64,
    },
    #[serde(rename = "osmosis_tokenfactory")]
    OsmosisTokenfactory {
        address: Bech32<H256>,
        commit: Commit,
        #[serde(default)]
        code_id: u64,
    },
}

#[test]
fn deployments_json_valid() {
    dbg!(&*DEPLOYMENTS);

    println!("{}", serde_json::to_string(&*DEPLOYMENTS).unwrap());
}

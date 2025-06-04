#![warn(clippy::pedantic)]

use std::{collections::BTreeMap, sync::LazyLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ucs04::UniversalChainId;
use unionlabs_primitives::{encoding::HexUnprefixed, H160};
use voyager_primitives::{ChainId, ClientType, IbcInterface};

pub static DEPLOYMENTS: LazyLock<BTreeMap<UniversalChainId<'static>, Deployment>> =
    LazyLock::new(|| {
        serde_json::from_slice::<Vec<Value>>(include_bytes!(
            "../../../deployments/deployments.json"
        ))
        .unwrap()
        .into_iter()
        .map(|value| {
            (
                value["universal_chain_id"]
                    .as_str()
                    .unwrap()
                    .to_owned()
                    .parse()
                    .unwrap(),
                serde_json::from_value(value).unwrap(),
            )
        })
        .collect()
    });

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deployment {
    pub chain_id: ChainId,
    pub ibc_interface: IbcInterface,
    pub deployments: DeployedContracts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployedContracts {
    pub core: DeployedContract,
    pub lightclient: BTreeMap<ClientType, DeployedContract>,
    pub app: BTreeMap<App, DeployedContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployedContract {
    // just string for now until i figure out a way to type this better
    pub address: String,
    pub height: u64,
    pub commit: H160<HexUnprefixed>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum App {
    #[serde(rename = "ucs00")]
    Ucs00,
    #[serde(rename = "ucs03")]
    Ucs03,
    #[serde(rename = "ucs06")]
    Ucs06,
}

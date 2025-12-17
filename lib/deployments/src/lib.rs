#![warn(clippy::pedantic)]

use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

pub use embed_commit::Rev;
use serde::{Deserialize, Serialize};
pub use ucs04::UniversalChainId;
pub use unionlabs_primitives::{Bech32, Bytes, H160, H256};
pub use voyager_primitives::ClientType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Deployments<'a>(BTreeMap<UniversalChainId<'a>, Deployment>);

impl<'a> Deref for Deployments<'a> {
    type Target = BTreeMap<UniversalChainId<'a>, Deployment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Deployments<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "ibc_interface")]
#[allow(clippy::large_enum_variant)]
pub enum Deployment {
    #[serde(rename = "ibc-cosmwasm")]
    IbcCosmwasm {
        /// The address used to create the deterministic addresses.
        deployer: Bech32<H160>,
        contracts: BTreeMap<Bech32<H256>, DeployedContract<IbcCosmwasmDeployedContractExtra>>,
    },
    #[serde(rename = "ibc-solidity")]
    IbcSolidity {
        /// The `Deployer.sol` deployment on this chain.
        deployer: H160,
        /// The address used to create the deterministic addresses, via the `deployer`.
        sender: H160,
        contracts: BTreeMap<H160, DeployedContract>,
    },
    #[serde(rename = "ibc-move/sui")]
    IbcMoveSui {
        contracts: BTreeMap<String, DeployedContract>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DeployedContract<E = ()> {
    pub name: String,
    /// If this contract was init'd with the bytecode-base bytecode via `frissitheto`, this is the salt used in the initial instantiate2 call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salt: Option<Bytes>,
    /// The initial height this contract was deployed at.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub height: u64,
    /// The git rev of the unionlabs/union repo that this contract was deployed from.
    #[serde(default, skip_serializing_if = "Rev::is_unknown")]
    pub commit: Rev,
    #[serde(flatten)]
    pub extra: E,
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "serde")]
const fn is_zero(n: &u64) -> bool {
    *n == 0
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IbcCosmwasmDeployedContractExtra {
    #[serde(default)]
    pub code_id: u64,
}

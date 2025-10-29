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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "ibc_interface")]
#[allow(clippy::large_enum_variant)]
pub enum DeploymentV1 {
    #[serde(rename = "ibc-cosmwasm")]
    IbcCosmwasm {
        deployer: Bech32<H160>,
        manager: Option<Bech32<H256>>,
        core: DeployedContractV1<Bech32<H256>, IbcCosmwasmDeployedContractExtra>,
        lightclient: BTreeMap<
            ClientType,
            DeployedContractV1<Bech32<H256>, IbcCosmwasmDeployedContractExtra>,
        >,
        app: App<Bech32<H256>, IbcCosmwasmDeployedContractExtra, IbcCosmwasmUcs03Extra>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        u: Option<DeployedContractV1<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eu: Option<DeployedContractV1<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lst: Option<DeployedContractV1<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        on_zkgm_call_proxy:
            Option<DeployedContractV1<Bech32<H256>, IbcCosmwasmDeployedContractExtra>>,
    },
    #[serde(rename = "ibc-solidity")]
    IbcSolidity {
        deployer: H160,
        sender: H160,
        manager: H160,
        multicall: DeployedContractV1<H160>,
        core: DeployedContractV1<H160>,
        lightclient: BTreeMap<ClientType, DeployedContractV1<H160>>,
        app: App<H160>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        u: Option<DeployedContractV1<H160>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eu: Option<DeployedContractV1<H160>>,
    },
    #[serde(rename = "ibc-move/sui")]
    IbcMoveSui {
        core: DeployedContractV1<String>,
        app: App<String>,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployedContractV1<Address, Extra = ()> {
    pub address: Address,
    pub height: u64,
    pub commit: Rev,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct App<Address, Ucs00Extra = (), Ucs03Extra = ()> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucs00: Option<DeployedContractV1<Address, Ucs00Extra>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucs03: Option<DeployedContractV1<Address, Ucs03Extra>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
        commit: Rev,
        #[serde(default)]
        code_id: u64,
    },
    #[serde(rename = "osmosis_tokenfactory")]
    OsmosisTokenfactory {
        address: Bech32<H256>,
        commit: Rev,
        #[serde(default)]
        code_id: u64,
    },
}

#[test]
#[allow(clippy::too_many_lines)]
fn deployments_json_valid() {
    fn convert<E, A, B: Into<Bytes>>(
        d: DeployedContractV1<A, E>,
        name: impl std::fmt::Display,
        salt: impl Into<Option<B>>,
    ) -> (A, DeployedContract<E>) {
        (
            d.address,
            DeployedContract {
                height: d.height,
                commit: d.commit,
                name: name.to_string(),
                salt: Into::<Option<_>>::into(salt).map(Into::into),
                extra: d.extra,
            },
        )
    }

    let v1 = std::fs::read_to_string("../../deployments/deployments.json").unwrap();

    let v2 = serde_json::from_str::<BTreeMap<UniversalChainId, DeploymentV1>>(&v1).unwrap()
        .clone()
        .into_iter()
        .map(|d| {
            (
                d.0,
                match d.1 {
                    DeploymentV1::IbcCosmwasm {
                        deployer,
                        manager,
                        core,
                        lightclient,
                        app,
                        u,
                        eu,
                        lst,
                        on_zkgm_call_proxy: _,
                    } => Deployment::IbcCosmwasm {
                        deployer,
                        contracts: [
                            convert(core, "core", b"ibc-is-based")
                        ]
                            .into_iter()
                            .chain(manager.map(|manager|
                            convert(DeployedContractV1 { address: manager, height: 0, commit: Rev::Unknown, extra: IbcCosmwasmDeployedContractExtra { code_id: 0 }}, "manager", "0x7b1f7c3b93ff643023d63bbbe182a179922ad85a2aa0e03ef50170b591a7b752".parse::<Bytes>().unwrap()),
                            ))
                            .chain(u.map(|u| convert(u, "u", "0x50bbead29d10abe51a7c32bbc02a9b00ff4a7db57c050b7a0ff61d6173c33965".parse::<Bytes>().unwrap())))
                            .chain(eu.map(|eu| convert(eu, "eu", "0x6a9b711ce5d3749ece29463110b6164dbb28dda28902586bf66e86699d60bd4c".parse::<Bytes>().unwrap())))
                            .chain(lst.map(|lst| convert(lst, "lst-hub", b"lst/eu")))
                            .chain(lightclient.into_iter().map(|c| {
                                convert(
                                    c.1,
                                    format!("lightclients/{}", c.0),
                                    format!("lightclients/{}", c.0).into_bytes(),
                                )
                            }))
                            .chain(app.ucs03.into_iter().flat_map(|c| {
                                [
                                    convert(
                                        DeployedContractV1 {
                                            address: c.address,
                                            height: c.height,
                                            commit: c.commit,
                                            extra: IbcCosmwasmDeployedContractExtra {
                                                code_id: c.extra.code_id,
                                            },
                                        },
                                        "protocols/ucs03",
                                        b"protocols/ucs03",
                                    ),
                                    match c.extra.minter {
                                        Minter::Cw20 {
                                            address,
                                            commit,
                                            code_id,
                                        } => convert::<_, _, [u8; 0]>(
                                            DeployedContractV1 {
                                                address,
                                                height: 0,
                                                commit,
                                                extra: IbcCosmwasmDeployedContractExtra { code_id },
                                            },
                                            "protocols/ucs03/cw20-token-minter",
                                            None::<[u8; 0]>,
                                        ),
                                        Minter::OsmosisTokenfactory {
                                            address,
                                            commit,
                                            code_id,
                                        } => convert::<_, _, [u8; 0]>(
                                            DeployedContractV1 {
                                                address,
                                                height: 0,
                                                commit,
                                                extra: IbcCosmwasmDeployedContractExtra { code_id },
                                            },
                                            "protocols/ucs03/osmosis-tokenfactory-token-minter",
                                            None::<[u8; 0]>,
                                        ),
                                    },
                                ]
                            }))
                            .collect(),
                    },
                    DeploymentV1::IbcSolidity {
                        deployer,
                        sender,
                        manager,
                        multicall,
                        core,
                        lightclient,
                        app,
                        u,
                        eu,
                    } => {
                        Deployment::IbcSolidity {
                            deployer,
                            sender,
                            contracts: [
                                convert(DeployedContractV1 { address: manager, height: 0, commit: Rev::Unknown, extra: () }, "manager", b"manager"), 
                                convert(multicall, "multicall", b"lib/multicall-v2"), 
                                convert(core, "core", b"ibc-is-based")]
                                .into_iter()
                                .chain(u.map(|u| convert(u, "u", "0x12c206e42a6e7773c97d1f1b855d7848492f9e4e396b33fcf0172d6758e9b047".parse::<Bytes>().unwrap())))
                                .chain(eu.map(|eu| convert(eu, "eu", "0x0dec0db7b56214f189bc3d33052145c6d7558c6a7ee0da79e34bdd8a29d569c2".parse::<Bytes>().unwrap())))
                                .chain(lightclient.into_iter().map(|c| {
                                    convert(
                                        c.1,
                                        format!("lightclients/{}", c.0),
                                        format!("lightclients/{}", c.0).into_bytes(),
                                    )
                                }))
                                .chain(app.ucs03.into_iter().flat_map(|c| {
                                    [convert(c, "protocols/ucs03", b"protocols/ucs03")]
                                }))
                                .collect(),
                        }
                    }
                    DeploymentV1::IbcMoveSui { core, app } => {
                        Deployment::IbcMoveSui {
                             contracts: [convert::<_, _, [u8; 0]>(core, "core", None)]
                                 .into_iter()
                                 .chain(app.ucs03.map(|ucs03| convert::<_, _, [u8; 0]>(ucs03, "protocols/ucs03", None)))
                                 .collect()
                         }
                    }
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    println!("{}", serde_json::to_string_pretty(&v2).unwrap());

    std::fs::write(
        "../../deployments/deployments-v2.json",
        serde_json::to_string_pretty(&v2).unwrap(),
    )
    .unwrap();
}

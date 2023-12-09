use std::{collections::VecDeque, fmt::Debug, marker::PhantomData, ops::Div, sync::Arc};

use beacon_api::errors::{InternalServerError, NotFoundError};
use chain_utils::evm::{CometblsMiddleware, Evm};
use contracts::ibc_handler::{
    self, AcknowledgePacketCall, ChannelOpenAckCall, ChannelOpenConfirmCall, ChannelOpenInitCall,
    ChannelOpenTryCall, ConnectionOpenAckCall, ConnectionOpenConfirmCall, ConnectionOpenInitCall,
    ConnectionOpenTryCall, CreateClientCall, IBCHandler, RecvPacketCall, UpdateClientCall,
};
use ethers::{
    abi::AbiEncode,
    contract::{ContractError, EthCall},
    providers::{Middleware, ProviderError},
    types::{EIP1186ProofResponse, U256},
    utils::keccak256,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use futures::Future;
use lightclient::cometbls::{CometblsConfig, CometblsMainnet, CometblsMinimal};
use prost::Message;
use protos::union::ibc::lightclients::ethereum::v1 as ethereum_v1;
use serde::{Deserialize, Serialize};
use typenum::Unsigned;
use unionlabs::{
    ethereum::{
        beacon::{GenesisData, LightClientBootstrap, LightClientFinalityUpdate},
        config::{ChainSpec, Mainnet, Minimal},
    },
    hash::H160,
    ibc::{
        core::client::{height::Height, msg_update_client::MsgUpdateClient},
        lightclients::{
            ethereum::{
                self,
                account_proof::AccountProof,
                account_update::AccountUpdate,
                light_client_update::LightClientUpdate,
                trusted_sync_committee::{ActiveSyncCommittee, TrustedSyncCommittee},
            },
            wasm,
        },
    },
    traits::{
        Chain, ChainOf, ClientState, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf,
        LightClientBase,
    },
    EthAbi, IntoEthAbi, MaybeRecoverableError,
};

use crate::{
    aggregate,
    aggregate::{Aggregate, AnyAggregate, LightClientSpecificAggregate},
    data,
    data::{
        AcknowledgementProof, AnyData, ChannelEndProof, ClientConsensusStateProof,
        ClientStateProof, CommitmentProof, ConnectionProof, Data, LightClientSpecificData,
    },
    fetch,
    fetch::{AnyFetch, Fetch, FetchStateProof, FetchUpdateHeaders, LightClientSpecificFetch},
    identified, msg,
    msg::{AnyMsg, Msg, MsgUpdateClientData},
    seq,
    use_aggregate::{do_aggregate, IsAggregateData, UseAggregate},
    wait,
    wait::{AnyWait, Wait, WaitForTimestamp},
    AnyLightClientIdentified, DoAggregate, Identified, LightClient, RelayerMsg,
};

pub const EVM_REVISION_NUMBER: u64 = 0;

impl LightClient for CometblsMainnet {
    type BaseCounterparty = Self::Counterparty;

    type Data = CometblsDataMsg<Mainnet>;
    type Fetch = CometblsFetchMsg<Self, Mainnet>;
    type Aggregate = CometblsAggregateMsg<Self, Mainnet>;

    type MsgError = TxSubmitError;

    fn proof(&self, msg: FetchStateProof<Self>) -> RelayerMsg {
        fetch(
            self.chain().chain_id(),
            LightClientSpecificFetch::<Self>(CometblsFetchMsg::FetchGetProof(GetProof {
                path: msg.path,
                height: msg.at,
            })),
        )
    }

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_ {
        do_msg(self.chain(), msg)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<_, Self>(self.chain(), msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(self.chain(), update_info)
    }
}

impl LightClient for CometblsMinimal {
    type BaseCounterparty = Self::Counterparty;

    type Data = CometblsDataMsg<Minimal>;
    type Fetch = CometblsFetchMsg<Self, Minimal>;
    type Aggregate = CometblsAggregateMsg<Self, Minimal>;

    type MsgError = TxSubmitError;

    fn proof(&self, msg: FetchStateProof<Self>) -> RelayerMsg {
        fetch(
            self.chain().chain_id(),
            LightClientSpecificFetch::<Self>(CometblsFetchMsg::FetchGetProof(GetProof {
                path: msg.path,
                height: msg.at,
            })),
        )
    }

    fn msg(&self, msg: Msg<Self>) -> impl Future<Output = Result<(), Self::MsgError>> + '_ {
        do_msg(self.chain(), msg)
    }

    fn do_fetch(&self, msg: Self::Fetch) -> impl Future<Output = Vec<RelayerMsg>> + '_ {
        do_fetch::<_, Self>(self.chain(), msg)
    }

    fn generate_counterparty_updates(
        &self,
        update_info: FetchUpdateHeaders<Self>,
    ) -> Vec<RelayerMsg> {
        generate_counterparty_updates::<_, Self>(self.chain(), update_info)
    }
}

fn generate_counterparty_updates<C, L>(
    evm: &Evm<C>,
    update_info: FetchUpdateHeaders<L>,
) -> Vec<RelayerMsg>
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Data = CometblsDataMsg<C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,
    LightClientSpecificFetch<L>: From<CometblsFetchMsg<L, C>>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    [RelayerMsg::Aggregate {
        queue: [seq([fetch(
            evm.chain_id,
            LightClientSpecificFetch(CometblsFetchMsg::FetchFinalityUpdate(PhantomData)),
        )])]
        .into(),
        data: [].into(),
        receiver: aggregate(
            evm.chain_id,
            LightClientSpecificAggregate(CometblsAggregateMsg::MakeCreateUpdates(
                MakeCreateUpdatesData { req: update_info },
            )),
        ),
    }]
    .into()
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct CreateUpdateData<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    pub req: FetchUpdateHeaders<L>,
    pub currently_trusted_slot: u64,
    pub light_client_update: LightClientUpdate<C>,
    pub is_next: bool,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MakeCreateUpdatesData<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    pub req: FetchUpdateHeaders<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MakeCreateUpdatesFromLightClientUpdatesData<
    L: LightClient<HostChain = Evm<C>>,
    C: ChainSpec,
> {
    pub req: FetchUpdateHeaders<L>,
    pub trusted_height: Height,
    pub finality_update: LightClientFinalityUpdate<C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchLightClientUpdate<C: ChainSpec> {
    pub period: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchLightClientUpdates<C: ChainSpec> {
    pub trusted_period: u64,
    pub target_period: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchBootstrap<C: ChainSpec> {
    pub slot: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchAccountUpdate<C: ChainSpec> {
    pub slot: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchBeaconGenesis<C: ChainSpec> {
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct BootstrapData<C: ChainSpec> {
    pub slot: u64,
    pub bootstrap: LightClientBootstrap<C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AccountUpdateData<C: ChainSpec> {
    pub slot: u64,
    pub ibc_handler_address: H160,
    pub update: EIP1186ProofResponse,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct BeaconGenesisData<C: ChainSpec> {
    genesis: GenesisData,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> C>,
}

try_from_relayer_msg! {
    #[CometblsMinimal(
        lc_msg(
            msg = Data(LightClientSpecificData),
            ty = CometblsDataMsg,
            variants(
                FinalityUpdate(FinalityUpdate<Minimal>),
                LightClientUpdates(LightClientUpdates<Minimal>),
                LightClientUpdate(LightClientUpdate<Minimal>),
                Bootstrap(BootstrapData<Minimal>),
                AccountUpdate(AccountUpdateData<Minimal>),
                BeaconGenesis(BeaconGenesisData<Minimal>),
            ),
        ),
        lc_msg(
            msg = Fetch(LightClientSpecificFetch),
            ty = CometblsFetchMsg,
            variants(
                FetchFinalityUpdate(PhantomData<Minimal>),
                FetchLightClientUpdates(FetchLightClientUpdates<Minimal>),
                FetchLightClientUpdate(FetchLightClientUpdate<Minimal>),
                FetchBootstrap(FetchBootstrap<Minimal>),
                FetchAccountUpdate(FetchAccountUpdate<Minimal>),
            ),
        ),
        // lc_msg(
        //     msg = Aggregate(LightClientSpecificAggregate),
        //     ty = CometblsAggregateMsg,
        //     variants(
        //         CreateUpdate(CreateUpdateData<CometblsMinimal, Minimal>),
        //         MakeCreateUpdates(MakeCreateUpdatesData<CometblsMinimal, Minimal>),
        //         MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<CometblsMinimal, Minimal>),
        //     ),
        // ),
    )]
}

try_from_relayer_msg! {
    #[CometblsMainnet(
        lc_msg(
            msg = Data(LightClientSpecificData),
            ty = CometblsDataMsg,
            variants(
                FinalityUpdate(FinalityUpdate<Mainnet>),
                LightClientUpdates(LightClientUpdates<Mainnet>),
                LightClientUpdate(LightClientUpdate<Mainnet>),
                Bootstrap(BootstrapData<Mainnet>),
                AccountUpdate(AccountUpdateData<Mainnet>),
                BeaconGenesis(BeaconGenesisData<Mainnet>),
            ),
        ),
        lc_msg(
            msg = Fetch(LightClientSpecificFetch),
            ty = CometblsFetchMsg,
            variants(
                FetchFinalityUpdate(PhantomData<Mainnet>),
                FetchLightClientUpdates(FetchLightClientUpdates<Mainnet>),
                FetchLightClientUpdate(FetchLightClientUpdate<Mainnet>),
                FetchBootstrap(FetchBootstrap<Mainnet>),
                FetchAccountUpdate(FetchAccountUpdate<Mainnet>),
                FetchBeaconGenesis(FetchBeaconGenesis<Mainnet>),
            ),
        ),
        // lc_msg(
        //     msg = Aggregate(LightClientSpecificAggregate),
        //     ty = CometblsAggregateMsg,
        //     variants(
        //         CreateUpdate(CreateUpdateData<CometblsMainnet, Mainnet>),
        //         MakeCreateUpdates(MakeCreateUpdatesData<CometblsMainnet, Mainnet>),
        //         MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<CometblsMainnet, Mainnet>),
        //     ),
        // ),
    )]
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum CometblsFetchMsg<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    #[display(fmt = "FetchFinalityUpdate")]
    FetchFinalityUpdate(PhantomData<C>),
    #[display(fmt = "FetchLightClientUpdates")]
    FetchLightClientUpdates(FetchLightClientUpdates<C>),
    #[display(fmt = "FetchLightClientUpdate")]
    FetchLightClientUpdate(FetchLightClientUpdate<C>),
    #[display(fmt = "FetchBootstrap")]
    FetchBootstrap(FetchBootstrap<C>),
    #[display(fmt = "FetchAccountUpdate")]
    FetchAccountUpdate(FetchAccountUpdate<C>),
    #[display(fmt = "FetchBeaconGenesis")]
    FetchBeaconGenesis(FetchBeaconGenesis<C>),
    #[display(fmt = "FetchGetProof::{}", "_0.path")]
    FetchGetProof(GetProof<C, L>),
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum CometblsDataMsg<C: ChainSpec> {
    #[display(fmt = "FinalityUpdate")]
    FinalityUpdate(FinalityUpdate<C>),
    #[display(fmt = "LightClientUpdates")]
    LightClientUpdates(LightClientUpdates<C>),
    #[display(fmt = "LightClientUpdate")]
    LightClientUpdate(LightClientUpdate<C>),
    #[display(fmt = "Bootstrap")]
    Bootstrap(BootstrapData<C>),
    #[display(fmt = "AccountUpdate")]
    AccountUpdate(AccountUpdateData<C>),
    #[display(fmt = "BeaconGenesis")]
    BeaconGenesis(BeaconGenesisData<C>),
}

impl<C, L> From<FinalityUpdate<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: FinalityUpdate<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::FinalityUpdate(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for FinalityUpdate<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::FinalityUpdate(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<LightClientUpdates<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: LightClientUpdates<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(
            CometblsDataMsg::LightClientUpdates(value),
        ))
    }
}

impl<C, L> From<LightClientUpdate<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: LightClientUpdate<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::LightClientUpdate(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for LightClientUpdates<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::LightClientUpdates(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> TryFrom<Data<L>> for LightClientUpdate<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::LightClientUpdate(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<BootstrapData<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: BootstrapData<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::Bootstrap(value)))
    }
}

impl<C, L> TryFrom<Data<L>> for BootstrapData<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::Bootstrap(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<AccountUpdateData<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: AccountUpdateData<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::AccountUpdate(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for AccountUpdateData<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::AccountUpdate(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

impl<C, L> From<BeaconGenesisData<C>> for Data<L>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    fn from(value: BeaconGenesisData<C>) -> Self {
        Data::LightClientSpecific(LightClientSpecificData(CometblsDataMsg::BeaconGenesis(
            value,
        )))
    }
}

impl<C, L> TryFrom<Data<L>> for BeaconGenesisData<C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Data = CometblsDataMsg<C>>,
{
    type Error = Data<L>;

    fn try_from(value: Data<L>) -> Result<Self, Self::Error> {
        let LightClientSpecificData(value) = LightClientSpecificData::try_from(value)?;

        match value {
            CometblsDataMsg::BeaconGenesis(ok) => Ok(ok),
            _ => Err(LightClientSpecificData(value).into()),
        }
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum CometblsAggregateMsg<L: LightClient<HostChain = Evm<C>>, C: ChainSpec> {
    #[display(fmt = "CreateUpdate")]
    CreateUpdate(CreateUpdateData<L, C>),
    #[display(fmt = "MakeCreateUpdates")]
    MakeCreateUpdates(MakeCreateUpdatesData<L, C>),
    #[display(fmt = "MakeCreateUpdatesFromLightClientUpdates")]
    MakeCreateUpdatesFromLightClientUpdates(MakeCreateUpdatesFromLightClientUpdatesData<L, C>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FinalityUpdate<C: ChainSpec>(pub LightClientFinalityUpdate<C>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientUpdates<C: ChainSpec>(pub Vec<LightClientUpdate<C>>);

impl<C, L> DoAggregate<L> for CometblsAggregateMsg<L, C>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Aggregate = Self, Fetch = CometblsFetchMsg<L, C>>,

    Identified<L, AccountUpdateData<C>>: IsAggregateData,
    Identified<L, BootstrapData<C>>: IsAggregateData,
    Identified<L, BeaconGenesisData<C>>: IsAggregateData,
    Identified<L, FinalityUpdate<C>>: IsAggregateData,
    Identified<L, LightClientUpdates<C>>: IsAggregateData,
    Identified<L, LightClientUpdate<C>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L::Counterparty>)>,

    AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    fn do_aggregate(
        Identified { chain_id, data }: Identified<L, Self>,
        aggregated_data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> Vec<RelayerMsg> {
        [match data {
            CometblsAggregateMsg::CreateUpdate(msg) => do_aggregate(
                Identified {
                    chain_id,
                    data: msg,
                },
                aggregated_data,
            ),
            CometblsAggregateMsg::MakeCreateUpdates(msg) => do_aggregate(
                Identified {
                    chain_id,
                    data: msg,
                },
                aggregated_data,
            ),
            CometblsAggregateMsg::MakeCreateUpdatesFromLightClientUpdates(msg) => do_aggregate(
                Identified {
                    chain_id,
                    data: msg,
                },
                aggregated_data,
            ),
        }]
        .into()
    }
}

fn make_create_update<C, L>(
    req: FetchUpdateHeaders<L>,
    chain_id: <<Evm<C> as Chain>::SelfClientState as ClientState>::ChainId,
    currently_trusted_slot: u64,
    light_client_update: LightClientUpdate<C>,
    is_next: bool,
) -> RelayerMsg
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<Identified<L, Aggregate<L>>>,
{
    // When we fetch the update at this height, the `next_sync_committee` will
    // be the current sync committee of the period that we want to update to.
    let previous_period = u64::max(
        1,
        light_client_update.attested_header.beacon.slot
            / (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64),
    ) - 1;
    RelayerMsg::Aggregate {
        queue: [
            fetch::<L>(
                chain_id,
                LightClientSpecificFetch(CometblsFetchMsg::FetchLightClientUpdate(
                    FetchLightClientUpdate {
                        period: previous_period,
                        __marker: PhantomData,
                    },
                )),
            ),
            fetch::<L>(
                chain_id,
                LightClientSpecificFetch(CometblsFetchMsg::FetchAccountUpdate(
                    FetchAccountUpdate {
                        slot: light_client_update.attested_header.beacon.slot,
                        __marker: PhantomData,
                    },
                )),
            ),
            fetch::<L>(
                chain_id,
                LightClientSpecificFetch(CometblsFetchMsg::FetchBeaconGenesis(
                    FetchBeaconGenesis {
                        __marker: PhantomData,
                    },
                )),
            ),
        ]
        .into(),
        data: [].into(),
        receiver: aggregate(
            chain_id,
            LightClientSpecificAggregate(CometblsAggregateMsg::CreateUpdate(CreateUpdateData {
                req,
                currently_trusted_slot,
                light_client_update,
                is_next,
            })),
        ),
    }
}

fn sync_committee_period<H: Into<u64>, C: ChainSpec>(height: H) -> u64 {
    height.into().div(C::PERIOD::U64)
}

async fn do_msg<C, L>(evm: &Evm<C>, msg: Msg<L>) -> Result<(), TxSubmitError>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Config = CometblsConfig>,
    ClientStateOf<<L::Counterparty as LightClientBase>::HostChain>: IntoEthAbi,
    ConsensusStateOf<<L::Counterparty as LightClientBase>::HostChain>: IntoEthAbi,
    HeaderOf<<L::Counterparty as LightClientBase>::HostChain>: IntoEthAbi,
    // not sure why these bounds are required
    <<L::BaseCounterparty as LightClientBase>::HostChain as Chain>::Header: EthAbi,
    <<L::BaseCounterparty as LightClientBase>::HostChain as Chain>::SelfClientState: EthAbi,
    <<L::BaseCounterparty as LightClientBase>::HostChain as Chain>::SelfConsensusState: EthAbi,
{
    evm.ibc_handlers
        .with(|ibc_handler| async move {
            let msg: ethers::contract::FunctionCall<_, _, ()> = match msg {
                Msg::ConnectionOpenInit(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenInitCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ConnectionOpenTry(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenTryCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ConnectionOpenAck(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenAckCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ConnectionOpenConfirm(data) => mk_function_call(
                    ibc_handler,
                    ConnectionOpenConfirmCall { msg: data.0.into() },
                ),
                Msg::ChannelOpenInit(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenInitCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ChannelOpenTry(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenTryCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ChannelOpenAck(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenAckCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::ChannelOpenConfirm(data) => mk_function_call(
                    ibc_handler,
                    ChannelOpenConfirmCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::RecvPacket(data) => mk_function_call(
                    ibc_handler,
                    RecvPacketCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::AckPacket(data) => mk_function_call(
                    ibc_handler,
                    AcknowledgePacketCall {
                        msg: data.msg.into(),
                    },
                ),
                Msg::CreateClient(data) => {
                    let register_client_result = ibc_handler.register_client(
                        data.config.client_type.clone(),
                        data.config.cometbls_client_address.clone().into(),
                    );

                    // TODO(benluelo): Better way to check if client type has already been registered?
                    match register_client_result.send().await {
                        Ok(ok) => {
                            ok.await.unwrap().unwrap();
                        }
                        Err(why) => tracing::info!(
                            "error registering client type, it is likely already registered: {}",
                            why.decode_revert::<String>().unwrap()
                        ),
                    }

                    mk_function_call(
                        ibc_handler,
                        CreateClientCall {
                            msg: contracts::shared_types::MsgCreateClient {
                                client_type: data.config.client_type,
                                client_state_bytes: data
                                    .msg
                                    .client_state
                                    .into_eth_abi_bytes()
                                    .into(),
                                consensus_state_bytes: data
                                    .msg
                                    .consensus_state
                                    .into_eth_abi_bytes()
                                    .into(),
                            },
                        },
                    )
                }
                Msg::UpdateClient(data) => mk_function_call(
                    ibc_handler,
                    UpdateClientCall {
                        msg: ibc_handler::MsgUpdateClient {
                            client_id: data.msg.client_id.to_string(),
                            client_message: data
                                .msg
                                .client_message
                                .clone()
                                .into_eth_abi_bytes()
                                .into(),
                        },
                    },
                ),
            };
            let result = msg.send().await;
            match result {
                Ok(ok) => {
                    let tx_rcp = ok.await?.ok_or(TxSubmitError::NoTxReceipt)?;
                    tracing::info!(?tx_rcp, "evm transaction submitted");
                    Ok(())
                }
                Err(ContractError::Revert(revert)) => {
                    tracing::error!(?revert, "evm transaction failed");
                    Ok(())
                }
                _ => {
                    panic!("evm transaction non-recoverable failure");
                }
            }
        })
        .await
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Contract(#[from] ContractError<CometblsMiddleware>),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("no tx receipt from tx")]
    NoTxReceipt,
}

impl MaybeRecoverableError for TxSubmitError {
    fn is_recoverable(&self) -> bool {
        // TODO: Figure out if any failures are unrecoverable
        true
    }
}

fn mk_function_call<Call: EthCall>(
    ibc_handler: IBCHandler<CometblsMiddleware>,
    data: Call,
) -> ethers::contract::FunctionCall<Arc<CometblsMiddleware>, CometblsMiddleware, ()> {
    ibc_handler
        .method_hash(<Call as EthCall>::selector(), data)
        .expect("method selector is generated; qed;")
}

async fn do_fetch<C, L>(evm: &Evm<C>, msg: CometblsFetchMsg<L, C>) -> Vec<RelayerMsg>
where
    C: ChainSpec,
    L: LightClient<HostChain = Evm<C>, Fetch = CometblsFetchMsg<L, C>, Data = CometblsDataMsg<C>>,
    LightClientSpecificData<L>: From<CometblsDataMsg<C>>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
{
    let msg = match msg {
        CometblsFetchMsg::FetchFinalityUpdate(PhantomData {}) => CometblsDataMsg::FinalityUpdate(
            FinalityUpdate(evm.beacon_api_client.finality_update().await.unwrap().data),
        ),
        CometblsFetchMsg::FetchLightClientUpdates(FetchLightClientUpdates {
            trusted_period,
            target_period,
            __marker: PhantomData,
        }) => CometblsDataMsg::LightClientUpdates(LightClientUpdates(
            evm.beacon_api_client
                .light_client_updates(trusted_period + 1, target_period - trusted_period)
                .await
                .unwrap()
                .0
                .into_iter()
                .map(|x| x.data)
                .collect(),
        )),
        CometblsFetchMsg::FetchLightClientUpdate(FetchLightClientUpdate {
            period,
            __marker: PhantomData,
        }) => CometblsDataMsg::LightClientUpdate(
            evm.beacon_api_client
                .light_client_updates(period, 1)
                .await
                .unwrap()
                .0
                .into_iter()
                .map(|x| x.data)
                .collect::<Vec<LightClientUpdate<_>>>()
                .pop()
                .unwrap(),
        ),
        CometblsFetchMsg::FetchBootstrap(FetchBootstrap { slot, __marker: _ }) => {
            // NOTE(benluelo): While this is technically two actions, I consider it to be one
            // action - if the beacon chain doesn't have the header, it won't have the bootstrap
            // either. It would be nice if the beacon chain exposed "fetch bootstrap by slot"
            // functionality; I'm surprised it doesn't.

            let mut amount_of_slots_back: u64 = 0;

            let floored_slot = slot
                / (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64)
                * (C::SLOTS_PER_EPOCH::U64 * C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64);

            tracing::info!("fetching bootstrap at {}", floored_slot);

            let bootstrap = loop {
                let header_response = evm
                    .beacon_api_client
                    .header(beacon_api::client::BlockId::Slot(
                        floored_slot - amount_of_slots_back,
                    ))
                    .await;

                let header = match header_response {
                    Ok(header) => header,
                    Err(beacon_api::errors::Error::NotFound(NotFoundError {
                        status_code: _,
                        error: _,
                        message,
                    })) if message.starts_with("No block found for id") => {
                        amount_of_slots_back += 1;
                        continue;
                    }

                    Err(err) => panic!("{err}"),
                };

                let bootstrap_response = evm
                    .beacon_api_client
                    .bootstrap(header.data.root.clone())
                    .await;

                match bootstrap_response {
                    Ok(ok) => break ok.data,
                    Err(err) => match err {
                        beacon_api::errors::Error::Internal(InternalServerError {
                            status_code: _,
                            error: _,
                            message,
                        }) if message.starts_with("syncCommitteeWitness not available") => {
                            amount_of_slots_back += 1;
                        }
                        _ => panic!("{err}"),
                    },
                };
            };

            // bootstrap contains the current sync committee for the given height
            CometblsDataMsg::Bootstrap(BootstrapData { slot, bootstrap })
        }
        CometblsFetchMsg::FetchAccountUpdate(FetchAccountUpdate { slot, __marker: _ }) => {
            let execution_height = evm
                .execution_height(Height {
                    revision_number: EVM_REVISION_NUMBER,
                    revision_height: slot,
                })
                .await;

            CometblsDataMsg::AccountUpdate(AccountUpdateData {
                slot,
                ibc_handler_address: evm.readonly_ibc_handler.address().0.into(),
                update: evm
                    .provider
                    .get_proof(
                        evm.readonly_ibc_handler.address(),
                        vec![],
                        // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                        Some(execution_height.into()),
                    )
                    .await
                    .unwrap(),
                __marker: PhantomData,
            })
        }
        CometblsFetchMsg::FetchBeaconGenesis(_) => {
            CometblsDataMsg::BeaconGenesis(BeaconGenesisData {
                genesis: evm.beacon_api_client.genesis().await.unwrap().data,
                __marker: PhantomData,
            })
        }
        CometblsFetchMsg::FetchGetProof(get_proof) => {
            let execution_height = evm.execution_height(get_proof.height).await;

            let path = get_proof.path.to_string();

            let location = keccak256(
                keccak256(path.as_bytes())
                    .into_iter()
                    .chain(U256::from(0).encode())
                    .collect::<Vec<_>>(),
            );

            let proof = evm
                .provider
                .get_proof(
                    evm.readonly_ibc_handler.address(),
                    vec![location.into()],
                    Some(execution_height.into()),
                )
                .await
                .unwrap();

            tracing::info!(?proof);

            let proof = match <[_; 1]>::try_from(proof.storage_proof) {
                Ok([proof]) => proof,
                Err(invalid) => {
                    panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
                }
            };

            let proof = ethereum_v1::StorageProof {
                proofs: [ethereum_v1::Proof {
                    key: proof.key.to_fixed_bytes().to_vec(),
                    // REVIEW(benluelo): Make sure this encoding works
                    value: proof.value.encode(),
                    proof: proof
                        .proof
                        .into_iter()
                        .map(|bytes| bytes.to_vec())
                        .collect(),
                }]
                .to_vec(),
            }
            .encode_to_vec();

            return [match get_proof.path {
                unionlabs::proof::Path::ClientStatePath(_) => data::<L>(
                    evm.chain_id,
                    ClientStateProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::ClientConsensusStatePath(_) => data::<L>(
                    evm.chain_id,
                    ClientConsensusStateProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::ConnectionPath(_) => data::<L>(
                    evm.chain_id,
                    ConnectionProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::ChannelEndPath(_) => data::<L>(
                    evm.chain_id,
                    ChannelEndProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::CommitmentPath(_) => data::<L>(
                    evm.chain_id,
                    CommitmentProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
                unionlabs::proof::Path::AcknowledgementPath(_) => data::<L>(
                    evm.chain_id,
                    AcknowledgementProof {
                        proof,
                        height: get_proof.height,
                    },
                ),
            }]
            .into();
        }
    };

    [data::<L>(evm.chain_id, LightClientSpecificData::from(msg))].into()
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct GetProof<C: ChainSpec, L: LightClient<HostChain = Evm<C>>> {
    path: unionlabs::proof::Path<
        <L::HostChain as Chain>::ClientId,
        HeightOf<ChainOf<L::Counterparty>>,
    >,
    height: <Evm<C> as Chain>::Height,
}

impl<L, C> UseAggregate<L> for Identified<L, CreateUpdateData<L, C>>
where
    Identified<L, AccountUpdateData<C>>: IsAggregateData,
    Identified<L, LightClientUpdate<C>>: IsAggregateData,
    Identified<L, BeaconGenesisData<C>>: IsAggregateData,

    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L::Counterparty>)>,

    L: LightClient<HostChain = Evm<C>>,
    C: ChainSpec,
{
    type AggregatedData = HList![
        Identified<L, LightClientUpdate<C>>,
        Identified<L, AccountUpdateData<C>>,
        Identified<L, BeaconGenesisData<C>>
    ];

    fn aggregate(
        Identified {
            chain_id,
            data:
                CreateUpdateData {
                    req,
                    currently_trusted_slot,
                    light_client_update,
                    is_next,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: bootstrap_chain_id,
                data: LightClientUpdate {
                    next_sync_committee,
                    ..
                }
            },
            Identified {
                chain_id: account_update_chain_id,
                data: AccountUpdateData {
                    ibc_handler_address,
                    update: account_update,
                    ..
                }
            },
            Identified {
                chain_id: beacon_api_chain_id,
                data: BeaconGenesisData {
                    genesis,
                    __marker: _,
                }
            }
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(bootstrap_chain_id, account_update_chain_id);
        assert_eq!(chain_id, account_update_chain_id);
        assert_eq!(chain_id, beacon_api_chain_id);

        let header = wasm::client_message::ClientMessage {
            data: ethereum::header::Header {
                consensus_update: light_client_update,
                trusted_sync_committee: TrustedSyncCommittee {
                    trusted_height: Height {
                        revision_number: EVM_REVISION_NUMBER,
                        revision_height: currently_trusted_slot,
                    },
                    sync_committee: if is_next {
                        ActiveSyncCommittee::Next(next_sync_committee.unwrap())
                    } else {
                        ActiveSyncCommittee::Current(next_sync_committee.unwrap())
                    },
                },
                account_update: AccountUpdate {
                    account_proof: AccountProof {
                        contract_address: ibc_handler_address,
                        storage_root: account_update.storage_hash.into(),
                        proof: account_update
                            .account_proof
                            .into_iter()
                            .map(|x| x.to_vec())
                            .collect(),
                    },
                },
            },
        };

        seq([
            wait::<L::Counterparty>(
                req.counterparty_chain_id.clone(),
                WaitForTimestamp {
                    timestamp: (genesis.genesis_time
                        + (header.data.consensus_update.signature_slot * C::SECONDS_PER_SLOT::U64))
                        .try_into()
                        .unwrap(),
                    __marker: PhantomData,
                },
            ),
            msg::<L::Counterparty>(
                req.counterparty_chain_id,
                MsgUpdateClientData {
                    msg: MsgUpdateClient {
                        client_id: req.counterparty_client_id,
                        client_message: header,
                    },
                    update_from: Height {
                        revision_number: 0,
                        revision_height: currently_trusted_slot,
                    },
                },
            ),
        ])
    }
}

impl<L, C> UseAggregate<L> for Identified<L, MakeCreateUpdatesData<L, C>>
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,
    Identified<L, FinalityUpdate<C>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<Identified<L, Aggregate<L>>>,
{
    type AggregatedData = HList![
        Identified<L, FinalityUpdate<C>>,
    ];

    fn aggregate(
        Identified {
            chain_id,
            data: MakeCreateUpdatesData { req },
        }: Self,
        hlist_pat![Identified {
            chain_id: bootstrap_chain_id,
            data: FinalityUpdate(finality_update)
        },]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(chain_id, bootstrap_chain_id);

        let target_period =
            sync_committee_period::<_, C>(finality_update.attested_header.beacon.slot);

        let trusted_period = sync_committee_period::<_, C>(req.update_from.revision_height);

        assert!(
        trusted_period <= target_period,
        "trusted period {trusted_period} is behind target period {target_period}, something is wrong!",
    );

        // Eth chain is more than 1 signature period ahead of us. We need to do sync committee
        // updates until we reach the `target_period - 1`.
        RelayerMsg::Aggregate {
            queue: [fetch::<L>(
                chain_id,
                LightClientSpecificFetch(CometblsFetchMsg::FetchLightClientUpdates(
                    FetchLightClientUpdates {
                        trusted_period,
                        target_period,
                        __marker: PhantomData,
                    },
                )),
            )]
            .into(),
            data: [].into(),
            receiver: aggregate(
                chain_id,
                LightClientSpecificAggregate(
                    CometblsAggregateMsg::MakeCreateUpdatesFromLightClientUpdates(
                        MakeCreateUpdatesFromLightClientUpdatesData {
                            req: req.clone(),
                            trusted_height: req.update_from,
                            finality_update,
                        },
                    ),
                ),
            ),
        }
    }
}

impl<L, C> UseAggregate<L> for Identified<L, MakeCreateUpdatesFromLightClientUpdatesData<L, C>>
where
    C: ChainSpec,
    L: LightClient<
        HostChain = Evm<C>,
        Fetch = CometblsFetchMsg<L, C>,
        Aggregate = CometblsAggregateMsg<L, C>,
    >,

    Identified<L, LightClientUpdates<C>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<Identified<L, Aggregate<L>>>,
{
    type AggregatedData = HList![
        Identified<L, LightClientUpdates<C>>,
    ];

    fn aggregate(
        Identified {
            chain_id,
            data:
                MakeCreateUpdatesFromLightClientUpdatesData {
                    req,
                    trusted_height,
                    finality_update,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: light_client_updates_chain_id,
            data: LightClientUpdates(light_client_updates)
        },]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(chain_id, light_client_updates_chain_id);

        let target_period = sync_committee_period::<_, C>(finality_update.signature_slot);

        let trusted_period = sync_committee_period::<_, C>(req.update_from.revision_height);

        let (updates, last_update_block_number) = light_client_updates.into_iter().fold(
            (VecDeque::new(), trusted_height.revision_height),
            |(mut vec, mut trusted_slot), update| {
                let old_trusted_slot = trusted_slot;

                trusted_slot = update.attested_header.beacon.slot;

                vec.push_back(make_create_update::<C, L>(
                    req.clone(),
                    chain_id,
                    old_trusted_slot,
                    update,
                    true,
                ));

                (vec, trusted_slot)
            },
        );

        let lc_updates = if trusted_period < target_period {
            updates
        } else {
            [].into()
        };

        let does_not_have_finality_update =
            last_update_block_number >= req.update_to.revision_height;

        tracing::error!(last_update_block_number, req.update_to.revision_height);

        let finality_update_msg = if does_not_have_finality_update {
            // do nothing
            None
        } else {
            // do finality update
            Some(make_create_update(
                req.clone(),
                chain_id,
                last_update_block_number,
                LightClientUpdate {
                    attested_header: finality_update.attested_header,
                    next_sync_committee: None,
                    next_sync_committee_branch: None,
                    finalized_header: finality_update.finalized_header,
                    finality_branch: finality_update.finality_branch,
                    sync_aggregate: finality_update.sync_aggregate,
                    signature_slot: finality_update.signature_slot,
                },
                false,
            ))
        };

        seq(lc_updates.into_iter().chain(finality_update_msg))
    }
}

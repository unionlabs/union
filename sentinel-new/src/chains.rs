use chain_utils::cosmos_sdk::CosmosSdkChainExt;
use prost::{Message, Name};
use protos::{google::protobuf::Any, ibc::applications::transfer::v1::MsgTransfer};
use serde::{Deserialize, Serialize};
use ucs01_relay::msg::{ExecuteMsg, TransferMsg};
use unionlabs::{
    cosmos::base::coin::Coin,
    ethereum::config::{ChainSpec, Mainnet, Minimal},
    hash::H160,
    id::ChannelId,
};

use crate::config::{EthereumConfig, OsmosisConfig, TransferModule, UnionConfig};

pub trait IbcTransfer: Send + Sync {
    fn send_ibc_transfer(&self, protocol: Protocol, channel: ChannelId, denom: String, amount: u64);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Protocol {
    Ics20 {
        receiver: String,
        module: TransferModule,
    },
    Ucs01 {
        receiver: Vec<u8>,
        contract: String,
    },
}

#[derive(Debug, Clone)]
pub enum Chain {
    EthereumMinimal(Ethereum<Minimal>),
    EthereumMainnet(Ethereum<Mainnet>),
    Osmosis(Osmosis),
    Union(Union),
}

impl IbcTransfer for Chain {
    fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        denom: String,
        amount: u64,
    ) {
        match self {
            Chain::EthereumMinimal(ethereum) => {
                ethereum.send_ibc_transfer(protocol, channel, denom, amount)
            }
            Chain::EthereumMainnet(_) => todo!(),
            Chain::Osmosis(_) => todo!(),
            Chain::Union(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ethereum<C: ChainSpec> {
    pub chain: chain_utils::ethereum::Ethereum<C>,
}

impl<C: ChainSpec> IbcTransfer for Ethereum<C> {
    fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        denom: String,
        amount: u64,
    ) {
    }
}

impl<C: ChainSpec> Ethereum<C> {
    pub async fn new(config: EthereumConfig) -> Self {
        let ethereum = chain_utils::ethereum::Ethereum::new(config.chain_config)
            .await
            .unwrap();

        Ethereum { chain: ethereum }
    }
}

#[derive(Debug, Clone)]
pub struct Osmosis {
    pub chain: chain_utils::cosmos::Cosmos,
}

impl IbcTransfer for Osmosis {
    fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        denom: String,
        amount: u64,
    ) {
        todo!()
    }
}

impl Osmosis {
    pub async fn new(config: OsmosisConfig) -> Self {
        let osmosis = chain_utils::cosmos::Cosmos::new(config.chain_config)
            .await
            .unwrap();

        Osmosis { chain: osmosis }
    }
}

#[derive(Debug, Clone)]
pub struct Union {
    pub chain: chain_utils::union::Union,
}

impl IbcTransfer for Union {
    fn send_ibc_transfer(
        &self,
        protocol: Protocol,
        channel: ChannelId,
        denom: String,
        amount: u64,
    ) {
        todo!()
    }
}

impl Union {
    pub async fn new(config: UnionConfig) -> Self {
        let union = chain_utils::union::Union::new(config.chain_config)
            .await
            .unwrap();

        Union { chain: union }
    }
}

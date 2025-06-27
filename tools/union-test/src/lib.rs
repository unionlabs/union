use std::{str::FromStr, time::Duration, sync::Arc};

use alloy::sol_types::SolValue;
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{Config as CosmosConfig, Module as CosmosModule, FeemarketConfig, GasFillerConfig};
use hex_literal::hex;
use ibc_union_msg::msg::{ExecuteMsg, MsgCreateClient};
use ibc_union_spec::{ChannelId, Timestamp, ClientId, datagram::Datagram};
use protos::{cosmos::base::v1beta1::Coin, ibc::core::channel};
use ucs03_zkgm::com::{
    FungibleAssetOrder, FungibleAssetOrderV2, Instruction, INSTR_VERSION_1, OP_FUNGIBLE_ASSET_ORDER,
};
use unionlabs::{
    bech32::Bech32,
    encoding::{Bincode, EncodeAs, EthAbi, Encode, Json},
    primitives::{Bytes, FixedBytes},
};
use cometbft_rpc::rpc_types::{Order, TxResponse};
use cosmos_client::BroadcastTxCommitError;

use unionlabs::{primitives::{H160, H256}, ErrorReporter};
use alloy::{network::AnyNetwork, contract::RawCallBuilder, providers::DynProvider};
use voyager_sdk::{anyhow::{self, Ok}, primitives::ChainId};
use std::process::Command;
use axum::async_trait;
use jsonrpsee::core::RpcResult;

pub mod evm;
pub mod cosmos;
pub mod channel_provider;
pub mod voyager;
pub mod helpers;

use crate::cosmos::IbcEvent;
use crate::channel_provider::{ChannelPool, ChannelConfirm, ChannelPair};

#[async_trait]
pub trait ChainEndpoint: Send + Sync {

    fn chain_id(&self) -> &ChainId;

    fn send_create_client(
        &self,
        tracking: &ChainId,
        ibc_interface: &str,
        client_type: &str,
    ) -> anyhow::Result<()>;

    async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm>;

    fn send_open_connection(
        &self,
        counterparty: &ChainId,
        client_id: u32,
        counterparty_client_id: u32,
    ) -> anyhow::Result<()>;

    async fn wait_for_open_connection(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm>;

    fn send_open_channel(
        &self,
        port_id: Bytes,
        counterparty_port: Bytes,
        connection_id: u32,
        version: String,
    ) -> anyhow::Result<()>;

    async fn wait_for_open_channel(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ChannelOpenConfirm>;

    // TODO: How to handle this for EVM chains?
    async fn send_ibc_packet(
        &self,
        contract: Bech32<H256>,
        funded_msgs: Vec<(Box<impl Encode<Json> + Clone + Send>, Vec<Coin>)>,
    ) -> anyhow::Result<H256> ;

    async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv>;
}

pub trait IbcEventHash {
    type Hash;
}

#[async_trait]
impl ChainEndpoint for evm::Module {

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    fn send_create_client(
        &self,
        tracking: &ChainId,
        ibc_interface: &str,
        client_type: &str,
    ) -> anyhow::Result<()> {
        voyager::create_client(
            self.chain_id.clone(),
            tracking.clone(),
            ibc_interface.to_string(),
            client_type.to_string(),
        )?;
        Ok(())
    }

    async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        self.wait_for_create_client(timeout).await
    }

    fn send_open_connection(
        &self,
        counterparty: &ChainId,
        client_id: u32,
        counterparty_client_id: u32,
    ) -> anyhow::Result<()> {
        voyager::connection_open(
            self.chain_id.clone(),
            client_id,
            counterparty_client_id,
        )?;
        Ok(())
    }

    async fn wait_for_open_connection(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        self.wait_for_connection_open_confirm(timeout).await
    }

    fn send_open_channel(
        &self,
        port_id: Bytes,
        counterparty_port: Bytes,
        connection_id: u32,
        version: String,
    ) -> anyhow::Result<()> {
        voyager::channel_open(
            self.chain_id.clone(),
            port_id,
            counterparty_port,
            connection_id,
            version,
        )?;
        Ok(())
    }

    async fn wait_for_open_channel(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ChannelOpenConfirm> {
        self.wait_for_channel_open_confirm(timeout).await
    }

    // TODO: How to handle this for EVM chains?
    async fn send_ibc_packet(
        &self,
        contract: Bech32<H256>,
        funded_msgs: Vec<(Box<impl Encode<Json> + Clone + Send>, Vec<Coin>)>,
    ) -> anyhow::Result<H256> {
        unimplemented!("Sending IBC packets is not implemented for EVM chains");
    }

    async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        self.wait_for_packet_recv(packet_hash, timeout).await
    }
}

impl IbcEventHash for ibc_solidity::Ibc::PacketRecv {
    type Hash = H256;
}

#[async_trait]
impl ChainEndpoint for cosmos::Module {

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    fn send_create_client(
        &self,
        tracking: &ChainId,
        ibc_interface: &str,
        client_type: &str,
    ) -> anyhow::Result<()> {
        voyager::create_client(
            self.chain_id.clone(),
            tracking.clone(),
            ibc_interface.to_string(),
            client_type.to_string(),
        )?;
        Ok(())
    }

    async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        self.wait_for_create_client_id(timeout).await
    }

    fn send_open_connection(
        &self,
        counterparty: &ChainId,
        client_id: u32,
        counterparty_client_id: u32,
    ) -> anyhow::Result<()> {
        voyager::connection_open(
            self.chain_id.clone(),
            client_id,
            counterparty_client_id,
        )?;
        Ok(())
    }

    async fn wait_for_open_connection(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        self.wait_for_connection_open_confirm(timeout).await
    }

    fn send_open_channel(
        &self,
        port_id: Bytes,
        counterparty_port: Bytes,
        connection_id: u32,
        version: String,
    ) -> anyhow::Result<()> {
        voyager::channel_open(
            self.chain_id.clone(),
            port_id,
            counterparty_port,
            connection_id,
            version,
        )?;
        Ok(())
    }

    async fn wait_for_open_channel(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ChannelOpenConfirm> {
        self.wait_for_channel_open_confirm(timeout).await
    }

    async fn send_ibc_packet(
        &self,
        contract: Bech32<H256>,
        funded_msgs: Vec<(Box<impl Encode<Json> + Clone + Send>, Vec<Coin>)>,
    ) -> anyhow::Result<H256>  {
        self.send_ibc_transaction(contract, funded_msgs).await
    }

    async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        self.wait_for_packet_recv(packet_hash, timeout).await
    }
}

pub struct TestContext<S: ChainEndpoint, D: ChainEndpoint> {
    pub src: S,
    pub dst: D,
    pub channel_pool: Arc<ChannelPool>,
}

impl<S, D> TestContext<S, D>
where
    S: ChainEndpoint + 'static,
    D: ChainEndpoint + 'static,
{
    pub async fn new(src: S, dst: D) -> anyhow::Result<Self> {
        voyager::init_fetch(src.chain_id().clone())?;
        voyager::init_fetch(dst.chain_id().clone())?;
        let channel_pool = ChannelPool::new();
        Ok(Self { src, dst, channel_pool })
    }

    pub async fn create_clients(
        &self,
        duration: Duration,
        src_ibc_interface: &str,
        src_client_type: &str,
        dst_ibc_interface: &str,
        dst_client_type: &str,
    ) -> anyhow::Result<(helpers::CreateClientConfirm, helpers::CreateClientConfirm)> {
        self.src.send_create_client(self.dst.chain_id(), src_ibc_interface, src_client_type)?;
        self.dst.send_create_client(self.src.chain_id(), dst_ibc_interface, dst_client_type)?;
        let dst_confirm = self.dst.wait_for_create_client(duration).await?;
        let src_confirm = self.src.wait_for_create_client(duration).await?;

        
        Ok((src_confirm, dst_confirm))
    }

    pub async fn open_connection(
        &self,
        send_from_source: bool,
        src_client_id: u32,
        dst_client_id: u32,
        duration: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        if send_from_source {
            self.src.send_open_connection(self.dst.chain_id(), src_client_id, dst_client_id)?;
            let conn = self.dst.wait_for_open_connection(duration).await?;
            return Ok(conn);
        } 
        self.dst.send_open_connection(self.src.chain_id(), dst_client_id, src_client_id)?;
        let conn = self.src.wait_for_open_connection(duration).await?;
        Ok(conn)
    }

    pub async fn open_channels(
        &self,
        send_from_source: bool,
        src_port: Bytes,
        dst_port: Bytes,
        connection_id: u32,
        version: String,
        count: usize,
        duration: Duration,
    ) -> anyhow::Result<usize> {
        if send_from_source {
            let opened = self
                .channel_pool
                .open_channels(
                    voyager::channel_open,
                    |t: Duration| {
                        async move {
                            let ev = self.dst.wait_for_open_channel(t).await?;
                            Ok(ChannelConfirm {
                                channel_id: ev.channel_id,
                                counterparty_channel_id: ev.counterparty_channel_id,
                            })
                        }
                    },
                    self.src.chain_id().clone(),
                    src_port,
                    self.dst.chain_id().clone(),
                    dst_port,
                    connection_id,
                    version,
                    count,
                    duration,
                )
                .await?;
            return Ok(opened);
        }

        let opened = self
            .channel_pool
            .open_channels(
                voyager::channel_open,
                |t: Duration| {
                    async move {
                        let ev = self.src.wait_for_open_channel(t).await?;
                        Ok(ChannelConfirm {
                            channel_id: ev.channel_id,
                            counterparty_channel_id: ev.counterparty_channel_id,
                        })
                    }
                },
                self.dst.chain_id().clone(),
                dst_port,
                self.src.chain_id().clone(),
                src_port,
                connection_id,
                version,
                count,
                duration,
            )
            .await?;
        Ok(opened)
    }

    pub async fn get_channel(&self) -> Option<ChannelPair> {
        self.channel_pool.get_channel(self.src.chain_id(), self.dst.chain_id()).await
    }

    pub async fn release_channel(&self, pair: ChannelPair) {
        self.channel_pool.release_channel(self.src.chain_id(), self.dst.chain_id(), pair).await;
    }

    pub async fn get_available_channel_count(&self) -> usize {
        self.channel_pool.get_available_channel_count(self.src.chain_id(), self.dst.chain_id()).await
    }


    pub async fn send_and_recv(
        &self,
        send_from_source: bool,
        contract: Bech32<H256>,
        funded_msgs: Vec<(Box<impl Encode<Json> + Clone + Send>, Vec<Coin>)>,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        if send_from_source {
            let packet_hash = self.src.send_ibc_packet(contract, funded_msgs).await?;
            let recv = self.dst.wait_for_packet_recv(packet_hash, timeout).await?;
            return Ok(recv);
        }
        unimplemented!("Sending IBC packets is not implemented for EVM chains");
    }
}


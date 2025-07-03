use std::{sync::Arc, time::Duration};

use alloy::{contract::RawCallBuilder, network::AnyNetwork, providers::DynProvider};
use axum::async_trait;
use protos::cosmos::base::v1beta1::Coin;
use unionlabs::{
    bech32::Bech32,
    primitives::{Bytes, H160, H256},
};
use voyager_sdk::{
    anyhow::{self, Ok},
    primitives::ChainId,
};

pub mod channel_provider;
pub mod cosmos;
pub mod evm;
pub mod helpers;
pub mod voyager;

use crate::channel_provider::{ChannelConfirm, ChannelPair, ChannelPool};

#[async_trait]
pub trait ChainEndpoint: Send + Sync {
    type Msg;
    type Contract;

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
    async fn send_ibc_transaction(
        &self,
        contract: Self::Contract,
        msg: Self::Msg,
    ) -> anyhow::Result<H256>;

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
impl<'a> ChainEndpoint for evm::Module<'a> {
    type Msg = RawCallBuilder<&'a DynProvider<AnyNetwork>, AnyNetwork>;
    type Contract = H160;

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
        client_id: u32,
        counterparty_client_id: u32,
    ) -> anyhow::Result<()> {
        voyager::connection_open(self.chain_id.clone(), client_id, counterparty_client_id)?;
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
    async fn send_ibc_transaction(
        &self,
        contract: Self::Contract,
        msg: Self::Msg,
    ) -> anyhow::Result<H256> {
        self.send_ibc_transaction(contract, msg)
            .await
            .map_err(Into::into)
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
    type Msg = (Vec<u8>, Vec<Coin>);
    type Contract = Bech32<H256>;

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
        client_id: u32,
        counterparty_client_id: u32,
    ) -> anyhow::Result<()> {
        voyager::connection_open(self.chain_id.clone(), client_id, counterparty_client_id)?;
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

    async fn send_ibc_transaction(
        &self,
        contract: Bech32<H256>,
        msg: Self::Msg,
    ) -> anyhow::Result<H256> {
        self.send_ibc_transaction(contract, msg).await
    }

    async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        self.wait_for_packet_recv(packet_hash, timeout).await
    }
}

pub enum ContractAddr {
    Cosmos(Bech32<H256>),
    Evm(H160),
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
        Ok(Self {
            src,
            dst,
            channel_pool,
        })
    }

    pub async fn create_clients(
        &self,
        duration: Duration,
        src_ibc_interface: &str,
        src_client_type: &str,
        dst_ibc_interface: &str,
        dst_client_type: &str,
    ) -> anyhow::Result<(helpers::CreateClientConfirm, helpers::CreateClientConfirm)> {
        self.src
            .send_create_client(self.dst.chain_id(), src_ibc_interface, src_client_type)?;
        let src_confirm = self.src.wait_for_create_client(duration).await?;
        self.dst
            .send_create_client(self.src.chain_id(), dst_ibc_interface, dst_client_type)?;
        let dst_confirm = self.dst.wait_for_create_client(duration).await?;

        Ok((src_confirm, dst_confirm))
    }

    pub async fn open_connection<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        src: &Src,
        src_client_id: u32,
        dst: &Dst,
        dst_client_id: u32,
        duration: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        src.send_open_connection(src_client_id, dst_client_id)?;
        let conn = dst.wait_for_open_connection(duration).await?;
        return Ok(conn);
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
                    |t: Duration| async move {
                        let ev = self.dst.wait_for_open_channel(t).await?;
                        Ok(ChannelConfirm {
                            channel_id: ev.channel_id,
                            counterparty_channel_id: ev.counterparty_channel_id,
                        })
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
                |t: Duration| async move {
                    let ev = self.src.wait_for_open_channel(t).await?;
                    Ok(ChannelConfirm {
                        channel_id: ev.channel_id,
                        counterparty_channel_id: ev.counterparty_channel_id,
                    })
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
        self.channel_pool
            .get_channel(self.src.chain_id(), self.dst.chain_id())
            .await
    }

    pub async fn release_channel(&self, pair: ChannelPair) {
        self.channel_pool
            .release_channel(self.src.chain_id(), self.dst.chain_id(), pair)
            .await;
    }

    pub async fn get_available_channel_count(&self) -> usize {
        self.channel_pool
            .get_available_channel_count(self.src.chain_id(), self.dst.chain_id())
            .await
    }

    pub async fn send_and_recv<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        let packet_hash = source_chain.send_ibc_transaction(contract, msg).await?;
        destination_chain
            .wait_for_packet_recv(packet_hash, timeout)
            .await
    }
}

use std::{future::Future, sync::Arc, time::Duration};

use alloy::{contract::RawCallBuilder, network::AnyNetwork, providers::DynProvider};
// use axum::async_trait;
use cosmos_client::wallet::LocalSigner;
use ibc_union_spec::{
    path::{BatchPacketsPath, StorePath},
    ChannelId, MustBeZero, Packet,
};
use jsonrpsee::http_client::HttpClient;
use protos::cosmos::base::v1beta1::Coin;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bech32, Bytes, FixedBytes, H160, H256},
};
use voyager_sdk::{
    anyhow::{self, anyhow, Context},
    primitives::{ChainId, ClientType, IbcInterface, IbcSpecId, QueryHeight},
    rpc::VoyagerRpcClient,
    serde_json,
};
pub mod channel_provider;
pub mod cosmos;
pub mod evm;
pub mod helpers;
pub mod voyager;
use regex::Regex;

use crate::{
    channel_provider::{ChannelConfirm, ChannelPair, ChannelPool},
    evm::zkgm::FungibleAssetMetadata,
};

#[allow(async_fn_in_trait)]
pub trait ChainEndpoint: Send + Sync {
    type Msg: Clone;
    type Contract: Clone;
    type PredictWrappedTokenResponse;
    type PredictWrappedTokenFromMetadataImageV2Response;
    type ProviderType;

    fn chain_id(&self) -> &ChainId;

    async fn predict_wrapped_token(
        &self,
        contract: Self::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenResponse>;

    async fn predict_wrapped_token_from_metadata_image_v2(
        &self,
        contract: Self::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        metadata_image: FixedBytes<32>,
        provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenFromMetadataImageV2Response>;

    async fn predict_wrapped_token_v2(
        &self,
        contract: Self::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        metadata: FungibleAssetMetadata,
        provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenFromMetadataImageV2Response>;

    async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm>;

    async fn wait_for_open_connection(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm>;

    fn wait_for_open_channel(
        &self,
        timeout: Duration,
        expected_event_count: usize,
    ) -> impl Future<Output = anyhow::Result<Vec<helpers::ChannelOpenConfirm>>> + Send;

    async fn send_ibc_transaction(
        &self,
        contract: Self::Contract,
        msg: Self::Msg,
        signer: &Self::ProviderType,
    ) -> anyhow::Result<(H256, u64)>;

    async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv>;

    async fn wait_for_update_client(
        &self,
        height: u64,
        timeout: Duration,
    ) -> anyhow::Result<helpers::UpdateClient>;

    async fn wait_for_packet_timeout(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketTimeout>;

    async fn wait_for_packet_ack(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketAck>;

    async fn wait_for_delegate(
        &self,
        validator: String,
        timeout: Duration,
    ) -> anyhow::Result<helpers::Delegate>;

    async fn wait_for_withdraw_rewards(
        &self,
        validator: String,
        timeout: Duration,
    ) -> anyhow::Result<helpers::WithdrawRewards>;

    async fn calculate_proof(
        &self,
        source_channel_id: u32,
        destination_channel_id: u32,
        encoded_packet: Vec<u8>,
        height: u64,
        chain_id: &str,
    ) -> anyhow::Result<Bytes>;
}

pub trait IbcEventHash {
    type Hash;
}

impl<'a> ChainEndpoint for evm::Module<'a> {
    type Msg = RawCallBuilder<&'a DynProvider<AnyNetwork>, AnyNetwork>;
    type Contract = H160;
    type PredictWrappedTokenResponse = H160;
    type PredictWrappedTokenFromMetadataImageV2Response = H160;
    type ProviderType = DynProvider<AnyNetwork>;

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    async fn calculate_proof(
        &self,
        source_channel_id: u32,
        destination_channel_id: u32,
        encoded_packet: Vec<u8>,
        height: u64,
        chain_id: &str,
    ) -> anyhow::Result<Bytes> {
        let voyager_client = HttpClient::builder()
            .build("http://localhost:7178")
            .expect("REASON");

        let packets = &[Packet {
            source_channel_id: source_channel_id.try_into().unwrap(),
            destination_channel_id: destination_channel_id.try_into().unwrap(),
            data: encoded_packet.into(),
            timeout_height: MustBeZero,
            timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        }];
        let path = serde_json::to_value(StorePath::BatchPackets(BatchPacketsPath::from_packets(
            packets,
        )))
        .unwrap();

        let response = voyager_client
            .query_ibc_proof(
                ChainId::new(chain_id.to_string()),
                IbcSpecId::new(IbcSpecId::UNION),
                QueryHeight::Specific(Height::new(height)),
                path,
            )
            .await
            .unwrap();

        Ok(voyager_client
            .encode_proof(
                ClientType::new(ClientType::COMETBLS),
                IbcInterface::new(IbcInterface::IBC_SOLIDITY),
                IbcSpecId::new(IbcSpecId::UNION),
                response.clone().into_result().unwrap().proof,
            )
            .await
            .unwrap())
    }

    async fn wait_for_delegate(
        &self,
        _validator: String,
        _timeout: Duration,
    ) -> anyhow::Result<helpers::Delegate> {
        unimplemented!("wait_for_delegate is not implemented for Cosmos chains")
    }

    async fn wait_for_withdraw_rewards(
        &self,
        _validator: String,
        _timeout: Duration,
    ) -> anyhow::Result<helpers::WithdrawRewards> {
        unimplemented!("wait_for_withdraw_rewards is not implemented for Cosmos chains")
    }

    async fn predict_wrapped_token(
        &self,
        contract: Self::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenResponse> {
        self.predict_wrapped_token(contract, channel, token, provider)
            .await
    }

    async fn predict_wrapped_token_from_metadata_image_v2(
        &self,
        contract: Self::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        metadata_image: FixedBytes<32>,
        provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenFromMetadataImageV2Response> {
        self.predict_wrapped_token_from_metadata_image_v2(
            contract,
            channel,
            token,
            metadata_image,
            provider,
        )
        .await
    }

    async fn predict_wrapped_token_v2(
        &self,
        contract: Self::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        metadata: FungibleAssetMetadata,
        provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenFromMetadataImageV2Response> {
        self.predict_wrapped_token_v2(contract, channel, token, metadata, provider)
            .await
    }

    async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        self.wait_for_create_client(timeout).await
    }

    async fn wait_for_open_connection(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        self.wait_for_connection_open_confirm(timeout).await
    }

    async fn wait_for_open_channel(
        &self,
        timeout: Duration,
        expected_event_count: usize,
    ) -> anyhow::Result<Vec<helpers::ChannelOpenConfirm>> {
        self.wait_for_channel_open_confirm(timeout, expected_event_count)
            .await
    }

    async fn send_ibc_transaction(
        &self,
        contract: Self::Contract,
        msg: Self::Msg,
        signer: &Self::ProviderType,
    ) -> anyhow::Result<(H256, u64)> {
        self.send_ibc_transaction(contract, msg, signer)
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

    async fn wait_for_update_client(
        &self,
        height: u64,
        timeout: Duration,
    ) -> anyhow::Result<helpers::UpdateClient> {
        self.wait_for_update_client(height, timeout).await
    }

    async fn wait_for_packet_timeout(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketTimeout> {
        self.wait_for_packet_timeout(packet_hash, timeout).await
    }

    async fn wait_for_packet_ack(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketAck> {
        self.wait_for_packet_ack(packet_hash, timeout).await
    }
}

impl IbcEventHash for ibc_solidity::Ibc::PacketRecv {
    type Hash = H256;
}

impl ChainEndpoint for cosmos::Module {
    type Msg = (Vec<u8>, Vec<Coin>);
    type Contract = Bech32<H256>;
    type PredictWrappedTokenResponse = String;
    type PredictWrappedTokenFromMetadataImageV2Response = String;
    type ProviderType = LocalSigner;

    async fn calculate_proof(
        &self,
        _source_channel_id: u32,
        _destination_channel_id: u32,
        _encoded_packet: Vec<u8>,
        _height: u64,
        _chain_id: &str,
    ) -> anyhow::Result<Bytes> {
        unimplemented!("calculate_proof is not implemented for Cosmos chains")
    }

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        self.wait_for_create_client_id(timeout).await
    }

    async fn wait_for_open_connection(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        self.wait_for_connection_open_confirm(timeout).await
    }

    async fn predict_wrapped_token(
        &self,
        contract: Self::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        _provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenResponse> {
        self.predict_wrapped_token(contract, channel, token).await
    }

    async fn predict_wrapped_token_from_metadata_image_v2(
        &self,
        _contract: Self::Contract,
        _channel: ChannelId,
        _token: Vec<u8>,
        _metadata_image: FixedBytes<32>,
        _provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenFromMetadataImageV2Response> {
        unimplemented!(
            "predict_wrapped_token_from_metadata_image_v2 is not implemented for Cosmos chains"
        )
    }

    async fn predict_wrapped_token_v2(
        &self,
        _contract: Self::Contract,
        _channel: ChannelId,
        _token: Vec<u8>,
        _metadata: FungibleAssetMetadata,
        _provider: &Self::ProviderType,
    ) -> anyhow::Result<Self::PredictWrappedTokenFromMetadataImageV2Response> {
        unimplemented!("predict_wrapped_token_v2 is not implemented for Cosmos chains")
    }

    async fn wait_for_open_channel(
        &self,
        timeout: Duration,
        expected_event_count: usize,
    ) -> anyhow::Result<Vec<helpers::ChannelOpenConfirm>> {
        self.wait_for_channel_open_confirm(timeout, expected_event_count)
            .await
    }

    async fn send_ibc_transaction(
        &self,
        contract: Bech32<H256>,
        msg: Self::Msg,
        signer: &Self::ProviderType,
    ) -> anyhow::Result<(H256, u64)> {
        self.send_ibc_transaction(contract, msg, signer).await
    }

    async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        self.wait_for_packet_recv(packet_hash, timeout).await
    }

    async fn wait_for_update_client(
        &self,
        height: u64,
        timeout: Duration,
    ) -> anyhow::Result<helpers::UpdateClient> {
        self.wait_for_update_client(height, timeout).await
    }

    async fn wait_for_packet_timeout(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketTimeout> {
        self.wait_for_packet_timeout(packet_hash, timeout).await
    }

    async fn wait_for_packet_ack(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketAck> {
        self.wait_for_packet_ack(packet_hash, timeout).await
    }

    async fn wait_for_delegate(
        &self,
        validator: String,
        timeout: Duration,
    ) -> anyhow::Result<helpers::Delegate> {
        self.wait_for_delegate(validator, timeout).await
    }

    async fn wait_for_withdraw_rewards(
        &self,
        validator: String,
        timeout: Duration,
    ) -> anyhow::Result<helpers::WithdrawRewards> {
        self.wait_for_withdraw_rewards(validator, timeout).await
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
    pub channel_count: usize,
    pub voyager_config_file_path: String,
}

impl<S, D> TestContext<S, D>
where
    S: ChainEndpoint + 'static,
    D: ChainEndpoint + 'static,
{
    pub async fn new(
        src: S,
        dst: D,
        channel_count: usize,
        voyager_config_file_path: &str,
    ) -> anyhow::Result<Self> {
        voyager::init_fetch(voyager_config_file_path, src.chain_id().clone())?;
        voyager::init_fetch(voyager_config_file_path, dst.chain_id().clone())?;
        let channel_pool = ChannelPool::new();
        println!(
            "Creating test context for {} and {}. Init_fetch called for both chains.",
            src.chain_id(),
            dst.chain_id()
        );
        Ok(Self {
            src,
            dst,
            channel_pool,
            channel_count,
            voyager_config_file_path: voyager_config_file_path.into(),
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
        voyager::create_client(
            &self.voyager_config_file_path,
            self.src.chain_id().clone(),
            self.dst.chain_id().clone(),
            src_ibc_interface.into(),
            src_client_type.into(),
        )?;
        let src_confirm = self.src.wait_for_create_client(duration).await?;
        voyager::create_client(
            &self.voyager_config_file_path,
            self.dst.chain_id().clone(),
            self.src.chain_id().clone(),
            dst_ibc_interface.into(),
            dst_client_type.into(),
        )?;
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
        voyager::connection_open(src.chain_id().clone(), src_client_id, dst_client_id)?;
        dst.wait_for_open_connection(duration).await
    }

    #[allow(clippy::too_many_arguments)]
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
                        Ok(self
                            .dst
                            .wait_for_open_channel(t, count)
                            .await?
                            .into_iter()
                            .map(|ev| ChannelConfirm {
                                channel_id: ev.channel_id,
                                counterparty_channel_id: ev.counterparty_channel_id,
                            })
                            .collect::<Vec<_>>())
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
                    Ok(self
                        .src
                        .wait_for_open_channel(t, count)
                        .await?
                        .into_iter()
                        .map(|ev| ChannelConfirm {
                            channel_id: ev.channel_id,
                            counterparty_channel_id: ev.counterparty_channel_id,
                        })
                        .collect::<Vec<_>>())
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

    pub async fn predict_wrapped_token<Src: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        provider: &Src::ProviderType,
    ) -> anyhow::Result<Src::PredictWrappedTokenResponse> {
        source_chain
            .predict_wrapped_token(contract, channel, token, provider)
            .await
    }

    pub async fn predict_wrapped_token_from_metadata_image_v2<Src: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        metadata_image: FixedBytes<32>,
        provider: &Src::ProviderType,
    ) -> anyhow::Result<Src::PredictWrappedTokenFromMetadataImageV2Response> {
        source_chain
            .predict_wrapped_token_from_metadata_image_v2(
                contract,
                channel,
                token,
                metadata_image,
                provider,
            )
            .await
    }

    pub async fn predict_wrapped_token_v2<Src: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        channel: ChannelId,
        token: Vec<u8>,
        metadata: FungibleAssetMetadata,
        provider: &Src::ProviderType,
    ) -> anyhow::Result<Src::PredictWrappedTokenFromMetadataImageV2Response> {
        source_chain
            .predict_wrapped_token_v2(contract, channel, token, metadata, provider)
            .await
    }

    pub async fn send_and_recv<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<helpers::PacketRecv> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );
        match destination_chain
            .wait_for_packet_recv(packet_hash, timeout)
            .await
        {
            Ok(evt) => Ok(evt),
            Err(e) => anyhow::bail!("wait_for_packet_recv failed: {:?}", e),
        }
    }

    pub async fn send_and_recv_and_ack<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<helpers::PacketAck> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );

        match destination_chain
            .wait_for_packet_recv(packet_hash, timeout)
            .await
        {
            Ok(evt) => evt,
            Err(e) => anyhow::bail!("wait_for_packet_recv failed: {:?}", e),
        };

        match source_chain.wait_for_packet_ack(packet_hash, timeout).await {
            Ok(evt) => Ok(evt),
            Err(e) => anyhow::bail!("wait_for_packet_ack failed: {:?}", e),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send_and_recv_stake<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        validator: String,
        signer: Src::ProviderType,
    ) -> anyhow::Result<helpers::Delegate> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), &signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );

        let delegate = match destination_chain
            .wait_for_delegate(validator, timeout)
            .await
        {
            Ok(evt) => Ok(evt),
            Err(e) => anyhow::bail!("wait_for_packet_recv failed: {:?}", e),
        };

        let _ = source_chain
            .wait_for_packet_ack(packet_hash, timeout)
            .await
            .unwrap();

        delegate
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send_and_recv_unstake<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        validator: String,
        signer: Src::ProviderType,
    ) -> anyhow::Result<helpers::WithdrawRewards> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), &signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );

        let withdraw_rewards = match destination_chain
            .wait_for_withdraw_rewards(validator, timeout)
            .await
        {
            Ok(evt) => Ok(evt),
            Err(e) => anyhow::bail!("wait_for_packet_recv failed: {:?}", e),
        };

        match source_chain.wait_for_packet_ack(packet_hash, timeout).await {
            Ok(evt) => evt,
            Err(e) => anyhow::bail!("wait_for_packet_ack failed: {:?}", e),
        };

        withdraw_rewards
    }

    pub async fn send_and_recv_refund<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<helpers::PacketRecv> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );

        let packet_recved = match destination_chain
            .wait_for_packet_recv(packet_hash, timeout)
            .await
        {
            Ok(evt) => Ok(evt),
            Err(e) => anyhow::bail!("wait_for_packet_recv failed: {:?}", e),
        };

        match source_chain.wait_for_packet_ack(packet_hash, timeout).await {
            Ok(evt) => evt,
            Err(e) => anyhow::bail!("wait_for_packet_ack failed: {:?}", e),
        };

        packet_recved
    }

    pub async fn send_and_recv_ack<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<helpers::PacketAck> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );

        match destination_chain
            .wait_for_packet_recv(packet_hash, timeout)
            .await
        {
            Ok(evt) => evt,
            Err(e) => anyhow::bail!("wait_for_packet_recv failed: {:?}", e),
        };

        let packet_acked = match source_chain.wait_for_packet_ack(packet_hash, timeout).await {
            Ok(evt) => evt,
            Err(e) => anyhow::bail!("wait_for_packet_ack failed: {:?}", e),
        };

        Ok(packet_acked)
    }

    pub async fn send_and_recv_refund_with_timeout<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<helpers::PacketTimeout> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );

        match source_chain
            .wait_for_packet_timeout(packet_hash, timeout)
            .await
        {
            Ok(evt) => Ok(evt),
            Err(e) => anyhow::bail!("wait_for_packet_timeout failed: {:?}", e),
        }
    }

    pub async fn send_and_get_height<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<u64> {
        let (packet_hash, height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {} and with height: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash,
            height
        );

        let update_client_result: anyhow::Result<helpers::UpdateClient> = match destination_chain
            .wait_for_update_client(height, timeout)
            .await
        {
            Ok(evt) => Ok(evt),
            Err(e) => anyhow::bail!("wait_for_update_client failed: {:?}", e),
        };
        println!("Update client event received: {:?}", update_client_result);

        Ok(update_client_result.unwrap().height)
    }

    pub async fn send_and_expect_revert<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        expected_revert_code: u32,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<()> {
        match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), signer)
            .await
        {
            Ok((_, _)) => anyhow::bail!(
                "Expected revert 0x{:08x}, but transaction succeeded",
                expected_revert_code
            ),

            Err(e) => {
                let err_str = format!("{:#}", e);

                let re = Regex::new(r"(0x[0-9A-Fa-f]+)").unwrap();
                let caps = re.captures(&err_str).ok_or_else(|| {
                    anyhow!(
                        "Transaction reverted but no rawValue hex found: {}",
                        err_str
                    )
                })?;

                let hex_full = caps
                    .get(1)
                    .map(|m| m.as_str())
                    .ok_or_else(|| anyhow!("regex matched but capture group missing"))?;

                let hexdigits = hex_full.trim_start_matches("0x");
                if hexdigits.len() < 8 {
                    anyhow::bail!("rawValue too short for a selector: {}", hex_full);
                }
                let selector_hex = &hexdigits[..8]; // first 4 bytes

                let actual = u32::from_str_radix(selector_hex, 16).with_context(|| {
                    format!("parsing revert selector `{selector_hex}` from `{hex_full}`")
                })?;

                if actual == expected_revert_code {
                    Ok(())
                } else {
                    anyhow::bail!(
                        "Expected selector 0x{:08x}, got 0x{:08x} (raw: {})",
                        expected_revert_code,
                        actual,
                        hex_full
                    )
                }
            }
        }
    }

    pub async fn send_and_recv_withdraw<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        timeout: Duration,
        signer: Src::ProviderType,
    ) -> anyhow::Result<()> {
        let (packet_hash, _height) = match source_chain
            .send_ibc_transaction(contract.clone(), msg.clone(), &signer)
            .await
        {
            Ok(hash) => {
                println!("send_ibc_tx succeeded with hash: {:?}", hash);
                hash
            }
            Err(e) => {
                anyhow::bail!("send_ibc_transaction failed: {:?}", e);
            }
        };
        println!(
            "Packet sent from {} to {} with hash: {}",
            source_chain.chain_id(),
            destination_chain.chain_id(),
            packet_hash
        );

        // TODO: any other event is expected here?
        match source_chain.wait_for_packet_ack(packet_hash, timeout).await {
            Ok(evt) => evt,
            Err(e) => anyhow::bail!("wait_for_packet_ack failed: {:?}", e),
        };

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send_and_recv_with_retry<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        max_retries: usize,
        retry_delay: Duration,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<helpers::PacketRecv> {
        let mut attempt = 0;
        println!(
            "Starting send_and_recv_with_retry with max_retries: {}, retry_delay: {:?}",
            max_retries, retry_delay
        );
        loop {
            attempt += 1;
            match self
                .send_and_recv(
                    source_chain,
                    contract.clone(),
                    msg.clone(),
                    destination_chain,
                    timeout,
                    signer,
                )
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < max_retries {
                        println!("Attempt {} failed: {}. Retrying...", attempt, e);
                        tokio::time::sleep(retry_delay).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn send_and_recv_and_ack_with_retry<Src: ChainEndpoint, Dst: ChainEndpoint>(
        &self,
        source_chain: &Src,
        contract: Src::Contract,
        msg: Src::Msg,
        destination_chain: &Dst,
        max_retries: usize,
        retry_delay: Duration,
        timeout: Duration,
        signer: &Src::ProviderType,
    ) -> anyhow::Result<helpers::PacketAck> {
        let mut attempt = 0;
        println!(
            "Starting send_and_recv_with_retry with max_retries: {}, retry_delay: {:?}",
            max_retries, retry_delay
        );
        loop {
            attempt += 1;
            match self
                .send_and_recv_and_ack(
                    source_chain,
                    contract.clone(),
                    msg.clone(),
                    destination_chain,
                    timeout,
                    signer,
                )
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < max_retries {
                        println!("Attempt {} failed: {}. Retrying...", attempt, e);
                        tokio::time::sleep(retry_delay).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    pub async fn calculate_proof<Src: ChainEndpoint>(
        &self,
        source_chain: &Src,
        source_channel_id: u32,
        destination_channel_id: u32,
        encoded_packet: Vec<u8>,
        height: u64,
        chain_id: &str,
    ) -> anyhow::Result<Bytes> {
        source_chain
            .calculate_proof(
                source_channel_id,
                destination_channel_id,
                encoded_packet,
                height,
                chain_id,
            )
            .await
    }
}

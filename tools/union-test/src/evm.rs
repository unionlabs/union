use std::{marker::PhantomData, panic::AssertUnwindSafe, str::FromStr, time::Duration};

use alloy::{
    contract::{Error, RawCallBuilder, Result},
    network::{AnyNetwork, EthereumWallet},
    providers::{
        fillers::RecommendedFillers, DynProvider, PendingTransactionError, Provider,
        ProviderBuilder,
    },
    rpc::types::Filter,
    signers::local::LocalSigner,
    sol_types::SolEventInterface,
    transports::{RpcError, TransportError},
};
use bip32::secp256k1::ecdsa::{self, SigningKey};
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use ethers::{
    abi::{self, Token},
    utils::hex as ethers_hex,
};
use ibc_solidity::Ibc::IbcEvents;
use ibc_union_spec::{datagram::Datagram, ChannelId};
use jsonrpsee::{core::RpcResult, types::ErrorObjectOwned};
use serde::{Deserialize, Serialize};
use tracing::{error, info_span, warn, Instrument};
use unionlabs::{
    primitives::{FixedBytes, H160, H256, U256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow::{self},
    primitives::ChainId,
};

use crate::helpers;

#[derive(Debug)]
pub struct Module<'a> {
    pub chain_id: ChainId,

    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub provider: DynProvider<AnyNetwork>,

    pub keyring: ConcurrentKeyring<alloy::primitives::Address, LocalSigner<SigningKey>>,

    pub max_gas_price: Option<u128>,

    pub fixed_gas_price: Option<u128>,

    pub gas_multiplier: f64,

    pub _marker: PhantomData<&'a ()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub rpc_url: String,
    pub ws_url: String,

    pub keyring: KeyringConfig,

    #[serde(default)]
    pub max_gas_price: Option<u128>,

    #[serde(default)]
    pub fixed_gas_price: Option<u128>,

    pub gas_multiplier: f64,
}

impl<'a> Module<'a> {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.rpc_url)
                .await?,
        );

        let raw_chain_id = provider.get_chain_id().await?;
        let chain_id = ChainId::new(raw_chain_id.to_string());

        if chain_id != config.chain_id {
            panic!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            );
        }

        Ok(Self {
            chain_id,
            ibc_handler_address: config.ibc_handler_address,
            multicall_address: config.multicall_address,
            provider,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|config| {
                    let signing_key = <ecdsa::SigningKey as bip32::PrivateKey>::from_bytes(
                        &config.value().as_slice().try_into().unwrap(),
                    )
                    .unwrap();

                    let signer = LocalSigner::from_signing_key(signing_key);

                    KeyringEntry {
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            max_gas_price: config.max_gas_price,
            fixed_gas_price: config.fixed_gas_price,
            gas_multiplier: config.gas_multiplier,
            _marker: PhantomData,
        })
    }

    async fn wait_for_event<T: std::fmt::Debug, F: Fn(IbcEvents) -> Option<T>>(
        &self,
        filter_fn: F,
        timeout: Duration,
        expected_event_count: usize,
    ) -> anyhow::Result<Vec<T>> {
        tokio::time::timeout(timeout, async {
            let mut prev_latest = self.provider.get_block_number().await?;
            let mut events = Vec::new();
            loop {
                let latest = self.provider.get_block_number().await?;

                if events.len() >= expected_event_count {
                    return Ok(events);
                }

                if prev_latest >= latest {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }

                while prev_latest <= latest {
                    let filter = Filter::new()
                        .address(alloy::primitives::Address::from(self.ibc_handler_address))
                        .from_block(prev_latest)
                        .to_block(prev_latest);

                    let logs = match self.provider.get_logs(&filter).await {
                        Ok(logs) => logs,
                        Err(e) => {
                            return Err(anyhow::anyhow!("get_logs RPC error: {}", e));
                        }
                    };
                    for log in logs {
                        if let Ok(ibc_event) = IbcEvents::decode_log(&log.inner) {
                            if let Some(event) = filter_fn(ibc_event.data) {
                                println!("Event found: {:?}", event);
                                events.push(event);
                            }
                        }
                    }

                    prev_latest += 1u64;
                }

                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        })
        .await
        .map_err(|_| anyhow::anyhow!("timed out after {:?}", timeout))?
    }

    pub async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        Ok(self
            .wait_for_event(
                |e| match e {
                    IbcEvents::CreateClient(ev) => Some(helpers::CreateClientConfirm {
                        client_id: ev.client_id,
                    }),
                    _ => None,
                },
                timeout,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_connection_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        Ok(self
            .wait_for_event(
                |e| match e {
                    IbcEvents::ConnectionOpenConfirm(ev) => Some(helpers::ConnectionConfirm {
                        connection_id: ev.connection_id,
                        counterparty_connection_id: ev.counterparty_connection_id,
                    }),
                    _ => None,
                },
                timeout,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_channel_open_confirm(
        &self,
        timeout: Duration,
        expected_event_count: usize,
    ) -> anyhow::Result<Vec<helpers::ChannelOpenConfirm>> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::ChannelOpenConfirm(ev) => Some(helpers::ChannelOpenConfirm {
                    channel_id: ev.channel_id,
                    counterparty_channel_id: ev.counterparty_channel_id,
                }),
                _ => None,
            },
            timeout,
            expected_event_count,
        )
        .await
    }

    pub async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        Ok(self
            .wait_for_event(
                |e| match e {
                    IbcEvents::PacketRecv(ev)
                        if ev.packet_hash.as_slice() == packet_hash.as_ref() =>
                    {
                        Some(helpers::PacketRecv {
                            packet_hash: ev.packet_hash.try_into().unwrap(),
                        })
                    }
                    _ => None,
                },
                timeout,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_packet_ack(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketAck> {
        Ok(self
            .wait_for_event(
                |e| match e {
                    IbcEvents::PacketAck(ev)
                        if ev.packet_hash.as_slice() == packet_hash.as_ref() =>
                    {
                        Some(helpers::PacketAck {
                            packet_hash: ev.packet_hash.try_into().unwrap(),
                        })
                    }
                    _ => None,
                },
                timeout,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    /// Retry an async operation up to `max_attempts` times with a small delay between attempts.
    async fn retry_with_backoff<F, Fut, T, E>(
        &self,
        mut operation: F,
        max_attempts: usize,
        backoff: std::time::Duration,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        let mut last_err = None;
        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(val) => return Ok(val),
                Err(err) if attempt < max_attempts => {
                    tracing::warn!(attempt, "operation failed, retrying...");
                    last_err = Some(err);
                    tokio::time::sleep(backoff).await;
                }
                Err(err) => return Err(err),
            }
        }
        Err(last_err.expect("retry_with_backoff called with zero attempts"))
    }

    pub async fn predict_wrapped_token(
        &self,
        ucs03_addr_on_evm: H160,
        channel: ChannelId,
        token: Vec<u8>,
        provider: &DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H160> {
        let ucs03_zkgm = zkgm::UCS03Zkgm::new(ucs03_addr_on_evm.into(), provider);
        let unwrapped = self
            .retry_with_backoff(
                || {
                    let call = ucs03_zkgm.predictWrappedToken(
                        U256::from(0u32).into(),
                        channel.raw().try_into().unwrap(),
                        token.clone().into(),
                    );
                    async move { call.call().await }
                },
                3,
                Duration::from_secs(5),
            )
            .await?;

        Ok(unwrapped._0.into())
    }

    pub async fn predict_wrapped_token_v2(
        &self,
        ucs03_addr_on_evm: H160,
        channel: ChannelId,
        token: Vec<u8>,
        metadata: zkgm::FungibleAssetMetadata,
        provider: &DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H160> {
        let ucs03_zkgm = zkgm::UCS03Zkgm::new(ucs03_addr_on_evm.into(), provider);
        let ret = self
            .retry_with_backoff(
                || {
                    let call = ucs03_zkgm.predictWrappedTokenV2(
                        U256::from(0u32).into(),
                        channel.raw().try_into().unwrap(),
                        token.clone().into(),
                        metadata.clone().into(),
                    );
                    async move { call.call().await }
                },
                3,
                Duration::from_secs(2),
            )
            .await?;

        Ok(ret._0.0.into())
    }

    pub async fn predict_wrapped_token_from_metadata_image_v2(
        &self,
        ucs03_addr_on_evm: H160,
        channel: ChannelId,
        token: Vec<u8>,
        metadata_image: FixedBytes<32>,
        provider: &DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H160> {
        let ucs03_zkgm = zkgm::UCS03Zkgm::new(ucs03_addr_on_evm.into(), provider);
        let ret = self
            .retry_with_backoff(
                || {
                    let call = ucs03_zkgm.predictWrappedTokenFromMetadataImageV2(
                        U256::from(0u32).into(),
                        channel.raw().try_into().unwrap(),
                        token.clone().into(),
                        metadata_image.into(),
                    );
                    async move { call.call().await }
                },
                3,
                Duration::from_secs(2),
            )
            .await?;
        Ok(ret._0.0.into())
    }

    pub async fn get_provider(&self) -> (alloy::primitives::Address, DynProvider<AnyNetwork>) {
        let wallet = loop {
            if let Some(w) = self.keyring.with(|w| async move { w.clone() }).await {
                break w;
            } else {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        };

        let wallet_addr = wallet.address();
        let provider = self.get_provider_with_wallet(&wallet).await;
        (wallet_addr, provider)
    }

    async fn get_provider_with_wallet(
        &self,
        wallet: &LocalSigner<SigningKey>,
    ) -> DynProvider<AnyNetwork> {
        DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .filler(AnyNetwork::recommended_fillers())
                .wallet(EthereumWallet::new(wallet.clone()))
                .connect_provider(self.provider.clone()),
        )
    }

    pub async fn wait_for_tx_inclusion(
        &self,
        provider: &DynProvider<AnyNetwork>,
        tx_hash: H256,
    ) -> anyhow::Result<(), TxSubmitError> {
        let mut attempts = 0;
        loop {
            let maybe_rcpt = provider
                .get_transaction_receipt(tx_hash.into())
                .await
                .map_err(|_rpc_err| TxSubmitError::InclusionError)?;

            if let Some(rcpt) = maybe_rcpt {
                println!("✅ tx {tx_hash:?} mined in block {:?}", rcpt.block_number);
                return Ok(());
            }
            if attempts <= 5 {
                attempts += 1;
                println!("receipt not yet available, retry {attempts}/5…");
                tokio::time::sleep(Duration::from_secs(4)).await;
                continue;
            }
            return Err(TxSubmitError::BatchTooLarge);
        }
    }

    fn is_nonce_too_low(&self, e: &Error) -> bool {
        if let Error::TransportError(TransportError::ErrorResp(rpc)) = e {
            println!("Nonce is too low, error entered here.: {:?}", rpc);
            rpc.message.contains("nonce too low")
        } else {
            false
        }
    }

    pub async fn zkgmerc20_approve(
        &self,
        contract: H160,
        spender: H160,
        amount: U256,
        provider: DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H256> {
        let mut attempts = 0;
        let erc = zkgmerc20::ZkgmERC20::new(contract.into(), provider.clone());
        loop {
            attempts += 1;
            let pending = erc.approve(spender.into(), amount.into()).send().await;

            match pending {
                Ok(pending) => {
                    let tx_hash = <H256>::from(*pending.tx_hash());
                    println!("pending: {:?}", pending);
                    self.wait_for_tx_inclusion(&provider, tx_hash).await?;
                    println!("Approved spender: {spender:?} for amount: {amount:?} on contract: {contract:?}");
                    return Ok(tx_hash);
                }
                Err(err) if attempts <= 5 && self.is_nonce_too_low(&err) => {
                    println!("Nonce too low, retrying... Attempt: {attempts}");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }

                Err(err) => {
                    return Err(err.into());
                }
            }
        }
    }

    pub async fn zkgmerc721_approve(
        &self,
        contract: H160,
        spender: H160,
        token_id: U256,
        provider: DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H256> {
        let mut attempts = 0;
        let erc = zkgm::ZkgmERC721::new(contract.into(), provider.clone());
        loop {
            attempts += 1;
            let pending = erc.approve(spender.into(), token_id.into()).send().await;

            match pending {
                Ok(pending) => {
                    let tx_hash = <H256>::from(*pending.tx_hash());
                    println!("pending: {:?}", pending);
                    self.wait_for_tx_inclusion(&provider, tx_hash).await?;
                    println!("Approved spender: {spender:?} for token_id: {token_id:?} on contract: {contract:?}");
                    return Ok(tx_hash);
                }
                Err(err) if attempts <= 5 && self.is_nonce_too_low(&err) => {
                    println!("Nonce too low, retrying... Attempt: {attempts}");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }

                Err(err) => {
                    return Err(err.into());
                }
            }
        }
    }

    pub async fn nft_owner_of(
        &self,
        contract: H160,
        owner: H160,
        token_id: U256,
        provider: DynProvider<AnyNetwork>,
    ) -> anyhow::Result<bool> {
        let actual_owner = self
            .retry_with_backoff(
                || {
                    let erc = basic_erc721::BasicERC721::new(contract.into(), provider.clone());
                    async move { erc.ownerOf(token_id.into()).call().await }
                },
                3,
                Duration::from_secs(2),
            )
            .await?;

        let owner_addr: alloy::primitives::Address = owner.into();
        Ok(actual_owner == owner_addr)
    }

    pub async fn basic_erc721_transfer_from(
        &self,
        contract: H160,
        from: H160,
        to: H160,
        token_id: U256,
        provider: DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H256> {
        let erc = basic_erc721::BasicERC721::new(contract.into(), provider.clone());
        let pending = erc
            .transferFrom(from.into(), to.into(), token_id.into())
            .send()
            .await?;
        let tx_hash = <H256>::from(*pending.tx_hash());
        Ok(tx_hash)
    }

    pub async fn deploy_basic_erc20(
        &self,
        spender: H160,
        provider: DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H160> {
        const BYTECODE: &str = "0x608060405234801561000f575f5ffd5b50604051610b91380380610b9183398101604081905261002e916102f0565b6040518060400160405280600481526020016311dbdb1960e21b8152506040518060400160405280600381526020016211d31160ea1b815250816003908161007691906103c1565b50600461008382826103c1565b50505061009633836100a860201b60201c565b6100a13382846100e5565b50506104a0565b6001600160a01b0382166100d65760405163ec442f0560e01b81525f60048201526024015b60405180910390fd5b6100e15f83836100f7565b5050565b6100f2838383600161021d565b505050565b6001600160a01b038316610121578060025f828254610116919061047b565b909155506101919050565b6001600160a01b0383165f90815260208190526040902054818110156101735760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016100cd565b6001600160a01b0384165f9081526020819052604090209082900390555b6001600160a01b0382166101ad576002805482900390556101cb565b6001600160a01b0382165f9081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161021091815260200190565b60405180910390a3505050565b6001600160a01b0384166102465760405163e602df0560e01b81525f60048201526024016100cd565b6001600160a01b03831661026f57604051634a1406b160e11b81525f60048201526024016100cd565b6001600160a01b038085165f90815260016020908152604080832093871683529290522082905580156102ea57826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516102e191815260200190565b60405180910390a35b50505050565b5f5f60408385031215610301575f5ffd5b825160208401519092506001600160a01b038116811461031f575f5ffd5b809150509250929050565b634e487b7160e01b5f52604160045260245ffd5b600181811c9082168061035257607f821691505b60208210810361037057634e487b7160e01b5f52602260045260245ffd5b50919050565b601f8211156100f257805f5260205f20601f840160051c8101602085101561039b5750805b601f840160051c820191505b818110156103ba575f81556001016103a7565b5050505050565b81516001600160401b038111156103da576103da61032a565b6103ee816103e8845461033e565b84610376565b6020601f821160018114610420575f83156104095750848201515b5f19600385901b1c1916600184901b1784556103ba565b5f84815260208120601f198516915b8281101561044f578785015182556020948501946001909201910161042f565b508482101561046c57868401515f19600387901b60f8161c191681555b50505050600190811b01905550565b8082018082111561049a57634e487b7160e01b5f52601160045260245ffd5b92915050565b6106e4806104ad5f395ff3fe608060405234801561000f575f5ffd5b5060043610610090575f3560e01c8063313ce56711610063578063313ce567146100fa57806370a082311461010957806395d89b4114610131578063a9059cbb14610139578063dd62ed3e1461014c575f5ffd5b806306fdde0314610094578063095ea7b3146100b257806318160ddd146100d557806323b872dd146100e7575b5f5ffd5b61009c610184565b6040516100a99190610554565b60405180910390f35b6100c56100c03660046105a4565b610214565b60405190151581526020016100a9565b6002545b6040519081526020016100a9565b6100c56100f53660046105cc565b61022d565b604051601281526020016100a9565b6100d9610117366004610606565b6001600160a01b03165f9081526020819052604090205490565b61009c610250565b6100c56101473660046105a4565b61025f565b6100d961015a366004610626565b6001600160a01b039182165f90815260016020908152604080832093909416825291909152205490565b60606003805461019390610657565b80601f01602080910402602001604051908101604052809291908181526020018280546101bf90610657565b801561020a5780601f106101e15761010080835404028352916020019161020a565b820191905f5260205f20905b8154815290600101906020018083116101ed57829003601f168201915b5050505050905090565b5f3361022181858561026c565b60019150505b92915050565b5f3361023a85828561027e565b6102458585856102ff565b506001949350505050565b60606004805461019390610657565b5f336102218185856102ff565b610279838383600161035c565b505050565b6001600160a01b038381165f908152600160209081526040808320938616835292905220545f198110156102f957818110156102eb57604051637dc7a0d960e11b81526001600160a01b038416600482015260248101829052604481018390526064015b60405180910390fd5b6102f984848484035f61035c565b50505050565b6001600160a01b03831661032857604051634b637e8f60e11b81525f60048201526024016102e2565b6001600160a01b0382166103515760405163ec442f0560e01b81525f60048201526024016102e2565b61027983838361042e565b6001600160a01b0384166103855760405163e602df0560e01b81525f60048201526024016102e2565b6001600160a01b0383166103ae57604051634a1406b160e11b81525f60048201526024016102e2565b6001600160a01b038085165f90815260016020908152604080832093871683529290522082905580156102f957826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161042091815260200190565b60405180910390a350505050565b6001600160a01b038316610458578060025f82825461044d919061068f565b909155506104c89050565b6001600160a01b0383165f90815260208190526040902054818110156104aa5760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016102e2565b6001600160a01b0384165f9081526020819052604090209082900390555b6001600160a01b0382166104e457600280548290039055610502565b6001600160a01b0382165f9081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161054791815260200190565b60405180910390a3505050565b602081525f82518060208401528060208501604085015e5f604082850101526040601f19601f83011684010191505092915050565b80356001600160a01b038116811461059f575f5ffd5b919050565b5f5f604083850312156105b5575f5ffd5b6105be83610589565b946020939093013593505050565b5f5f5f606084860312156105de575f5ffd5b6105e784610589565b92506105f560208501610589565b929592945050506040919091013590565b5f60208284031215610616575f5ffd5b61061f82610589565b9392505050565b5f5f60408385031215610637575f5ffd5b61064083610589565b915061064e60208401610589565b90509250929050565b600181811c9082168061066b57607f821691505b60208210810361068957634e487b7160e01b5f52602260045260245ffd5b50919050565b8082018082111561022757634e487b7160e01b5f52601160045260245ffdfea26469706673582212207cfaf374f2b49d6608fed1bc9f4e741950ec2c77f36bb3d89258c5218167ab4764736f6c634300081e0033";
        let mut code = ethers_hex::decode(BYTECODE.trim_start_matches("0x"))?;
        let initial_supply = ethers::types::U256::from_dec_str("1000000000000000000")?;
        let spender = ethers::types::Address::from_str(&spender.to_string())?;
        let encoded = abi::encode(&[Token::Uint(initial_supply.into()), Token::Address(spender)]);

        code.extend(&encoded);

        let mut attempts = 0;
        loop {
            attempts += 1;

            let mut call = RawCallBuilder::new_raw_deploy(provider.clone(), code.clone().into());
            let gas_est = call.estimate_gas().await?;
            call = call.gas((gas_est as f64 * 2.2) as u64);
            let pending = call.send().await;
            match pending {
                Ok(pending) => {
                    let tx_hash = *pending.tx_hash();
                    self.wait_for_tx_inclusion(&provider, tx_hash.into())
                        .await?;
                    let receipt = pending.get_receipt().await?;

                    let address = receipt
                        .contract_address
                        .expect("deploy didnt return an address");
                    return Ok(address.into());
                }
                Err(err) if attempts <= 5 && self.is_nonce_too_low(&err) => {
                    println!("Nonce too low, retrying... Attempt: {attempts}");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
                Err(err) => {
                    return Err(anyhow::anyhow!("Failed to deploy contract: {:?}", err).into());
                }
            }
        }
    }

    pub async fn send_ibc_transaction(
        &self,
        _contract: H160,
        msg: RawCallBuilder<&DynProvider<AnyNetwork>, AnyNetwork>,
        _provider: &DynProvider<AnyNetwork>,
    ) -> RpcResult<FixedBytes<32>> {
        let res = self
            .keyring
            .with({
                let msg = msg.clone();
                move |wallet| -> _ { AssertUnwindSafe(self.submit_transaction(wallet, msg)) }
            })
            .await;

        match res {
            Some(Ok(tx_hash)) => {
                let receipt_with = self
                    .provider
                    .get_transaction_receipt(tx_hash.into())
                    .await
                    .map_err(|e| {
                        ErrorObjectOwned::owned(
                            -1,
                            format!("failed to fetch receipt: {:?}", e),
                            None::<()>,
                        )
                    })?
                    .ok_or_else(|| ErrorObjectOwned::owned(-1, "receipt not found", None::<()>))?;

                let logs = &receipt_with.inner.inner.inner.receipt.logs;

                for raw in logs {
                    if let Ok(alloy_log) = IbcEvents::decode_log(&raw.inner) {
                        if let IbcEvents::PacketSend(ev) = alloy_log.data {
                            return Ok(ev.packet_hash.into());
                        }
                    }
                }

                Err(ErrorObjectOwned::owned(
                    -1,
                    "no PacketSend event found in transaction",
                    None::<()>,
                ))
            }

            Some(Err(e)) => Err(ErrorObjectOwned::owned(
                -1,
                format!("transaction submission failed: {:?}", e),
                None::<()>,
            )),

            None => Err(ErrorObjectOwned::owned(
                -1,
                "no signers available",
                None::<()>,
            )),
        }
    }

    pub async fn submit_transaction(
        &self,
        _wallet: &LocalSigner<SigningKey>,
        mut call: RawCallBuilder<&DynProvider<AnyNetwork>, AnyNetwork>,
    ) -> Result<H256, TxSubmitError> {
        if let Some(max_gas_price) = self.max_gas_price {
            let gas_price = self
                .provider
                .get_gas_price()
                .await
                .expect("unable to fetch gas price");

            if gas_price > max_gas_price {
                warn!(%max_gas_price, %gas_price, "gas price is too high");

                return Err(TxSubmitError::GasPriceTooHigh {
                    max: self.max_gas_price.expect("max gas price is set"),
                    price: gas_price,
                });
            } else {
                println!("gas price: {}", gas_price);
            }
        }

        println!("submitting evm tx");

        let gas_estimate = call.estimate_gas().await.map_err(|e| {
            println!("error estimating gas: {:?}", e);
            if ErrorReporter(&e)
                .to_string()
                .contains("gas required exceeds")
            {
                TxSubmitError::BatchTooLarge
            } else {
                TxSubmitError::Estimate(e)
            }
        })?;

        let gas_to_use = ((gas_estimate as f64) * self.gas_multiplier) as u64;
        println!(
            "gas estimate: {gas_estimate}, gas to use: {gas_to_use}, gas multiplier: {}",
            self.gas_multiplier
        );

        if let Some(fixed) = self.fixed_gas_price {
            call = call.gas_price(fixed);
        }

        match call.gas(gas_to_use).send().await {
            Ok(ok) => {
                let tx_hash = <H256>::from(*ok.tx_hash());
                async move {
                    self.wait_for_tx_inclusion(&self.provider, tx_hash).await?;
                    println!("tx included: {:?}", tx_hash);

                    Ok(tx_hash)
                }
                .instrument(info_span!("evm tx", %tx_hash))
                .await
            }

            Err(
                Error::PendingTransactionError(PendingTransactionError::TransportError(
                    TransportError::ErrorResp(e),
                ))
                | Error::TransportError(TransportError::ErrorResp(e)),
            ) if e
                .message
                .contains("insufficient funds for gas * price + value") =>
            {
                error!("out of gas");
                return Err(TxSubmitError::OutOfGas);
            }

            Err(
                Error::PendingTransactionError(PendingTransactionError::TransportError(
                    TransportError::ErrorResp(e),
                ))
                | Error::TransportError(TransportError::ErrorResp(e)),
            ) if e.message.contains("oversized data")
                || e.message.contains("exceeds block gas limit")
                || e.message.contains("gas required exceeds") =>
            {
                return Err(TxSubmitError::BatchTooLarge);
            }

            Err(Error::PendingTransactionError(PendingTransactionError::FailedToRegister)) => {
                return Err(TxSubmitError::PendingTransactionError(
                    PendingTransactionError::FailedToRegister,
                ));
            }

            Err(err) => return Err(TxSubmitError::Error(err)),
        }
    }

    pub async fn predict_stake_manager_address(
        &self,
        zkgm_addr: H160,
        provider: DynProvider<AnyNetwork>,
    ) -> anyhow::Result<H160> {
        let zkgm = zkgm::UCS03Zkgm::new(zkgm_addr.into(), provider.clone());

        let addr = self
            .retry_with_backoff(
                || {
                    let call = zkgm.predictStakeManagerAddress();
                    async move { call.call().await }
                },
                3,
                Duration::from_secs(2),
            )
            .await?;
        Ok(addr.into())
    }

    pub async fn setup_governance_token(
        &self,
        zkgm_addr: H160,
        channel_id: u32,
        metadata_image: FixedBytes<32>,
        provider: DynProvider<AnyNetwork>,
    ) -> Result<H256, TxSubmitError> {
        let zkgm = zkgm::UCS03Zkgm::new(zkgm_addr.into(), provider.clone());

        let pending = zkgm
            .registerGovernanceToken(
                channel_id,
                zkgm::GovernanceToken {
                    unwrappedToken: b"muno".into(),
                    metadataImage: metadata_image.into(),
                },
            )
            .send()
            .await?;

        let tx_hash = <H256>::from(*pending.tx_hash());

        self.wait_for_tx_inclusion(&provider, tx_hash).await?;
        Ok(tx_hash)
    }
}

pub mod zkgm {
    alloy::sol! {
        #![sol(rpc)]
        struct GovernanceToken {
            bytes    unwrappedToken;
            bytes32  metadataImage;
        }


        struct Instruction {
            uint8 version;
            uint8 opcode;
            bytes operand;
        }

        struct FungibleAssetMetadata {
            bytes implementation;
            bytes initializer;
        }


        contract ZkgmERC721 {
            function mint(uint256 tokenId, address to ) external;
            function burn(uint256 tokenId) external;
            function ownerOf(uint256 tokenId) external view returns (address);
            function approve(address to, uint256 tokenId) external;
            function transferFrom(address from, address to, uint256 tokenId) external;
            function setApprovalForAll(address operator, bool approved) external;
            function getApproved(uint256 tokenId) external view returns (address);
            function isApprovedForAll(address owner, address operator) external view returns (bool);
        }


        contract UCS03Zkgm {
            function send(
                uint32 channelId,
                uint64 timeoutHeight,
                uint64 timeoutTimestamp,
                bytes32 salt,
                Instruction calldata instruction
            ) public payable;

            function registerGovernanceToken(
                uint32 channelId,
                GovernanceToken calldata govToken
            ) external;

            function getGovernanceToken(
                uint32 channelId
            ) external view returns (address wrappedGovernanceToken, GovernanceToken governanceToken);

            function predictStakeManagerAddress() public view returns (ZkgmERC721) ;

            function predictWrappedToken(
                uint256 path,
                uint32 channel,
                bytes calldata token
            ) external view returns (address, bytes32);

            function predictWrappedTokenFromMetadataImageV2(
                uint256 path,
                uint32 channel,
                bytes calldata token,
                bytes32 metadataImage
            ) public returns (address, bytes32);

            function predictWrappedTokenV2(
                uint256 path,
                uint32 channel,
                bytes calldata token,
                FungibleAssetMetadata calldata metadata
            ) public returns (address, bytes32);
        }
    }
}

pub mod basic_erc721 {
    alloy::sol! {
        #![sol(rpc)]
        contract BasicERC721 {
            function mint(uint256 tokenId, address to) external;
            function approve(address spender, uint256 tokenId) external;
            function transferFrom(address from, address to, uint256 tokenId) external;

            function ownerOf(uint256 tokenId) external view returns (address);
        }
    }
}

pub mod zkgmerc20 {
    alloy::sol! {
        #![sol(rpc)]

        contract ZkgmERC20 {

            struct InitializeParams {
                address authority;
                address minter;
                string name;
                string symbol;
                uint8 decimals;
            }
            function initialize(
                address _authority,
                address _minter,
                string memory _name,
                string memory _symbol,
                uint8 _decimals
            ) external;

            function mint(
                address to,
                uint256 amount
            ) external;

            function approve(address spender, uint256 value) public returns (bool);
        }
    }
}

pub mod multicall {
    alloy::sol! {
        #![sol(rpc)]

        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }

        #[derive(Debug)]
        struct Result {
            bool success;
            bytes returnData;
        }

        #[derive(Debug)]
        event MulticallResult(Result[]);

        contract Multicall {
            function multicall(
                Call3[] calldata calls
            ) public payable returns (Result[] memory returnData);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Error(#[from] Error),
    #[error("error estimating gas")]
    Estimate(#[source] Error),
    #[error("error waiting for transaction")]
    PendingTransactionError(#[from] PendingTransactionError),
    #[error("out of gas")]
    OutOfGas,
    #[error("inclusion never happened")]
    InclusionError,
    #[error("0x revert")]
    EmptyRevert(Vec<Datagram>),
    #[error("gas price is too high: max {max}, price {price}")]
    GasPriceTooHigh { max: u128, price: u128 },
    #[error("rpc error (this is just the IbcDatagram conversion functions but i need to make those errors better)")]
    RpcError(#[from] ErrorObjectOwned),
    #[error("batch too large")]
    BatchTooLarge,
    #[error("rpc transport error waiting for receipt: {0}")]
    RpcTransport(#[from] RpcError<TransportError>),
}

use std::{marker::PhantomData, str::FromStr, panic::AssertUnwindSafe, time::Duration};

use alloy::{
    contract::{self, Error, RawCallBuilder, Result},
    network::{AnyNetwork, EthereumWallet},
    primitives::{Bytes as AlloyBytes, TxHash},
    providers::{
        fillers::RecommendedFillers, layers::CacheLayer, DynProvider, PendingTransactionError,
        Provider, ProviderBuilder,
    },
    rpc::types::{self, AnyReceiptEnvelope, Filter, Log, TransactionReceipt},
    signers::local::LocalSigner,
    sol_types::SolEventInterface,
    transports::TransportError,
};
use ethers::abi::{self, Token};

use bip32::secp256k1::ecdsa::{self, SigningKey};
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use cosmos_client::rpc::Rpc;
use ibc_solidity::Ibc::{
    self, ChannelOpenConfirm, ConnectionOpenConfirm, CreateClient, IbcErrors, IbcEvents, PacketRecv,
};
use ibc_union_spec::{datagram::Datagram, IbcUnion, ChannelId};
// use voyager_sdk::plugin::Plugin::
// use crate::multicall::{Call3, Multicall, MulticallResult};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
    Extensions, MethodsError,
};
use multicall::{Call3, Multicall, MulticallResult};
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{error, info, info_span, instrument, trace, warn, Instrument};
use unionlabs::{
    primitives::{Bytes, FixedBytes, H160, H256, U256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow::{self, anyhow, bail},
    into_value,
    primitives::ChainId,
};

use crate::{evm::zkgm::GovernanceToken, helpers};
use ethers::utils::hex as ethers_hex;
use hex_literal::hex;

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

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub rpc_url: String,

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

    async fn wait_for_event<T, F: Fn(IbcEvents) -> Option<T>>(
        &self,
        filter_fn: F,
        timeout: Duration,
    ) -> anyhow::Result<T> {
        tokio::time::timeout(timeout, async {
            let mut prev_latest = self.provider.get_block_number().await?;
            loop {
                let latest = self.provider.get_block_number().await?;

                if prev_latest >= latest {
                    tokio::time::sleep(Duration::from_secs(5)).await;
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
                                return Ok(event);
                            }
                        }
                    }

                    prev_latest += 1u64;
                }

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        })
        .await
        .map_err(|_| anyhow::anyhow!("timed out after {:?}", timeout))?
    }

    pub async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::CreateClient(ev) => Some(helpers::CreateClientConfirm {
                    client_id: ev.client_id,
                }),
                _ => None,
            },
            timeout,
        )
        .await
    }

    pub async fn wait_for_connection_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::ConnectionOpenConfirm(ev) => Some(helpers::ConnectionConfirm {
                    connection_id: ev.connection_id,
                    counterparty_connection_id: ev.counterparty_connection_id,
                }),
                _ => None,
            },
            timeout,
        )
        .await
    }

    pub async fn wait_for_channel_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ChannelOpenConfirm> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::ChannelOpenConfirm(ev) => Some(helpers::ChannelOpenConfirm {
                    channel_id: ev.channel_id,
                    counterparty_channel_id: ev.counterparty_channel_id,
                }),
                _ => None,
            },
            timeout,
        )
        .await
    }

    pub async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::PacketRecv(ev) if ev.packet_hash.as_slice() == packet_hash.as_ref() => {
                    Some(helpers::PacketRecv {
                        packet_hash: ev.packet_hash.try_into().unwrap(),
                    })
                }
                _ => None,
            },
            timeout,
        )
        .await
    }

    pub async fn wait_for_send_packet(
        &self,
        channel_id: u32,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketSend> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::PacketSend(ev) if ev.channel_id == channel_id => {
                    Some(helpers::PacketSend {
                        packet_hash: ev.packet_hash.try_into().unwrap(),
                    })
                }
                _ => None,
            },
            timeout,
        )
        .await
    }

    pub async fn predict_wrapped_token(
        &self,
        ucs03_addr_on_evm: H160,
        channel: ChannelId,
        token: Vec<u8>,
    ) -> anyhow::Result<H160> {
        let signer = self.get_provider().await;
        let ucs03_zkgm = zkgm::UCS03Zkgm::new(ucs03_addr_on_evm.into(), signer);
        // let path: U256 = ::from(0);
        let mut call = ucs03_zkgm.predictWrappedToken(
            U256::from(0u32).into(),
            channel.raw().try_into().unwrap(),
            token.into()
        );   

        let ret = call.call().await;
        let unwrapped = ret.unwrap();
        let wrapped_token: H160 = unwrapped._0.into();
        Ok(wrapped_token)
    }


    pub async fn predict_wrapped_token_from_metadata_image_v2(
        &self,
        ucs03_addr_on_evm: H160,
        channel: ChannelId,
        token: Vec<u8>,
        metadata_image: FixedBytes<32>,
    ) -> anyhow::Result<H160> {
        let signer = self.get_provider().await;
        let ucs03_zkgm = zkgm::UCS03Zkgm::new(ucs03_addr_on_evm.into(), signer);
        let mut call = ucs03_zkgm.predictWrappedTokenFromMetadataImageV2(
            U256::from(0u32).into(),
            channel.raw().try_into().unwrap(),
            token.into(),
            metadata_image.into()
        );   

        let ret = call.call().await;
        let unwrapped = ret.unwrap();
        let wrapped_token: H160 = unwrapped._0.into();
        Ok(wrapped_token)
    }

    // async fn submit_transaction_zkgm(
    //     &self,
    //     ucs03_addr_on_evm: H160,
    //     wallet: &LocalSigner<SigningKey>,
    //     send_call_struct: zkgm::UCS03Zkgm::sendCall,
    // ) -> Result<H256, TxSubmitError> {
    //     let signer = DynProvider::new(
    //         ProviderBuilder::new()
    //             .network::<AnyNetwork>()
    //             .filler(AnyNetwork::recommended_fillers())
    //             .wallet(EthereumWallet::new(wallet.clone()))
    //             .connect_provider(self.provider.clone()),
    //     );

    //     if let Some(max_gas_price) = self.max_gas_price {
    //         let gas_price = self
    //             .provider
    //             .get_gas_price()
    //             .await
    //             .expect("unable to fetch gas price");

    //         if gas_price > max_gas_price {
    //             warn!(%max_gas_price, %gas_price, "gas price is too high");

    //             return Err(TxSubmitError::GasPriceTooHigh {
    //                 max: self.max_gas_price.expect("max gas price is set"),
    //                 price: gas_price,
    //             });
    //         } else {
    //             println!("gas price: {}", gas_price);
    //         }
    //     }

    //     let ucs03_zkgm = zkgm::UCS03Zkgm::new(ucs03_addr_on_evm.into(), signer);

    //     let mut call = ucs03_zkgm.send(
    //         send_call_struct.channelId,
    //         send_call_struct.timeoutHeight,
    //         send_call_struct.timeoutTimestamp,
    //         send_call_struct.salt,
    //         send_call_struct.instruction,
    //     );
    //     println!("submitting evm tx");

    //     let gas_estimate = call.estimate_gas().await.map_err(|e| {
    //         println!("error estimating gas: {:?}", e);
    //         if ErrorReporter(&e)
    //             .to_string()
    //             .contains("gas required exceeds")
    //         {
    //             TxSubmitError::BatchTooLarge
    //         } else {
    //             TxSubmitError::Estimate(e)
    //         }
    //     })?;

    //     let gas_to_use = ((gas_estimate as f64) * self.gas_multiplier) as u64;
    //     println!(
    //         "gas estimate: {gas_estimate}, gas to use: {gas_to_use}, gas multiplier: {}",
    //         self.gas_multiplier
    //     );

    //     if let Some(fixed) = self.fixed_gas_price {
    //         call = call.gas_price(fixed);
    //     }

    //     match call.gas(gas_to_use).send().await {
    //         Ok(ok) => {
    //             let tx_hash = <H256>::from(*ok.tx_hash());
    //             async move {
    //                 let _ = ok.get_receipt().await?;
    //                 println!("tx included: {:?}", tx_hash);

    //                 Ok(tx_hash)
    //             }
    //             .instrument(info_span!("evm tx", %tx_hash))
    //             .await
    //         }

    //         Err(
    //             Error::PendingTransactionError(PendingTransactionError::TransportError(
    //                 TransportError::ErrorResp(e),
    //             ))
    //             | Error::TransportError(TransportError::ErrorResp(e)),
    //         ) if e
    //             .message
    //             .contains("insufficient funds for gas * price + value") =>
    //         {
    //             error!("out of gas");
    //             return Err(TxSubmitError::OutOfGas);
    //         }

    //         Err(
    //             Error::PendingTransactionError(PendingTransactionError::TransportError(
    //                 TransportError::ErrorResp(e),
    //             ))
    //             | Error::TransportError(TransportError::ErrorResp(e)),
    //         ) if e.message.contains("oversized data")
    //             || e.message.contains("exceeds block gas limit")
    //             || e.message.contains("gas required exceeds") =>
    //         {
    //             return Err(TxSubmitError::BatchTooLarge);
    //         }

    //         Err(err) => return Err(TxSubmitError::Error(err)),
    //     }
    // }

    pub async fn get_provider(&self) -> DynProvider<AnyNetwork> {
        let maybe_wallet = self
            .keyring
            .with(|wallet| async move { wallet.clone() })
            .await;
        let wallet = maybe_wallet.expect("no signers available in keyring");

        // 2) Now that we've got the wallet, build your provider in a normal async call.
        //    This future lives *outside* the .with, so UnwindSafe is not required here.
        self.get_provider_with_wallet(&wallet).await
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

    pub async fn deploy_basic_erc721(&self, name: &str, symbol: &str) -> anyhow::Result<H160> {
        const BYTECODE: &str = "0x608060405234801561000f575f5ffd5b506040516111cf3803806111cf83398101604081905261002e916100ef565b81815f61003b83826101d8565b50600161004882826101d8565b5050505050610292565b634e487b7160e01b5f52604160045260245ffd5b5f82601f830112610075575f5ffd5b81516001600160401b0381111561008e5761008e610052565b604051601f8201601f19908116603f011681016001600160401b03811182821017156100bc576100bc610052565b6040528181528382016020018510156100d3575f5ffd5b8160208501602083015e5f918101602001919091529392505050565b5f5f60408385031215610100575f5ffd5b82516001600160401b03811115610115575f5ffd5b61012185828601610066565b602085015190935090506001600160401b0381111561013e575f5ffd5b61014a85828601610066565b9150509250929050565b600181811c9082168061016857607f821691505b60208210810361018657634e487b7160e01b5f52602260045260245ffd5b50919050565b601f8211156101d357805f5260205f20601f840160051c810160208510156101b15750805b601f840160051c820191505b818110156101d0575f81556001016101bd565b50505b505050565b81516001600160401b038111156101f1576101f1610052565b610205816101ff8454610154565b8461018c565b6020601f821160018114610237575f83156102205750848201515b5f19600385901b1c1916600184901b1784556101d0565b5f84815260208120601f198516915b828110156102665787850151825560209485019460019092019101610246565b508482101561028357868401515f19600387901b60f8161c191681555b50505050600190811b01905550565b610f308061029f5f395ff3fe608060405234801561000f575f5ffd5b50600436106100e5575f3560e01c806370a0823111610088578063a22cb46511610063578063a22cb465146101db578063b88d4fde146101ee578063c87b56dd14610201578063e985e9c514610214575f5ffd5b806370a082311461019f57806394bf804d146101c057806395d89b41146101d3575f5ffd5b8063095ea7b3116100c3578063095ea7b31461015157806323b872dd1461016657806342842e0e146101795780636352211e1461018c575f5ffd5b806301ffc9a7146100e957806306fdde0314610111578063081812fc14610126575b5f5ffd5b6100fc6100f7366004610bbc565b610227565b60405190151581526020015b60405180910390f35b610119610278565b6040516101089190610c05565b610139610134366004610c17565b610307565b6040516001600160a01b039091168152602001610108565b61016461015f366004610c49565b61032e565b005b610164610174366004610c71565b61033d565b610164610187366004610c71565b6103cb565b61013961019a366004610c17565b6103ea565b6101b26101ad366004610cab565b6103f4565b604051908152602001610108565b6101646101ce366004610cc4565b610439565b610119610443565b6101646101e9366004610cee565b610452565b6101646101fc366004610d3b565b61045d565b61011961020f366004610c17565b610475565b6100fc610222366004610e18565b6104e6565b5f6001600160e01b031982166380ac58cd60e01b148061025757506001600160e01b03198216635b5e139f60e01b145b8061027257506301ffc9a760e01b6001600160e01b03198316145b92915050565b60605f805461028690610e40565b80601f01602080910402602001604051908101604052809291908181526020018280546102b290610e40565b80156102fd5780601f106102d4576101008083540402835291602001916102fd565b820191905f5260205f20905b8154815290600101906020018083116102e057829003601f168201915b5050505050905090565b5f61031182610513565b505f828152600460205260409020546001600160a01b0316610272565b61033982823361054b565b5050565b6001600160a01b03821661036b57604051633250574960e11b81525f60048201526024015b60405180910390fd5b5f610377838333610558565b9050836001600160a01b0316816001600160a01b0316146103c5576040516364283d7b60e01b81526001600160a01b0380861660048301526024820184905282166044820152606401610362565b50505050565b6103e583838360405180602001604052805f81525061045d565b505050565b5f61027282610513565b5f6001600160a01b03821661041e576040516322718ad960e21b81525f6004820152602401610362565b506001600160a01b03165f9081526003602052604090205490565b610339818361064a565b60606001805461028690610e40565b6103393383836106ab565b61046884848461033d565b6103c53385858585610749565b606061048082610513565b505f61049660408051602081019091525f815290565b90505f8151116104b45760405180602001604052805f8152506104df565b806104be84610871565b6040516020016104cf929190610e8f565b6040516020818303038152906040525b9392505050565b6001600160a01b039182165f90815260056020908152604080832093909416825291909152205460ff1690565b5f818152600260205260408120546001600160a01b03168061027257604051637e27328960e01b815260048101849052602401610362565b6103e58383836001610901565b5f828152600260205260408120546001600160a01b039081169083161561058457610584818486610a05565b6001600160a01b038116156105be5761059f5f855f5f610901565b6001600160a01b0381165f90815260036020526040902080545f190190555b6001600160a01b038516156105ec576001600160a01b0385165f908152600360205260409020805460010190555b5f8481526002602052604080822080546001600160a01b0319166001600160a01b0389811691821790925591518793918516917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef91a4949350505050565b6001600160a01b03821661067357604051633250574960e11b81525f6004820152602401610362565b5f61067f83835f610558565b90506001600160a01b038116156103e5576040516339e3563760e11b81525f6004820152602401610362565b6001600160a01b0382166106dd57604051630b61174360e31b81526001600160a01b0383166004820152602401610362565b6001600160a01b038381165f81815260056020908152604080832094871680845294825291829020805460ff191686151590811790915591519182527f17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31910160405180910390a3505050565b6001600160a01b0383163b1561086a57604051630a85bd0160e11b81526001600160a01b0384169063150b7a029061078b908890889087908790600401610ea3565b6020604051808303815f875af19250505080156107c5575060408051601f3d908101601f191682019092526107c291810190610edf565b60015b61082c573d8080156107f2576040519150601f19603f3d011682016040523d82523d5f602084013e6107f7565b606091505b5080515f0361082457604051633250574960e11b81526001600160a01b0385166004820152602401610362565b805181602001fd5b6001600160e01b03198116630a85bd0160e11b1461086857604051633250574960e11b81526001600160a01b0385166004820152602401610362565b505b5050505050565b60605f61087d83610a69565b60010190505f8167ffffffffffffffff81111561089c5761089c610d27565b6040519080825280601f01601f1916602001820160405280156108c6576020820181803683370190505b5090508181016020015b5f19016f181899199a1a9b1b9c1cb0b131b232b360811b600a86061a8153600a85049450846108d057509392505050565b808061091557506001600160a01b03821615155b156109d6575f61092484610513565b90506001600160a01b038316158015906109505750826001600160a01b0316816001600160a01b031614155b8015610963575061096181846104e6565b155b1561098c5760405163a9fbf51f60e01b81526001600160a01b0384166004820152602401610362565b81156109d45783856001600160a01b0316826001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560405160405180910390a45b505b50505f90815260046020526040902080546001600160a01b0319166001600160a01b0392909216919091179055565b610a10838383610b40565b6103e5576001600160a01b038316610a3e57604051637e27328960e01b815260048101829052602401610362565b60405163177e802f60e01b81526001600160a01b038316600482015260248101829052604401610362565b5f8072184f03e93ff9f4daa797ed6e38ed64bf6a1f0160401b8310610aa75772184f03e93ff9f4daa797ed6e38ed64bf6a1f0160401b830492506040015b6d04ee2d6d415b85acef81000000008310610ad3576d04ee2d6d415b85acef8100000000830492506020015b662386f26fc100008310610af157662386f26fc10000830492506010015b6305f5e1008310610b09576305f5e100830492506008015b6127108310610b1d57612710830492506004015b60648310610b2f576064830492506002015b600a83106102725760010192915050565b5f6001600160a01b03831615801590610b9c5750826001600160a01b0316846001600160a01b03161480610b795750610b7984846104e6565b80610b9c57505f828152600460205260409020546001600160a01b038481169116145b949350505050565b6001600160e01b031981168114610bb9575f5ffd5b50565b5f60208284031215610bcc575f5ffd5b81356104df81610ba4565b5f81518084528060208401602086015e5f602082860101526020601f19601f83011685010191505092915050565b602081525f6104df6020830184610bd7565b5f60208284031215610c27575f5ffd5b5035919050565b80356001600160a01b0381168114610c44575f5ffd5b919050565b5f5f60408385031215610c5a575f5ffd5b610c6383610c2e565b946020939093013593505050565b5f5f5f60608486031215610c83575f5ffd5b610c8c84610c2e565b9250610c9a60208501610c2e565b929592945050506040919091013590565b5f60208284031215610cbb575f5ffd5b6104df82610c2e565b5f5f60408385031215610cd5575f5ffd5b82359150610ce560208401610c2e565b90509250929050565b5f5f60408385031215610cff575f5ffd5b610d0883610c2e565b915060208301358015158114610d1c575f5ffd5b809150509250929050565b634e487b7160e01b5f52604160045260245ffd5b5f5f5f5f60808587031215610d4e575f5ffd5b610d5785610c2e565b9350610d6560208601610c2e565b925060408501359150606085013567ffffffffffffffff811115610d87575f5ffd5b8501601f81018713610d97575f5ffd5b803567ffffffffffffffff811115610db157610db1610d27565b604051601f8201601f19908116603f0116810167ffffffffffffffff81118282101715610de057610de0610d27565b604052818152828201602001891015610df7575f5ffd5b816020840160208301375f6020838301015280935050505092959194509250565b5f5f60408385031215610e29575f5ffd5b610e3283610c2e565b9150610ce560208401610c2e565b600181811c90821680610e5457607f821691505b602082108103610e7257634e487b7160e01b5f52602260045260245ffd5b50919050565b5f81518060208401855e5f93019283525090919050565b5f610b9c610e9d8386610e78565b84610e78565b6001600160a01b03858116825284166020820152604081018390526080606082018190525f90610ed590830184610bd7565b9695505050505050565b5f60208284031215610eef575f5ffd5b81516104df81610ba456fea26469706673582212209c4d699c7cd6d7ef5b15c27f9a3b26c796f4729875ba739af749616ffdb892ba64736f6c634300081e0033";
        let mut code = ethers_hex::decode(BYTECODE.trim_start_matches("0x"))?;
        let initial_supply = "1000000000000000000";
            let encoded = abi::encode(&[
                Token::String(name.into()),
                Token::String(symbol.into()),
            ]);

        code.extend(&encoded);
        let provider = self.get_provider().await;

        let from = self
            .keyring
            .with(|w| async move { w.address() })
            .await
            .unwrap();
        let nonce = provider
            .get_transaction_count(from.into())
            .await?;

        // 3) Build a *deploy* call
        let mut call = RawCallBuilder::new_raw_deploy(
            provider.clone(),     // your DynProvider<AnyNetwork>
            code.into(),          // the bytecode
        ).nonce(nonce);;

        println!("[deploy_basic_erc721] before gas. Nonce: {}", nonce);
        // 4) Estimate gas + buffer
        let gas_est = call.estimate_gas().await?;
        call = call.gas(((gas_est as f64 * 2.2) as u64));
        println!("[deploy_basic_erc721] Estimated gas: {}", gas_est);
        // 5) Send & await receipt
        let pending = call.send().await?;
        println!("[deploy_basic_erc721] pending: {:?}", pending);
        let receipt = pending.get_receipt().await?;
        println!("[deploy_basic_erc721] receipt: {:?}", receipt);

        // 6) Extract the new contract address
        let address = receipt
            .contract_address
            .expect("deploy didn’t return an address");
        Ok(address.into())
    }
        

    pub async fn basic_erc721_mint(
        &self,
        contract: H160,
        token_id: U256,
        recipient: H160,
    ) -> anyhow::Result<H256> {
        let provider = self.get_provider().await;
        let erc = basic_erc721::BasicERC721::new(contract.into(), provider.clone());
        let pending = erc.mint(token_id.into(), recipient.into()).send().await?;
        let tx_hash = <H256>::from(*pending.tx_hash());
        Ok(tx_hash)
    }

    pub async fn zkgmerc20_approve(
        &self,
        contract: H160,
        spender: H160,
        amount: U256,
    ) -> anyhow::Result<H256> {
        let provider = self.get_provider().await;
        let erc = zkgmerc20::ZkgmERC20::new(contract.into(), provider.clone());
        let pending = erc.approve(spender.into(), amount.into()).send().await?;
        let tx_hash = <H256>::from(*pending.tx_hash()); 
        println!("pending: {:?}", pending);

        let receipt = pending.get_receipt().await?;
        println!("PENDING receipt: {:?}", receipt);
        println!("Approved spender: {spender:?} for amount: {amount:?} on contract: {contract:?}");
        Ok(tx_hash)
    }

    pub async fn basic_erc721_approve(
        &self,
        contract: H160,
        spender: H160,
        token_id: U256,
    ) -> anyhow::Result<H256> {
        let provider = self.get_provider().await;
        let erc = basic_erc721::BasicERC721::new(contract.into(), provider.clone());
        let pending = erc.approve(spender.into(), token_id.into()).send().await?;
        let tx_hash = <H256>::from(*pending.tx_hash());
        Ok(tx_hash)
    }

    pub async fn basic_erc721_transfer_from(
        &self,
        contract: H160,
        from: H160,
        to: H160,
        token_id: U256,
    ) -> anyhow::Result<H256> {
        let provider = self.get_provider().await;
        let erc = basic_erc721::BasicERC721::new(contract.into(), provider.clone());
        let pending = erc.transferFrom(from.into(), to.into(), token_id.into()).send().await?;
        let tx_hash = <H256>::from(*pending.tx_hash());
        Ok(tx_hash)
    }

    pub async fn deploy_basic_erc20(&self, spender: H160) -> anyhow::Result<H160> {
        const BYTECODE: &str = "0x608060405234801561000f575f5ffd5b50604051610b91380380610b9183398101604081905261002e916102f0565b6040518060400160405280600481526020016311dbdb1960e21b8152506040518060400160405280600381526020016211d31160ea1b815250816003908161007691906103c1565b50600461008382826103c1565b50505061009633836100a860201b60201c565b6100a13382846100e5565b50506104a0565b6001600160a01b0382166100d65760405163ec442f0560e01b81525f60048201526024015b60405180910390fd5b6100e15f83836100f7565b5050565b6100f2838383600161021d565b505050565b6001600160a01b038316610121578060025f828254610116919061047b565b909155506101919050565b6001600160a01b0383165f90815260208190526040902054818110156101735760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016100cd565b6001600160a01b0384165f9081526020819052604090209082900390555b6001600160a01b0382166101ad576002805482900390556101cb565b6001600160a01b0382165f9081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161021091815260200190565b60405180910390a3505050565b6001600160a01b0384166102465760405163e602df0560e01b81525f60048201526024016100cd565b6001600160a01b03831661026f57604051634a1406b160e11b81525f60048201526024016100cd565b6001600160a01b038085165f90815260016020908152604080832093871683529290522082905580156102ea57826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516102e191815260200190565b60405180910390a35b50505050565b5f5f60408385031215610301575f5ffd5b825160208401519092506001600160a01b038116811461031f575f5ffd5b809150509250929050565b634e487b7160e01b5f52604160045260245ffd5b600181811c9082168061035257607f821691505b60208210810361037057634e487b7160e01b5f52602260045260245ffd5b50919050565b601f8211156100f257805f5260205f20601f840160051c8101602085101561039b5750805b601f840160051c820191505b818110156103ba575f81556001016103a7565b5050505050565b81516001600160401b038111156103da576103da61032a565b6103ee816103e8845461033e565b84610376565b6020601f821160018114610420575f83156104095750848201515b5f19600385901b1c1916600184901b1784556103ba565b5f84815260208120601f198516915b8281101561044f578785015182556020948501946001909201910161042f565b508482101561046c57868401515f19600387901b60f8161c191681555b50505050600190811b01905550565b8082018082111561049a57634e487b7160e01b5f52601160045260245ffd5b92915050565b6106e4806104ad5f395ff3fe608060405234801561000f575f5ffd5b5060043610610090575f3560e01c8063313ce56711610063578063313ce567146100fa57806370a082311461010957806395d89b4114610131578063a9059cbb14610139578063dd62ed3e1461014c575f5ffd5b806306fdde0314610094578063095ea7b3146100b257806318160ddd146100d557806323b872dd146100e7575b5f5ffd5b61009c610184565b6040516100a99190610554565b60405180910390f35b6100c56100c03660046105a4565b610214565b60405190151581526020016100a9565b6002545b6040519081526020016100a9565b6100c56100f53660046105cc565b61022d565b604051601281526020016100a9565b6100d9610117366004610606565b6001600160a01b03165f9081526020819052604090205490565b61009c610250565b6100c56101473660046105a4565b61025f565b6100d961015a366004610626565b6001600160a01b039182165f90815260016020908152604080832093909416825291909152205490565b60606003805461019390610657565b80601f01602080910402602001604051908101604052809291908181526020018280546101bf90610657565b801561020a5780601f106101e15761010080835404028352916020019161020a565b820191905f5260205f20905b8154815290600101906020018083116101ed57829003601f168201915b5050505050905090565b5f3361022181858561026c565b60019150505b92915050565b5f3361023a85828561027e565b6102458585856102ff565b506001949350505050565b60606004805461019390610657565b5f336102218185856102ff565b610279838383600161035c565b505050565b6001600160a01b038381165f908152600160209081526040808320938616835292905220545f198110156102f957818110156102eb57604051637dc7a0d960e11b81526001600160a01b038416600482015260248101829052604481018390526064015b60405180910390fd5b6102f984848484035f61035c565b50505050565b6001600160a01b03831661032857604051634b637e8f60e11b81525f60048201526024016102e2565b6001600160a01b0382166103515760405163ec442f0560e01b81525f60048201526024016102e2565b61027983838361042e565b6001600160a01b0384166103855760405163e602df0560e01b81525f60048201526024016102e2565b6001600160a01b0383166103ae57604051634a1406b160e11b81525f60048201526024016102e2565b6001600160a01b038085165f90815260016020908152604080832093871683529290522082905580156102f957826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161042091815260200190565b60405180910390a350505050565b6001600160a01b038316610458578060025f82825461044d919061068f565b909155506104c89050565b6001600160a01b0383165f90815260208190526040902054818110156104aa5760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016102e2565b6001600160a01b0384165f9081526020819052604090209082900390555b6001600160a01b0382166104e457600280548290039055610502565b6001600160a01b0382165f9081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161054791815260200190565b60405180910390a3505050565b602081525f82518060208401528060208501604085015e5f604082850101526040601f19601f83011684010191505092915050565b80356001600160a01b038116811461059f575f5ffd5b919050565b5f5f604083850312156105b5575f5ffd5b6105be83610589565b946020939093013593505050565b5f5f5f606084860312156105de575f5ffd5b6105e784610589565b92506105f560208501610589565b929592945050506040919091013590565b5f60208284031215610616575f5ffd5b61061f82610589565b9392505050565b5f5f60408385031215610637575f5ffd5b61064083610589565b915061064e60208401610589565b90509250929050565b600181811c9082168061066b57607f821691505b60208210810361068957634e487b7160e01b5f52602260045260245ffd5b50919050565b8082018082111561022757634e487b7160e01b5f52601160045260245ffdfea26469706673582212207cfaf374f2b49d6608fed1bc9f4e741950ec2c77f36bb3d89258c5218167ab4764736f6c634300081e0033"; 
            let mut code = ethers_hex::decode(BYTECODE.trim_start_matches("0x"))?;
            let initial_supply = ethers::types::U256::from_dec_str("1000000000000000000")?;
            let spender = ethers::types::Address::from_str(&spender.to_string())?;
            let encoded = abi::encode(&[
                Token::Uint(initial_supply.into()),
                Token::Address(spender),
            ]);

        code.extend(&encoded);
        let provider = self.get_provider().await;

        let from = self
            .keyring
            .with(|w| async move { w.address() })
            .await
            .unwrap();
        let nonce = provider
            .get_transaction_count(from.into())
            .await?;

        // 3) Build a *deploy* call
        let mut call = RawCallBuilder::new_raw_deploy(
            provider.clone(),     // your DynProvider<AnyNetwork>
            code.into(),          // the bytecode
        ).nonce(nonce);;

        println!("[deploy_basic_erc20] before gas. Nonce: {}", nonce);
        // 4) Estimate gas + buffer
        let gas_est = call.estimate_gas().await?;
        call = call.gas(((gas_est as f64 * 2.2) as u64));
        println!("[deploy_basic_erc20] Estimated gas: {}", gas_est);
        // 5) Send & await receipt
        let pending = call.send().await?;
        println!("[deploy_basic_erc20] pending: {:?}", pending);
        let receipt = pending.get_receipt().await?;
        println!("[deploy_basic_erc20] receipt: {:?}", receipt);

        // 6) Extract the new contract address
        let address = receipt
            .contract_address
            .expect("deploy didn’t return an address");
        Ok(address.into())
    }


    pub async fn send_ibc_transaction(
        &self,
        contract: H160,
        msg: RawCallBuilder<&DynProvider<AnyNetwork>, AnyNetwork>,
    ) -> RpcResult<FixedBytes<32>> {
        let res = self
            .keyring
            .with({
                let msg = msg.clone();
                move |wallet| -> _ {
                    AssertUnwindSafe(self.submit_transaction(contract, wallet, msg))
                }
            })
            .await;

        match res {
            Some(Ok(hash)) => Ok(hash),
            Some(Err(e)) => Err(ErrorObject::owned(
                -1,
                format!("transaction submission failed: {:?}", e),
                None::<()>,
            )),
            None => Err(ErrorObject::owned(-1, "no signers available", None::<()>)),
        }
    }

    pub async fn submit_transaction(
        &self,
        ucs03_addr_on_evm: H160,
        wallet: &LocalSigner<SigningKey>,
        mut call: RawCallBuilder<&DynProvider<AnyNetwork>, AnyNetwork>,
    ) -> Result<H256, TxSubmitError> {
        let signer = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .filler(AnyNetwork::recommended_fillers())
                .wallet(EthereumWallet::new(wallet.clone()))
                .connect_provider(self.provider.clone()),
        );

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
                    // let _ = ok.get_receipt().await?;
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

            Err(Error::PendingTransactionError(
                PendingTransactionError::FailedToRegister
            )) =>
                {
                    println!("Idk whats happening");
                    // let tx_req = call.clone().into_tx_request();
                    return Err(TxSubmitError::PendingTransactionError((PendingTransactionError::FailedToRegister)));

                    // // 1) Build and sign the raw transaction
                    // let tx_req = call.clone().into_tx_request();
                    // let sig = wallet.sign_transaction(&tx_req).await
                    //     .map_err(TxSubmitError::Error)?;
                    // let raw_tx = sig.rlp();

                    // let pending = signer
                    //     .provider()
                    //     .send_raw_transaction(raw_tx.into())
                    //     .await
                    //     .map_err(TxSubmitError::Error)?;

                    // let tx_hash = pending.tx_hash();

                    // // 3) Manual poll loop
                    // loop {
                    //     if let Some(receipt) = self.provider.get_transaction_receipt(tx_hash).await
                    //         .map_err(TxSubmitError::Error)?
                    //     {
                    //         return Ok(tx_hash);
                    //     }
                    //     tokio::time::sleep(Duration::from_secs(1)).await;
                    // }
                }

            

            Err(err) => return Err(TxSubmitError::Error(err)),
        }
    }


    pub async fn predict_stake_manager_address(
        &self,
        zkgm_addr: H160,
    ) -> anyhow::Result<H160> {
        // 1) build a typed alloy client
        let provider = self.get_provider().await;
        let zkgm = zkgm::UCS03Zkgm::new(zkgm_addr.into(), provider.clone());

        // 2) call predictStakeManagerAddress(...)
        let ret = zkgm.predictStakeManagerAddress().call().await?;
        Ok(ret.into())
    }
    
    pub async fn setup_governance_token(
        &self,
        zkgm_addr: H160,
        spender: H160,
        channel_id: u32,
        metadata_image: FixedBytes<32>,
    ) -> anyhow::Result<(GovernanceToken)> {
        // 1) build a typed alloy client
        let provider = self.get_provider().await;
        let zkgm = zkgm::UCS03Zkgm::new(zkgm_addr.into(), provider.clone());
        
        // 3) call registerGovernanceToken(...)
        let pending = zkgm
            .registerGovernanceToken(channel_id, 
                zkgm::GovernanceToken {
                    unwrappedToken: b"muno".into(),
                    metadataImage: metadata_image.into(),
                },)
            .send()
            .await?;
        // wait for inclusion…
        let _receipt = pending.get_receipt().await?;

        // 4) now query back the registered token address
        let ret=
            zkgm.getGovernanceToken(channel_id).call().await?;
        Ok(ret.governanceToken)
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

        contract ZkgmERC721 {
            function mint(uint256 tokenId, address to ) external;

            /// burn an existing NFT
            function burn(uint256 tokenId) external;

            /// `ERC721` standard:
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
    #[error("0x revert")]
    EmptyRevert(Vec<Datagram>),
    #[error("gas price is too high: max {max}, price {price}")]
    GasPriceTooHigh { max: u128, price: u128 },
    #[error("rpc error (this is just the IbcDatagram conversion functions but i need to make those errors better)")]
    RpcError(#[from] ErrorObjectOwned),
    #[error("batch too large")]
    BatchTooLarge,
}

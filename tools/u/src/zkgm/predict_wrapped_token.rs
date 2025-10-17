use alloy::{
    network::AnyNetwork,
    primitives::{Address, keccak256},
    providers::ProviderBuilder,
    sol,
    sol_types::SolValue,
};
use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use deployments::{DEPLOYMENTS, Deployment};
use ibc_union_spec::ChannelId;
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use ucs03_zkgm::msg::{PredictWrappedTokenResponse, QueryMsg};
use unionlabs::primitives::{Bytes, H256, U256};
use voyager_primitives::IbcInterface;

#[derive(Debug, Args)]
pub struct Cmd {
    /// The chain id to predict the wrapped token on, or `custom` to pass custom endpoints.
    chain_id: String,

    /// Provide a custom rpc endpoint to query from.
    ///
    /// This is required if using a custom chain.
    #[arg(long, required_if_eq("chain_id", "custom"))]
    rpc_url: Option<String>,

    /// The address of the ucs03-zkgm implementation to query the wrapped token from.
    #[arg(long, required_if_eq("chain_id", "custom"), help_heading = "Custom")]
    address: Option<String>,

    /// Force usage of the specified interface.
    ///
    /// This can usually be inferred from the address format, but it can be explicitly set with this option.
    #[arg(
        long,
        value_parser(|s: &str| <Result::<_>>::Ok(IbcInterface::new(s.to_owned()))),
        help_heading = "Custom"
    )]
    ibc_interface: Option<IbcInterface>,

    #[command(flatten)]
    args: PredictWrappedTokenArgs,
}

#[derive(Debug, Args)]
pub struct PredictWrappedTokenArgs {
    /// The base token path.
    ///
    /// For tokens that will be wrapped, this is 0. For tokens that will be unwrapped, this is the channel id that the token was originally wrapped over.
    path: U256,

    /// The destination channel id.
    channel_id: ChannelId,

    /// The base token.
    token: String,

    /// Treat `token` as if it were ascii bytes.
    #[arg(long)]
    token_ascii: bool,

    /// The metadata image (hash) to use for the wrapped token.
    #[arg(long, help_heading = "Image")]
    image: Option<H256>,

    /// The implementation for the wrapped token metadata.
    #[arg(
        long,
        // conflicts_with = 
        // required_unless_present("image"),
        conflicts_with("image"),
        help_heading = "Preimage"
    )]
    implementation: Option<String>,

    /// Treat `implementation` as if it were ascii bytes.
    #[arg(long, conflicts_with("image"), help_heading = "Preimage")]
    implementation_ascii: bool,

    /// The initializer for the wrapped token metadata.
    #[arg(
        long,
        // required_unless_present("image"),
        conflicts_with("image"),
        help_heading = "Preimage"
    )]
    initializer: Option<String>,

    /// Treat `initializer` as if it were ascii bytes.
    #[arg(long, conflicts_with("image"), help_heading = "Preimage")]
    initializer_ascii: bool,
}

impl PredictWrappedTokenArgs {
    fn token_bytes(&self) -> Result<Vec<u8>> {
        Ok(if self.token_ascii {
            self.token.clone().into_bytes()
        } else {
            self.token.parse::<Bytes>()?.into_vec()
        })
    }

    fn metadata(&self) -> Result<Metadata> {
        if let Some(image) = self.image {
            Ok(Metadata::Image(image))
        } else {
            let implementation = if self.implementation_ascii {
                self.implementation.clone().unwrap().into_bytes().into()
            } else {
                self.implementation.as_ref().unwrap().parse::<Bytes>()?
            };

            let initializer = if self.initializer_ascii {
                self.initializer.clone().unwrap().into_bytes().into()
            } else {
                self.initializer.as_ref().unwrap().parse::<Bytes>()?
            };

            Ok(Metadata::Preimage {
                implementation,
                initializer,
            })
        }
    }
}

enum Metadata {
    Image(H256),
    Preimage {
        implementation: Bytes,
        initializer: Bytes,
    },
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let Cmd {
            chain_id,
            args,
            rpc_url,
            address,
            ibc_interface,
        } = self;

        if chain_id == "custom" {
            let address = address.unwrap();
            let rpc_url = rpc_url.unwrap();

            match ibc_interface {
                Some(ibc_interface) => match ibc_interface.as_str() {
                    IbcInterface::IBC_SOLIDITY => {
                        predict_wrapped_token_ibc_solidity(
                            rpc_url,
                            address.parse::<Address>()?,
                            args,
                        )
                        .await
                    }
                    IbcInterface::IBC_COSMWASM => {
                        predict_wrapped_token_ibc_cosmwasm(rpc_url, address, args).await
                    }
                    s => bail!("unsupported IBC interface `{s}`"),
                },
                None => {
                    // try to guess the ibc interface based on the address format
                    if address.starts_with("0x") {
                        predict_wrapped_token_ibc_solidity(
                            rpc_url,
                            address.parse::<Address>()?,
                            args,
                        )
                        .await
                    } else {
                        predict_wrapped_token_ibc_cosmwasm(rpc_url, address, args).await
                    }
                }
            }
        } else {
            let chain_id = chain_id.parse().context(
                "invalid chain id, expected either a \
                ucs04 universal chain id or `custom`",
            )?;

            let deployment = &DEPLOYMENTS[&chain_id];

            if ibc_interface.is_some() {
                bail!("--ibc-interface can only be used with `custom`");
            }

            if address.is_some() {
                bail!("--address can only be used with `custom`");
            }

            match deployment {
                Deployment::IbcSolidity { app, .. } => {
                    predict_wrapped_token_ibc_solidity(
                        rpc_url.unwrap_or_else(|| {
                            format!(
                                "https://rpc.{}.{}.chain.kitchen",
                                chain_id.id(),
                                chain_id.family(),
                            )
                        }),
                        app.ucs03
                            .as_ref()
                            .context(anyhow!("no ucs03 deployment for {chain_id}"))?
                            .address
                            .into(),
                        args,
                    )
                    .await
                }
                Deployment::IbcCosmwasm { app, .. } => {
                    predict_wrapped_token_ibc_cosmwasm(
                        rpc_url.unwrap_or_else(|| {
                            format!(
                                "https://rpc.{}.{}.chain.kitchen",
                                chain_id.id(),
                                chain_id.family(),
                            )
                        }),
                        app.ucs03
                            .as_ref()
                            .context(anyhow!("no ucs03 deployment for {chain_id}"))?
                            .address
                            .to_string(),
                        args,
                    )
                    .await
                }
            }
        }
    }
}

async fn predict_wrapped_token_ibc_solidity(
    rpc_url: String,
    address: Address,
    args: PredictWrappedTokenArgs,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .connect(&rpc_url)
        .await?;

    let zkgm = IZkgm::new(address, provider);

    let res = match args.metadata()? {
        Metadata::Image(hash) => {
            zkgm.predictWrappedTokenFromMetadataImageV2(
                args.path.into(),
                args.channel_id.raw(),
                args.token_bytes()?.into(),
                hash.into(),
            )
            .call()
            .await?
            ._0
        }
        Metadata::Preimage {
            implementation,
            initializer,
        } => {
            zkgm.predictWrappedTokenV2(
                args.path.into(),
                args.channel_id.raw(),
                args.token_bytes()?.into(),
                IZkgm::TokenMetadata {
                    implementation: implementation.into(),
                    initializer: initializer.into(),
                },
            )
            .call()
            .await?
            ._0
        }
    };

    println!("{}", res);

    Ok(())
}

async fn predict_wrapped_token_ibc_cosmwasm(
    rpc_url: String,
    address: String,
    args: PredictWrappedTokenArgs,
) -> Result<()> {
    let client = cometbft_rpc::Client::new(rpc_url).await?;

    let res = client
        .grpc_abci_query::<_, QuerySmartContractStateResponse>(
            "/cosmwasm.wasm.v1.Query/SmartContractState",
            &QuerySmartContractStateRequest {
                address,
                query_data: match args.metadata()? {
                    Metadata::Image(metadata_image) => {
                        serde_json::to_vec(&QueryMsg::PredictWrappedTokenV2 {
                            path: args.path.to_string(),
                            channel_id: args.channel_id,
                            token: args.token_bytes()?.into(),
                            metadata_image,
                        })
                        .unwrap()
                    }
                    Metadata::Preimage {
                        implementation,
                        initializer,
                    } => serde_json::to_vec(&QueryMsg::PredictWrappedTokenV2 {
                        path: args.path.to_string(),
                        channel_id: args.channel_id,
                        token: args.token_bytes()?.into(),
                        metadata_image: keccak256(
                            (implementation, initializer).abi_encode_params(),
                        )
                        .into(),
                    })
                    .unwrap(),
                },
            },
            None,
            false,
        )
        .await?
        .into_result()?
        .map(|res| serde_json::from_slice::<PredictWrappedTokenResponse>(&res.data))
        .transpose()?
        .unwrap();

    println!("{}", res.wrapped_token);

    Ok(())
}

sol! {
    #![sol(rpc)]

    interface IZkgm {
        struct TokenMetadata {
            bytes implementation;
            bytes initializer;
        }

        function predictWrappedTokenV2(
            uint256 path,
            uint32 channel,
            bytes calldata token,
            TokenMetadata calldata metadata
        ) external returns (address, bytes32);

        function predictWrappedTokenFromMetadataImageV2(
            uint256 path,
            uint32 channel,
            bytes calldata token,
            bytes32 metadataHash
        ) external returns (address, bytes32);
    }
}

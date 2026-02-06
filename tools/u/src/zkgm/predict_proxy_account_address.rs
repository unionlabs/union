use alloy::hex;
use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use cosmwasm_std::instantiate2_address;
use deployments::Deployment;
use ibc_union_spec::ChannelId;
use ucs03_zkgm::contract::proxy_account_salt;
use unionlabs::primitives::{Bech32, Bytes, H160, U256};
use voyager_primitives::IbcInterface;

use crate::deployments::DEPLOYMENTS;

#[derive(Debug, Args)]
pub struct Cmd {
    /// The chain id to predict the proxy account on, or `custom` to pass custom endpoints.
    chain_id: String,
    /// The address of the ucs03-zkgm implementation to query the wrapped token from.
    #[arg(long, required_if_eq("chain_id", "custom"), help_heading = "Custom")]
    zkgm_address: Option<String>,
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
    args: CallProxySalt,
}

#[derive(Debug, Args)]
pub struct CallProxySalt {
    pub path: U256,
    pub channel_id: ChannelId,
    pub sender: String,
    #[arg(long)]
    pub ascii: bool,
}

impl CallProxySalt {
    fn sender_bytes(&self) -> Result<Vec<u8>> {
        Ok(if self.ascii {
            self.sender.clone().into_bytes()
        } else {
            self.sender.parse::<Bytes>()?.into_vec()
        })
    }
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let Cmd {
            chain_id,
            args,
            zkgm_address,
            ibc_interface,
        } = self;

        if chain_id == "custom" {
            let zkgm_address = zkgm_address.unwrap();

            match ibc_interface {
                Some(ibc_interface) => match ibc_interface.as_str() {
                    IbcInterface::IBC_SOLIDITY => predict_proxy_account_evm(zkgm_address, args),
                    IbcInterface::IBC_COSMWASM => {
                        predict_proxy_account_cosmwasm(zkgm_address, args)
                    }
                    s => bail!("unsupported IBC interface `{s}`"),
                },
                None => {
                    // try to guess the ibc interface based on the address format
                    if zkgm_address.starts_with("0x") {
                        predict_proxy_account_evm(zkgm_address, args)
                    } else {
                        predict_proxy_account_cosmwasm(zkgm_address, args)
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

            if zkgm_address.is_some() {
                bail!("--zkgm-address can only be used with `custom`");
            }

            match deployment {
                Deployment::IbcSolidity { contracts, .. } => predict_proxy_account_evm(
                    contracts
                        .iter()
                        .find(|(_, deployment)| deployment.name == "protocols/ucs03")
                        .as_ref()
                        .context(anyhow!("no ucs03 deployment for {chain_id}"))?
                        .0
                        .to_string(),
                    args,
                ),
                Deployment::IbcCosmwasm { contracts, .. } => predict_proxy_account_cosmwasm(
                    contracts
                        .iter()
                        .find(|(_, deployment)| deployment.name == "protocols/ucs03")
                        .as_ref()
                        .context(anyhow!("no ucs03 deployment for {chain_id}"))?
                        .0
                        .to_string(),
                    args,
                ),
                _ => todo!(),
            }
        }
    }
}

fn predict_proxy_account_cosmwasm(zkgm_address: String, args: CallProxySalt) -> Result<()> {
    const BYTECODE_BASE_CHECKSUM: &[u8] =
        &hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1");

    let salt = proxy_account_salt(&ucs03_zkgm::state::CallProxySalt {
        path: args.path,
        channel_id: args.channel_id,
        sender: args.sender_bytes()?.into(),
    });

    let zkgm_address = zkgm_address.parse::<Bech32>()?;

    let res = instantiate2_address(
        BYTECODE_BASE_CHECKSUM,
        &zkgm_address.data().to_vec().into(),
        salt.iter().as_slice(),
    )?;

    println!("{}", Bech32::new(zkgm_address.hrp(), res.as_slice()));

    Ok(())
}

fn predict_proxy_account_evm(zkgm_address: String, args: CallProxySalt) -> Result<()> {
    let salt = proxy_account_salt(&ucs03_zkgm::state::CallProxySalt {
        path: args.path,
        channel_id: args.channel_id,
        sender: args.sender_bytes()?.into(),
    });

    let zkgm_address = zkgm_address.parse::<H160>()?;

    let res = create3::predict_deterministic_address(zkgm_address.into(), salt.into());

    println!("{res}");

    Ok(())
}

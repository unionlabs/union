//! NOTE: Copied from cosmwasm/deployer

use anyhow::Result;
use cosmos_client::gas::any::GasFiller as AnyGasFiller;

#[derive(Debug, Clone, PartialEq, Default, clap::Args)]
pub struct GasFillerArgs {
    #[arg(long, value_enum, default_value_t = GasFillerType::Fixed)]
    pub gas: GasFillerType,

    // Whether or not to simulate the transactions first.
    #[arg(long, default_value_t = false)]
    pub simulate: bool,

    #[arg(long, help_heading = "Gas filler args", default_value_t = 100_000_000)]
    pub max_gas: u64,
    #[arg(long, help_heading = "Gas filler args", default_value_t = 0)]
    pub min_gas: u64,
    #[arg(
        long,
        help_heading = "Gas filler args",
        required_if_eq("gas", "fixed"),
        default_value_t = 1.0
    )]
    pub gas_multiplier: f64,

    #[arg(
        long,
        help_heading = "--gas fixed",
        required_if_eq("gas", "fixed"),
        default_value_t
    )]
    pub gas_price: f64,
    #[arg(
        long,
        help_heading = "--gas fixed",
        required_if_eq("gas", "fixed"),
        default_value_t
    )]
    pub gas_denom: String,

    /// The denom to use for the feemarket gas token.
    ///
    /// If not set, the Params.FeeDenom value will be used.
    #[arg(
        long,
        help_heading = "--gas feemarket",
        // required_if_eq("gas", "feemarket"),
    )]
    pub fee_denom: Option<String>,

    /// The multiplier to use for the EIP-1559 fee calculation.
    ///
    /// This will be multiplied by the base fee as queried from the chain.
    #[arg(
        long,
        help_heading = "--gas osmosis-eip1559-feemarket",
        required_if_eq("gas", "osmosis_eip1559_feemarket")
    )]
    pub base_fee_multiplier: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default, clap::ValueEnum)]
pub enum GasFillerType {
    #[default]
    Fixed,
    Feemarket,
    OsmosisEip1559Feemarket,
}

pub async fn any_gas_filler_from_args(
    args: GasFillerArgs,
    rpc_url: String,
) -> Result<AnyGasFiller> {
    Ok(match args.gas {
        GasFillerType::Fixed => AnyGasFiller::Fixed(cosmos_client::gas::fixed::GasFiller {
            gas_price: args.gas_price,
            gas_denom: args.gas_denom.clone(),
            gas_multiplier: args.gas_multiplier,
            max_gas: args.max_gas,
            min_gas: args.min_gas,
        }),
        GasFillerType::Feemarket => AnyGasFiller::Feemarket(
            cosmos_client::gas::feemarket::GasFiller::new(cosmos_client::gas::feemarket::Config {
                rpc_url,
                max_gas: args.max_gas,
                gas_multiplier: Some(args.gas_multiplier),
                denom: args.fee_denom,
            })
            .await?,
        ),
        GasFillerType::OsmosisEip1559Feemarket => AnyGasFiller::OsmosisEip1559Feemarket(
            cosmos_client::gas::osmosis_eip1559_feemarket::GasFiller::new(
                cosmos_client::gas::osmosis_eip1559_feemarket::Config {
                    rpc_url,
                    max_gas: args.max_gas,
                    gas_multiplier: Some(args.gas_multiplier),
                    base_fee_multiplier: args.base_fee_multiplier,
                    denom: args.fee_denom,
                },
            )
            .await?,
        ),
    })
}

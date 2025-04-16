use serde::{Deserialize, Serialize};
use unionlabs::cosmos::tx::fee::Fee;

use crate::gas::{feemarket, fixed, osmosis_eip1559_feemarket, GasFillerT};

#[derive(Debug)]
pub enum GasFiller {
    Fixed(fixed::GasFiller),
    Feemarket(feemarket::GasFiller),
    OsmosisEip1559Feemarket(osmosis_eip1559_feemarket::GasFiller),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "config")]
pub enum Config {
    // fixed gas filler is it's own config
    Fixed(fixed::GasFiller),
    Feemarket(feemarket::Config),
    OsmosisEip1559Feemarket(osmosis_eip1559_feemarket::Config),
}

impl GasFillerT for GasFiller {
    async fn max_gas(&self) -> u64 {
        match self {
            Self::Fixed(f) => f.max_gas().await,
            Self::Feemarket(f) => f.max_gas().await,
            GasFiller::OsmosisEip1559Feemarket(f) => f.max_gas().await,
        }
    }

    async fn mk_fee(&self, gas: u64) -> Fee {
        match self {
            Self::Fixed(f) => f.mk_fee(gas).await,
            Self::Feemarket(f) => f.mk_fee(gas).await,
            GasFiller::OsmosisEip1559Feemarket(f) => f.mk_fee(gas).await,
        }
    }
}

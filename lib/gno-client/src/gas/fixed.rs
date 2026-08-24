use gno_rpc::rpc_types::TxFee;
use serde::{Deserialize, Serialize};

use super::GasFillerT;
use crate::gas::u128_saturating_mul_f64;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GasFiller {
    #[serde(with = "::serde_utils::string")]
    pub gas_price: f64,
    pub gas_denom: String,
    #[serde(with = "::serde_utils::string")]
    pub gas_multiplier: f64,
    pub max_gas: u64,
    #[serde(default)]
    pub min_gas: u64,
}

impl GasFillerT for GasFiller {
    async fn max_gas(&self) -> u64 {
        self.max_gas
    }

    async fn mk_fee(&self, gas: u64) -> Result<TxFee, crate::BroadcastTxCommitError> {
        // gas limit = provided gas * multiplier, clamped between min_gas and max_gas
        let gas_limit = u128_saturating_mul_f64(gas.into(), self.gas_multiplier)
            .clamp(self.min_gas.into(), self.max_gas.into());

        let amount = u128_saturating_mul_f64(gas_limit, self.gas_price);

        Ok(TxFee {
            gas_wanted: gas_limit.try_into().unwrap_or(i64::MAX),
            gas_fee: format!("{amount}{}", self.gas_denom),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filler(gas_price: f64, gas_multiplier: f64, min_gas: u64, max_gas: u64) -> GasFiller {
        GasFiller {
            gas_price,
            gas_denom: "ugnot".to_owned(),
            gas_multiplier,
            max_gas,
            min_gas,
        }
    }

    #[tokio::test]
    async fn prices_off_the_post_multiplier_gas_limit_not_the_raw_gas() {
        // regression test: pricing off the pre-multiplier `gas` understated the
        // effective price-per-gas whenever gas_multiplier > 1.0
        let fee = filler(1.0, 2.0, 0, 1_000_000).mk_fee(100).await.unwrap();

        assert_eq!(fee.gas_wanted, 200);
        assert_eq!(fee.gas_fee, "200ugnot");
    }

    #[tokio::test]
    async fn clamps_gas_limit_to_max_gas() {
        let fee = filler(1.0, 10.0, 0, 500).mk_fee(100).await.unwrap();

        assert_eq!(fee.gas_wanted, 500);
        assert_eq!(fee.gas_fee, "500ugnot");
    }

    #[tokio::test]
    async fn clamps_gas_limit_to_min_gas() {
        let fee = filler(1.0, 1.0, 1000, 1_000_000).mk_fee(1).await.unwrap();

        assert_eq!(fee.gas_wanted, 1000);
        assert_eq!(fee.gas_fee, "1000ugnot");
    }
}

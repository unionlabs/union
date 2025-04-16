use serde::{Deserialize, Serialize};
use unionlabs::cosmos::{base::coin::Coin, tx::fee::Fee};

use super::GasFillerT;
use crate::gas::u128_saturating_mul_f64;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GasFiller {
    pub gas_price: f64,
    pub gas_denom: String,
    pub gas_multiplier: f64,
    pub max_gas: u64,
    #[serde(default)]
    pub min_gas: u64,
}

impl GasFillerT for GasFiller {
    async fn max_gas(&self) -> u64 {
        self.max_gas
    }

    async fn mk_fee(&self, gas: u64) -> Fee {
        // gas limit = provided gas * multiplier, clamped between min_gas and max_gas
        let gas_limit = u128_saturating_mul_f64(gas.into(), self.gas_multiplier)
            .clamp(self.min_gas.into(), self.max_gas.into());

        let amount = u128_saturating_mul_f64(gas.into(), self.gas_price);

        Fee {
            amount: vec![Coin {
                amount,
                denom: self.gas_denom.clone(),
            }],
            gas_limit: gas_limit.try_into().unwrap_or(u64::MAX),
            payer: String::new(),
            granter: String::new(),
        }
    }
}

use num_rational::BigRational;
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use unionlabs::cosmos::{base::coin::Coin, tx::fee::Fee};

use crate::gas::{u128_saturating_mul_f64, GasFillerT};

#[derive(Debug)]
pub struct GasFiller {
    max_gas: u64,
    gas_multiplier: f64,
    base_fee_multiplier: f64,
    denom: String,
    client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub max_gas: u64,
    pub gas_multiplier: Option<f64>,
    pub base_fee_multiplier: Option<f64>,
    pub denom: Option<String>,
}

#[allow(clippy::unwrap_used)] // TODO: Better error handling here
impl GasFiller {
    pub async fn new(config: Config) -> Result<Self, cometbft_rpc::JsonRpcError> {
        let client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let denom =
            match config.denom {
                Some(denom) => denom,
                None => client
                    .grpc_abci_query::<_, protos::osmosis::txfees::v1beta1::QueryBaseDenomResponse>(
                        "/osmosis.txfees.v1beta1.Query/BaseDenom",
                        &protos::osmosis::txfees::v1beta1::QueryBaseDenomRequest {},
                        None,
                        false,
                    )
                    .await?
                    .into_result()
                    .unwrap()
                    .unwrap()
                    .base_denom,
            };

        debug!("base denom is {denom}");

        Ok(Self {
            max_gas: config.max_gas,
            gas_multiplier: config.gas_multiplier.unwrap_or(1.0),
            base_fee_multiplier: config.base_fee_multiplier.unwrap_or(1.0),
            denom,
            client,
        })
    }

    pub(crate) async fn get_base_fee(&self) -> BigRational {
        let raw_base_fee = &self
            .client
            .grpc_abci_query::<_, protos::osmosis::txfees::v1beta1::QueryEipBaseFeeResponse>(
                "/osmosis.txfees.v1beta1.Query/GetEipBaseFee",
                &protos::osmosis::txfees::v1beta1::QueryEipBaseFeeRequest {},
                None,
                false,
            )
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .unwrap()
            .base_fee;

        let denominator: u128 = raw_base_fee.parse().unwrap();

        let one = 10_u128.pow(18);

        BigRational::new(denominator.into(), one.into())
    }
}

impl GasFillerT for GasFiller {
    async fn max_gas(&self) -> u64 {
        self.max_gas
    }

    #[instrument(
        skip_all,
        fields(
            self.max_gas = %self.max_gas,
            self.gas_multiplier = %self.gas_multiplier,
            self.base_fee_multiplier = %self.base_fee_multiplier,
            self.denom = %self.denom,
            gas = %gas,
        )
    )]
    async fn mk_fee(&self, gas: u64) -> Fee {
        // gas limit = provided gas * multiplier, clamped between min_gas and max_gas
        let gas_limit = u64::try_from(u128_saturating_mul_f64(gas.into(), self.gas_multiplier))
            .unwrap_or(self.max_gas)
            .min(self.max_gas);

        let base_fee = self.get_base_fee().await;

        let gas_price = BigRational::from_f64(self.base_fee_multiplier)
            .expect("base fee multiplier is rational")
            * base_fee.clone();

        debug!(
            base_fee = %base_fee.to_f64().expect("should be a valid f64"),
            gas_price = %gas_price.to_f64().expect("should be a valid f64"),
            "gas price"
        );

        let amount = BigRational::from_integer(gas_limit.into()) * gas_price;

        let amount = amount.to_integer().try_into().unwrap_or(u128::MAX);

        debug!(amount, "fee");

        Fee {
            amount: vec![Coin {
                amount,
                denom: self.denom.clone(),
            }],
            gas_limit,
            payer: String::new(),
            granter: String::new(),
        }
    }
}

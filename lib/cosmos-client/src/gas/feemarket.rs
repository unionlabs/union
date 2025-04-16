use num_rational::BigRational;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use unionlabs::cosmos::{base::coin::Coin, tx::fee::Fee};

use crate::gas::{u128_saturating_mul_f64, GasFillerT};

#[derive(Debug)]
pub struct GasFiller {
    max_gas: u64,
    gas_multiplier: f64,
    denom: String,
    client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub max_gas: u64,
    pub gas_multiplier: Option<f64>,
    pub denom: Option<String>,
}

#[allow(clippy::unwrap_used)] // TODO: Better error handling here
impl GasFiller {
    pub async fn new(config: Config) -> Result<Self, cometbft_rpc::JsonRpcError> {
        let client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let denom = match config.denom {
            Some(denom) => denom,
            None => {
                client
                    .grpc_abci_query::<_, protos::feemarket::feemarket::v1::ParamsResponse>(
                        "/feemarket.feemarket.v1.Query/Params",
                        &protos::feemarket::feemarket::v1::ParamsRequest {},
                        None,
                        false,
                    )
                    .await?
                    .into_result()
                    .unwrap()
                    .unwrap()
                    .params
                    .unwrap()
                    .fee_denom
            }
        };

        Ok(Self {
            max_gas: config.max_gas,
            gas_multiplier: config.gas_multiplier.unwrap_or(1.0),
            denom,
            client,
        })
    }

    pub(crate) async fn get_gas_price(&self) -> Coin {
        let response = self
            .client
            .grpc_abci_query::<_, protos::feemarket::feemarket::v1::GasPriceResponse>(
                "/feemarket.feemarket.v1.Query/GasPrice",
                &protos::feemarket::feemarket::v1::GasPriceRequest {
                    denom: self.denom.clone(),
                },
                None,
                false,
            )
            .await
            .unwrap()
            .into_result()
            .unwrap()
            .unwrap()
            .price
            .unwrap();

        Coin {
            denom: response.denom,
            amount: response.amount.parse().unwrap(),
        }
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
            self.denom = %self.denom,
            gas = %gas,
        )
    )]
    async fn mk_fee(&self, gas: u64) -> Fee {
        // gas limit = provided gas * multiplier, clamped between min_gas and max_gas
        let gas_limit = u64::try_from(u128_saturating_mul_f64(gas.into(), self.gas_multiplier))
            .unwrap_or(self.max_gas)
            .min(self.max_gas);

        let coin = self.get_gas_price().await;

        let one = 10_u128.pow(18);
        let gas_price = BigRational::new(coin.amount.into(), one.into());

        debug!(
            price = gas_price.to_f64(),
            raw_price = coin.amount,
            "gas price"
        );

        let amount = gas_price * BigRational::from_integer(gas_limit.into());

        let amount = amount.to_integer().try_into().unwrap_or(u128::MAX);

        debug!(amount, "fee");

        Fee {
            amount: vec![Coin {
                amount,
                denom: coin.denom,
            }],
            gas_limit,
            payer: String::new(),
            granter: String::new(),
        }
    }
}

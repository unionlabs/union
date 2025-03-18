use num_rational::BigRational;
use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use tracing::debug;
use unionlabs::cosmos::{base::coin::Coin, tx::fee::Fee};

pub trait GasFillerT {
    async fn max_gas(&self) -> u64;

    async fn mk_fee(&self, gas: u64) -> Fee;
}

impl<T: GasFillerT> GasFillerT for &T {
    async fn max_gas(&self) -> u64 {
        (*self).max_gas().await
    }

    async fn mk_fee(&self, gas: u64) -> Fee {
        (*self).mk_fee(gas).await
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StaticGasFiller {
    pub gas_price: f64,
    pub gas_denom: String,
    pub gas_multiplier: f64,
    pub max_gas: u64,
    #[serde(default)]
    pub min_gas: u64,
}

impl GasFillerT for StaticGasFiller {
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

fn u128_saturating_mul_f64(u: u128, f: f64) -> u128 {
    (BigRational::from_integer(u.into()) * BigRational::from_float(f).expect("finite"))
        .to_integer()
        .try_into()
        .unwrap_or(u128::MAX)
    // .expect("overflow")
}

#[derive(Debug)]
pub struct FeemarketGasFiller {
    max_gas: u64,
    gas_multiplier: Option<f64>,
    denom: String,
    client: cometbft_rpc::Client,
}

#[allow(clippy::unwrap_used)] // TODO: Better error handling here
impl FeemarketGasFiller {
    pub async fn new(
        rpc_url: String,
        max_gas: u64,
        gas_multiplier: Option<f64>,
        denom: Option<String>,
    ) -> Result<Self, cometbft_rpc::JsonRpcError> {
        let client = cometbft_rpc::Client::new(rpc_url).await?;

        let denom = match denom {
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
            max_gas,
            gas_multiplier,
            denom,
            client,
        })
    }

    async fn get_gas_price(&self) -> Coin {
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

impl GasFillerT for FeemarketGasFiller {
    async fn max_gas(&self) -> u64 {
        self.max_gas
    }

    async fn mk_fee(&self, gas: u64) -> Fee {
        // gas limit = provided gas * multiplier, clamped between min_gas and max_gas
        let gas = u64::try_from(u128_saturating_mul_f64(
            gas.into(),
            self.gas_multiplier.unwrap_or(1.0),
        ))
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

        let amount = gas_price * BigRational::from_integer(gas.into());

        let amount = amount.to_integer().try_into().unwrap_or(u128::MAX);

        debug!(amount, "fee");

        Fee {
            amount: vec![Coin {
                amount,
                denom: coin.denom,
            }],
            gas_limit: gas,
            payer: String::new(),
            granter: String::new(),
        }
    }
}

use gno_rpc::rpc_types::TxFee;
use num_rational::BigRational;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::gas::{GasFillerT, u128_saturating_mul_f64};

#[derive(Debug, Clone)]
pub struct GasFiller {
    max_gas: u64,
    gas_multiplier: f64,
    denom_override: Option<String>,
    client: gno_rpc::Client,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub max_gas: u64,
    #[serde(with = "::serde_utils::string_opt")]
    pub gas_multiplier: Option<f64>,
    pub denom: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GasPriceResponse {
    #[serde(with = "::serde_utils::string")]
    gas: i64,
    price: String,
}

/// gno's `GasPrice{ Gas, Price }`; price per unit of gas is `amount / gas`, not `amount` alone.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GasPrice {
    pub gas: i64,
    pub amount: u128,
    pub denom: String,
}

impl GasFiller {
    pub fn new(config: Config, client: gno_rpc::Client) -> Self {
        Self {
            max_gas: config.max_gas,
            gas_multiplier: config.gas_multiplier.unwrap_or(1.0),
            denom_override: config.denom,
            client,
        }
    }

    pub(crate) async fn get_gas_price(&self) -> Result<GasPrice, crate::BroadcastTxCommitError> {
        let response = self
            .client
            .abci_query("auth/gasprice", &[], None, false)
            .await?;

        if let Some(error) = response.response.response_base.error {
            return Err(crate::BroadcastTxCommitError::TxFailed {
                error,
                log: response.response.response_base.log,
            });
        }

        let value = response.response.response_base.data.unwrap_or_default();

        parse_gas_price_response(&value)
    }
}

fn parse_gas_price_response(value: &[u8]) -> Result<GasPrice, crate::BroadcastTxCommitError> {
    let gas_price = serde_json::from_slice::<GasPriceResponse>(value)?;

    if gas_price.price.is_empty() {
        return Ok(GasPrice {
            gas: gas_price.gas,
            amount: 0,
            denom: String::new(),
        });
    }

    let split_at = gas_price
        .price
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(gas_price.price.len());

    let (amount, denom) = gas_price.price.split_at(split_at);

    let amount: u128 = amount
        .parse()
        .map_err(|_| crate::BroadcastTxCommitError::InvalidGasPrice {
            price: gas_price.price.clone(),
        })?;

    if amount > 0 && gas_price.gas <= 0 {
        return Err(crate::BroadcastTxCommitError::InvalidGasPrice {
            price: gas_price.price,
        });
    }

    Ok(GasPrice {
        gas: gas_price.gas,
        amount,
        denom: denom.to_owned(),
    })
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
            gas = %gas,
        )
    )]
    async fn mk_fee(&self, gas: u64) -> Result<TxFee, crate::BroadcastTxCommitError> {
        // gas limit = provided gas * multiplier, clamped to max_gas
        let gas_limit = u128_saturating_mul_f64(gas.into(), self.gas_multiplier)
            .try_into()
            .unwrap_or(self.max_gas)
            .min(self.max_gas);

        let price = self.get_gas_price().await?;

        let fee = compute_fee(gas_limit, &price, self.denom_override.as_deref())?;

        debug!(gas_limit, gas_fee = %fee.gas_fee, "computed fee from dynamic gas price");

        Ok(fee)
    }
}

fn compute_fee(
    gas_limit: u64,
    price: &GasPrice,
    denom_override: Option<&str>,
) -> Result<TxFee, crate::BroadcastTxCommitError> {
    let denom = if price.denom.is_empty() {
        denom_override.ok_or(crate::BroadcastTxCommitError::MissingGasDenom)?
    } else {
        price.denom.as_str()
    };

    let price_per_gas = BigRational::new(price.amount.into(), price.gas.max(1).into());

    let amount = price_per_gas * BigRational::from_integer(gas_limit.into());
    let amount = amount.ceil().to_integer().try_into().unwrap_or(u128::MAX);

    Ok(TxFee {
        gas_wanted: gas_limit.try_into().unwrap_or(i64::MAX),
        gas_fee: format!("{amount}{denom}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_normal_price() {
        let price = parse_gas_price_response(br#"{"gas": "1000", "price": "5ugnot"}"#).unwrap();

        assert_eq!(
            price,
            GasPrice {
                gas: 1000,
                amount: 5,
                denom: "ugnot".to_owned()
            }
        );
    }

    #[test]
    fn parses_zero_price_as_no_price_set() {
        let price = parse_gas_price_response(br#"{"gas": "0", "price": ""}"#).unwrap();

        assert_eq!(
            price,
            GasPrice {
                gas: 0,
                amount: 0,
                denom: String::new()
            }
        );
    }

    #[test]
    fn rejects_malformed_price() {
        let err = parse_gas_price_response(br#"{"gas": "1000", "price": "abcugnot"}"#).unwrap_err();

        assert!(matches!(
            err,
            crate::BroadcastTxCommitError::InvalidGasPrice { price } if price == "abcugnot"
        ));
    }

    #[test]
    fn rejects_nonzero_price_with_non_positive_gas() {
        let err = parse_gas_price_response(br#"{"gas": "0", "price": "5ugnot"}"#).unwrap_err();

        assert!(matches!(
            err,
            crate::BroadcastTxCommitError::InvalidGasPrice { price } if price == "5ugnot"
        ));

        let err = parse_gas_price_response(br#"{"gas": "-1", "price": "5ugnot"}"#).unwrap_err();

        assert!(matches!(
            err,
            crate::BroadcastTxCommitError::InvalidGasPrice { price } if price == "5ugnot"
        ));
    }

    fn gas_price(gas: i64, amount: u128, denom: &str) -> GasPrice {
        GasPrice {
            gas,
            amount,
            denom: denom.to_owned(),
        }
    }

    #[test]
    fn compute_fee_prices_gas_limit_at_reported_price() {
        let fee = compute_fee(1000, &gas_price(1000, 5, "ugnot"), None).unwrap();

        assert_eq!(fee.gas_wanted, 1000);
        assert_eq!(fee.gas_fee, "5ugnot");
    }

    #[test]
    fn compute_fee_never_overrides_a_denom_the_chain_reported() {
        let fee = compute_fee(1000, &gas_price(1000, 5, "ugnot"), Some("uatom")).unwrap();

        assert_eq!(fee.gas_fee, "5ugnot");
    }

    #[test]
    fn compute_fee_falls_back_to_override_when_chain_reports_no_denom() {
        let fee = compute_fee(1000, &gas_price(0, 0, ""), Some("ugnot")).unwrap();

        assert_eq!(fee.gas_fee, "0ugnot");
    }

    #[test]
    fn compute_fee_errors_instead_of_producing_a_denomless_fee() {
        let err = compute_fee(1000, &gas_price(0, 0, ""), None).unwrap_err();

        assert!(matches!(
            err,
            crate::BroadcastTxCommitError::MissingGasDenom
        ));
    }

    #[tokio::test]
    #[ignore = "hits a live gno node"]
    async fn get_gas_price_against_live_node() {
        let client = gno_rpc::Client::new("https://sapphire.rpc.onbloc.xyz:443")
            .await
            .unwrap();

        let filler = GasFiller::new(
            Config {
                max_gas: 1_000_000_000,
                gas_multiplier: None,
                denom: None,
            },
            client,
        );

        let price = filler.get_gas_price().await.unwrap();

        println!("{price:?}");

        assert!(price.gas > 0);
        assert!(!price.denom.is_empty());
    }
}

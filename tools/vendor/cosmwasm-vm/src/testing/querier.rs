use serde::de::DeserializeOwned;

use cosmwasm_std::testing::{MockQuerier as StdMockQuerier, MockQuerierCustomHandlerResult};
use cosmwasm_std::{
    to_binary, to_vec, Binary, Coin, ContractResult, CustomQuery, Empty, Querier as _,
    QueryRequest, SystemError, SystemResult,
};

use crate::{BackendError, BackendResult, GasInfo, Querier};

const GAS_COST_QUERY_FLAT: u64 = 100_000;
/// Gas per request byte
const GAS_COST_QUERY_REQUEST_MULTIPLIER: u64 = 0;
/// Gas per reponse byte
const GAS_COST_QUERY_RESPONSE_MULTIPLIER: u64 = 100;

/// MockQuerier holds an immutable table of bank balances
pub struct MockQuerier<C: CustomQuery + DeserializeOwned = Empty> {
    querier: StdMockQuerier<C>,
}

impl<C: CustomQuery + DeserializeOwned> MockQuerier<C> {
    pub fn new(balances: &[(&str, &[Coin])]) -> Self {
        MockQuerier {
            querier: StdMockQuerier::new(balances),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance(
        &mut self,
        addr: impl Into<String>,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.querier.update_balance(addr, balance)
    }

    #[cfg(feature = "staking")]
    pub fn update_staking(
        &mut self,
        denom: &str,
        validators: &[cosmwasm_std::Validator],
        delegations: &[cosmwasm_std::FullDelegation],
    ) {
        self.querier.update_staking(denom, validators, delegations);
    }

    pub fn update_wasm<WH: 'static>(&mut self, handler: WH)
    where
        WH: Fn(&cosmwasm_std::WasmQuery) -> cosmwasm_std::QuerierResult,
    {
        self.querier.update_wasm(handler)
    }

    pub fn with_custom_handler<CH: 'static>(mut self, handler: CH) -> Self
    where
        CH: Fn(&C) -> MockQuerierCustomHandlerResult,
    {
        self.querier = self.querier.with_custom_handler(handler);
        self
    }
}

impl<C: CustomQuery + DeserializeOwned> Querier for MockQuerier<C> {
    fn query_raw(
        &self,
        bin_request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        let response = self.querier.raw_query(bin_request);
        let gas_info = GasInfo::with_externally_used(
            GAS_COST_QUERY_FLAT
                + (GAS_COST_QUERY_REQUEST_MULTIPLIER * (bin_request.len() as u64))
                + (GAS_COST_QUERY_RESPONSE_MULTIPLIER
                    * (to_binary(&response).unwrap().len() as u64)),
        );

        // In a production implementation, this should stop the query execution in the middle of the computation.
        // Thus no query response is returned to the caller.
        if gas_info.externally_used > gas_limit {
            return (Err(BackendError::out_of_gas()), gas_info);
        }

        // We don't use FFI in the mock implementation, so BackendResult is always Ok() regardless of error on other levels
        (Ok(response), gas_info)
    }
}

impl MockQuerier {
    pub fn query<C: CustomQuery>(
        &self,
        request: &QueryRequest<C>,
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        // encode the request, then call raw_query
        let request_binary = match to_vec(request) {
            Ok(raw) => raw,
            Err(err) => {
                let gas_info = GasInfo::with_externally_used(err.to_string().len() as u64);
                return (
                    Ok(SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Serializing query request: {}", err),
                        request: b"N/A".into(),
                    })),
                    gas_info,
                );
            }
        };
        self.query_raw(&request_binary, gas_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coin, from_binary, AllBalanceResponse, BalanceResponse, BankQuery, Empty};

    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;

    #[test]
    fn query_raw_fails_when_out_of_gas() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let querier: MockQuerier<Empty> = MockQuerier::new(&[(&addr, &balance)]);

        let gas_limit = 20;
        let (result, _gas_info) = querier.query_raw(b"broken request", gas_limit);
        match result.unwrap_err() {
            BackendError::OutOfGas {} => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn bank_querier_all_balances() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let querier = MockQuerier::new(&[(&addr, &balance)]);

        // all
        let all = querier
            .query::<Empty>(
                &BankQuery::AllBalances { address: addr }.into(),
                DEFAULT_QUERY_GAS_LIMIT,
            )
            .0
            .unwrap()
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_binary(&all).unwrap();
        assert_eq!(&res.amount, &balance);
    }

    #[test]
    fn bank_querier_one_balance() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let querier = MockQuerier::new(&[(&addr, &balance)]);

        // one match
        let fly = querier
            .query::<Empty>(
                &BankQuery::Balance {
                    address: addr.clone(),
                    denom: "FLY".to_string(),
                }
                .into(),
                DEFAULT_QUERY_GAS_LIMIT,
            )
            .0
            .unwrap()
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&fly).unwrap();
        assert_eq!(res.amount, coin(777, "FLY"));

        // missing denom
        let miss = querier
            .query::<Empty>(
                &BankQuery::Balance {
                    address: addr,
                    denom: "MISS".to_string(),
                }
                .into(),
                DEFAULT_QUERY_GAS_LIMIT,
            )
            .0
            .unwrap()
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&miss).unwrap();
        assert_eq!(res.amount, coin(0, "MISS"));
    }

    #[test]
    fn bank_querier_missing_account() {
        let addr = String::from("foobar");
        let balance = vec![coin(123, "ELF"), coin(777, "FLY")];
        let querier = MockQuerier::new(&[(&addr, &balance)]);

        // all balances on empty account is empty vec
        let all = querier
            .query::<Empty>(
                &BankQuery::AllBalances {
                    address: String::from("elsewhere"),
                }
                .into(),
                DEFAULT_QUERY_GAS_LIMIT,
            )
            .0
            .unwrap()
            .unwrap()
            .unwrap();
        let res: AllBalanceResponse = from_binary(&all).unwrap();
        assert_eq!(res.amount, vec![]);

        // any denom on balances on empty account is empty coin
        let miss = querier
            .query::<Empty>(
                &BankQuery::Balance {
                    address: String::from("elsewhere"),
                    denom: "ELF".to_string(),
                }
                .into(),
                DEFAULT_QUERY_GAS_LIMIT,
            )
            .0
            .unwrap()
            .unwrap()
            .unwrap();
        let res: BalanceResponse = from_binary(&miss).unwrap();
        assert_eq!(res.amount, coin(0, "ELF"));
    }
}

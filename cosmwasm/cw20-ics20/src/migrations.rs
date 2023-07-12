// v1 format is anything older than 0.12.0
pub mod v1 {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::Addr;
    use cw_storage_plus::Item;

    #[cw_serde]
    pub struct Config {
        pub default_timeout: u64,
        pub gov_contract: Addr,
    }

    pub const CONFIG: Item<Config> = Item::new("ics20_config");
}

// v2 format is anything older than 0.13.1 when we only updated the internal balances on success ack
pub mod v2 {
    use cosmwasm_std::{to_binary, Addr, DepsMut, Env, Order, StdResult, WasmQuery};
    use cw20::{BalanceResponse, Cw20QueryMsg};

    use crate::{
        amount::Amount,
        state::{ChannelState, CHANNEL_INFO, CHANNEL_STATE},
        ContractError,
    };

    pub fn update_balances(mut deps: DepsMut, env: &Env) -> Result<(), ContractError> {
        let channels = CHANNEL_INFO
            .keys(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        match channels.len() {
            0 => Ok(()),
            1 => {
                let channel = &channels[0];
                let addr = &env.contract.address;
                let states = CHANNEL_STATE
                    .prefix(channel)
                    .range(deps.storage, None, None, Order::Ascending)
                    .collect::<StdResult<Vec<_>>>()?;
                for (denom, state) in states.into_iter() {
                    update_denom(deps.branch(), addr, channel, denom, state)?;
                }
                Ok(())
            }
            _ => Err(ContractError::CannotMigrate {
                previous_contract: "multiple channels open".into(),
            }),
        }
    }

    fn update_denom(
        deps: DepsMut,
        contract: &Addr,
        channel: &str,
        denom: String,
        mut state: ChannelState,
    ) -> StdResult<()> {
        // handle this for both native and cw20
        let balance = match Amount::from_parts(denom.clone(), state.outstanding) {
            Amount::Native(coin) => deps.querier.query_balance(contract, coin.denom)?.amount,
            Amount::Cw20(coin) => {
                // FIXME: we should be able to do this with the following line, but QuerierWrapper doesn't play
                // with the Querier generics
                // `Cw20Contract(contract.clone()).balance(&deps.querier, contract)?`
                let query = WasmQuery::Smart {
                    contract_addr: coin.address,
                    msg: to_binary(&Cw20QueryMsg::Balance {
                        address: contract.into(),
                    })?,
                };
                let res: BalanceResponse = deps.querier.query(&query.into())?;
                res.balance
            }
        };

        // this checks if we have received some coins that are "in flight" and not yet accounted in the state
        let diff = balance - state.outstanding;
        // if they are in flight, we add them to the internal state now, as if we added them when sent (not when acked)
        // to match the current logic
        if !diff.is_zero() {
            state.outstanding += diff;
            state.total_sent += diff;
            CHANNEL_STATE.save(deps.storage, (channel, &denom), &state)?;
        }

        Ok(())
    }
}

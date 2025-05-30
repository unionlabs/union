use alloy_primitives::U256;
use cosmwasm_std::{Addr, Uint128};

pub const FLAG_ALLOW_FAILURE: U256 = U256::from_be_slice(&[1]);
pub const FLAG_ALLOW_MARKET_MAKER: U256 = U256::from_be_slice(&[2]);

alloy_sol_types::sol! {
    #[derive(Debug)]
    struct FundedDispatchFund {
        bytes token;
        uint256 amount;
    }

    #[derive(Debug)]
    struct FundedDispatchParameters {
        uint256 flags;
        FundedDispatchFund[] funds;
        bytes contract_address;
        bytes contract_calldata;
        bytes beneficiary;
    }
}

fn has_flag(value: U256, flag: U256) -> bool {
    (value & flag) == flag
}

impl FundedDispatchParameters {
    pub fn allow_failure(&self) -> bool {
        has_flag(self.flags, FLAG_ALLOW_FAILURE)
    }

    pub fn allow_market_maker(&self) -> bool {
        has_flag(self.flags, FLAG_ALLOW_MARKET_MAKER)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum StoredFundedDispatchFund {
    Native { denom: String, amount: Uint128 },
    Cw20 { address: String, amount: Uint128 },
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StoredFundedDispatchParameters {
    pub allow_failure: bool,
    pub beneficiary: Addr,
    pub contract: Addr,
    pub funds: Vec<StoredFundedDispatchFund>,
}

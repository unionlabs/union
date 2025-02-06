use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{StdError, StdResult, Uint128};
pub use cw20::Cw20ExecuteMsg as ExecuteMsg;
use cw20::{Cw20Coin, Logo, MinterResponse};

#[cw_serde]
pub struct InstantiateMarketingInfo {
    pub project: Option<String>,
    pub description: Option<String>,
    pub marketing: Option<String>,
    pub logo: Option<Logo>,
}

#[cw_serde]
#[cfg_attr(test, derive(Default))]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub marketing: Option<InstantiateMarketingInfo>,
}

impl InstantiateMsg {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }

    pub fn validate(&self) -> StdResult<()> {
        // Check name, symbol, decimals
        if !self.has_valid_name() {
            return Err(StdError::generic_err(
                "Name is not in the expected format (3-50 UTF-8 bytes)",
            ));
        }
        if !self.has_valid_symbol() {
            return Err(StdError::generic_err(
                "Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}",
            ));
        }
        if self.decimals > 18 {
            return Err(StdError::generic_err("Decimals must not exceed 18"));
        }
        Ok(())
    }

    fn has_valid_name(&self) -> bool {
        let bytes = self.name.as_bytes();
        if bytes.len() < 3 || bytes.len() > 50 {
            return false;
        }
        true
    }

    fn has_valid_symbol(&self) -> bool {
        let bytes = self.symbol.as_bytes();
        if bytes.len() < 3 || bytes.len() > 12 {
            return false;
        }
        for byte in bytes.iter() {
            if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
                return false;
            }
        }
        true
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    /// Only with "mintable" extension.
    /// Returns who can mint and the hard cap on maximum tokens after minting.
    #[returns(cw20::MinterResponse)]
    Minter {},
    /// Only with "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    #[returns(cw20::AllowanceResponse)]
    Allowance { owner: String, spender: String },
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this owner has approved. Supports pagination.
    #[returns(cw20::AllAllowancesResponse)]
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this spender has been granted. Supports pagination.
    #[returns(cw20::AllSpenderAllowancesResponse)]
    AllSpenderAllowances {
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "enumerable" extension
    /// Returns all accounts that have balances. Supports pagination.
    #[returns(cw20::AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "marketing" extension
    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    #[returns(cw20::MarketingInfoResponse)]
    MarketingInfo {},
    /// Only with "marketing" extension
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data is stored for this
    /// contract.
    #[returns(cw20::DownloadLogoResponse)]
    DownloadLogo {},
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_instantiatemsg_name() {
        // Too short
        let mut msg = InstantiateMsg {
            name: str::repeat("a", 2),
            ..InstantiateMsg::default()
        };
        assert!(!msg.has_valid_name());

        // In the correct length range
        msg.name = str::repeat("a", 3);
        assert!(msg.has_valid_name());

        // Too long
        msg.name = str::repeat("a", 51);
        assert!(!msg.has_valid_name());
    }

    #[test]
    fn validate_instantiatemsg_symbol() {
        // Too short
        let mut msg = InstantiateMsg {
            symbol: str::repeat("a", 2),
            ..InstantiateMsg::default()
        };
        assert!(!msg.has_valid_symbol());

        // In the correct length range
        msg.symbol = str::repeat("a", 3);
        assert!(msg.has_valid_symbol());

        // Too long
        msg.symbol = str::repeat("a", 13);
        assert!(!msg.has_valid_symbol());

        // Has illegal char
        let illegal_chars = [[64u8], [91u8], [123u8]];
        illegal_chars.iter().for_each(|c| {
            let c = std::str::from_utf8(c).unwrap();
            msg.symbol = str::repeat(c, 3);
            assert!(!msg.has_valid_symbol());
        });
    }
}

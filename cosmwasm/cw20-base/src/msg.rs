use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{StdError, Uint128};
use cw20::{Cw20Coin, Cw20ExecuteMsg, Logo, MinterResponse};

use crate::ContractError;

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMetadata {
        name: String,
        symbol: String,
        decimals: u8,
    },
    #[serde(untagged)]
    Cw20ExecuteMsg(Cw20ExecuteMsg),
}

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

pub fn validate_name(name: &str) -> Result<(), ContractError> {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 50 {
        return Err(
            StdError::generic_err("Name is not in the expected format (3-50 UTF-8 bytes)").into(),
        );
    }
    Ok(())
}

pub fn validate_symbol(symbol: &str) -> Result<(), ContractError> {
    let bytes = symbol.as_bytes();
    if bytes.len() < 3 || bytes.len() > 12 {
        return Err(
            StdError::generic_err("Ticker symbol is in expected format [a-zA-Z\\-]{3,12}").into(),
        );
    }
    for byte in bytes.iter() {
        if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
            return Err(StdError::generic_err(
                "Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}",
            )
            .into());
        }
    }
    Ok(())
}

pub fn validate_decimals(decimals: u8) -> Result<(), ContractError> {
    if decimals > 18 {
        return Err(StdError::generic_err("Decimals must not exceed 18").into());
    }
    Ok(())
}

impl InstantiateMsg {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        // Check name, symbol, decimals
        validate_name(&self.name)?;

        validate_symbol(&self.symbol)?;

        validate_decimals(self.decimals)?;

        Ok(())
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
    fn cw20_untagged_message_correctly_serialized() {
        let execute = Cw20ExecuteMsg::Transfer {
            recipient: "hello".into(),
            amount: 10u128.into(),
        };

        let serialized = cosmwasm_std::to_json_string(&execute).unwrap();

        assert_eq!(
            cosmwasm_std::to_json_string(&ExecuteMsg::Cw20ExecuteMsg(execute.clone())).unwrap(),
            serialized
        );

        let transfer: ExecuteMsg = cosmwasm_std::from_json(&serialized).unwrap();

        assert_eq!(transfer, ExecuteMsg::Cw20ExecuteMsg(execute));
    }

    #[test]
    fn validate_instantiatemsg_name() {
        // Too short
        assert!(validate_name(&str::repeat("a", 2)).is_err());

        // In the correct length range
        assert!(validate_name(&str::repeat("a", 3)).is_ok());

        // Too long
        assert!(validate_name(&str::repeat("a", 51)).is_err());
    }

    #[test]
    fn validate_instantiatemsg_symbol() {
        // Too short
        assert!(validate_symbol(&str::repeat("a", 2)).is_err());

        // In the correct length range
        assert!(validate_symbol(&str::repeat("a", 3)).is_ok());

        // Too long
        assert!(validate_symbol(&str::repeat("a", 13)).is_err());

        // Has illegal char
        let illegal_chars = [[64u8], [91u8], [123u8]];
        illegal_chars.iter().for_each(|c| {
            let c = std::str::from_utf8(c).unwrap();
            assert!(validate_symbol(&str::repeat(c, 3)).is_err());
        });
    }
}

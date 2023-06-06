/*!
This is an *IBC Enabled* contract that allows us to send CW20 tokens from one chain over the standard ICS20
protocol to the bank module of another chain. In short, it lets us send our custom CW20 tokens with IBC and use
them just like native tokens on other chains.

It is only designed to send tokens and redeem previously sent tokens. It will not mint tokens belonging
to assets originating on the foreign chain. This is different than the Golang `ibctransfer` module, but
we properly implement ICS20 and respond with an error message... let's hope the Go side handles this correctly.

For more information on this contract, please check out the
[README](https://github.com/CosmWasm/cw-plus/blob/main/contracts/cw20-ics20/README.md).
*/

pub mod amount;
pub mod contract;
mod error;
pub mod ibc;
mod migrations;
pub mod msg;
pub mod state;
mod test_helpers;

pub use crate::error::ContractError;

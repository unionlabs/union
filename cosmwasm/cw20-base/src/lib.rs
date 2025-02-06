/*!
This is a basic implementation of a cw20 contract. It implements
the [CW20 spec](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md) and is designed to
be deployed as is, or imported into other contracts to easily build
cw20-compatible tokens with custom logic.

Implements:

- [x] CW20 Base
- [x] Mintable extension
- [x] Allowances extension

For more information on this contract, please check out the
[README](https://github.com/CosmWasm/cw-plus/blob/main/contracts/cw20-base/README.md).
*/

pub mod allowances;
pub mod contract;
pub mod enumerable;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

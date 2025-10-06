use cosmwasm_std::Addr;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AccessManagedError {
    #[error("caller {caller} is not authorized")]
    AccessManagedUnauthorized { caller: Addr },
    #[error("caller {caller} requires a delay of {delay}")]
    AccessManagedRequiredDelay { caller: Addr, delay: u32 },
    #[error("{authority} is not a contract")]
    AccessManagedInvalidAuthority { authority: Addr },
}

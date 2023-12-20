//! A module containing an assertion framework for CosmWasm contracts.
//! The methods in here never panic but return errors instead.

/// Quick check for a guard. If the condition (first argument) is false,
/// then return the second argument `x` wrapped in `Err(x)`.
///
/// ```
/// # enum ContractError {
/// #     DelegatePerm {},
/// # }
/// #
/// # struct Permissions {
/// #     delegate: bool,
/// # }
/// #
/// # fn body() -> Result<(), ContractError> {
/// # let permissions = Permissions { delegate: true };
/// use cosmwasm_std::ensure;
/// ensure!(permissions.delegate, ContractError::DelegatePerm {});
///
/// // is the same as
///
/// if !permissions.delegate {
///   return Err(ContractError::DelegatePerm {});
/// }
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            return Err(std::convert::From::from($e));
        }
    };
}

/// Quick check for a guard. Like `assert_eq!`, but rather than panic,
/// it returns the third argument `x` wrapped in `Err(x)`.
///
/// ```
/// # use cosmwasm_std::{MessageInfo, Addr};
/// #
/// # enum ContractError {
/// #     Unauthorized {},
/// # }
/// # struct Config {
/// #     admin: String,
/// # }
/// #
/// # fn body() -> Result<(), ContractError> {
/// # let info = MessageInfo { sender: Addr::unchecked("foo"), funds: Vec::new() };
/// # let cfg = Config { admin: "foo".to_string() };
/// use cosmwasm_std::ensure_eq;
///
/// ensure_eq!(info.sender, cfg.admin, ContractError::Unauthorized {});
///
/// // is the same as
///
/// if info.sender != cfg.admin {
///   return Err(ContractError::Unauthorized {});
/// }
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ensure_eq {
    ($a:expr, $b:expr, $e:expr) => {
        // Not implemented via `ensure!` because the caller would have to import both macros.
        if !($a == $b) {
            return Err(std::convert::From::from($e));
        }
    };
}

/// Quick check for a guard. Like `assert_ne!`, but rather than panic,
/// it returns the third argument `x` wrapped in Err(x).
///
/// ```
/// # enum ContractError {
/// #     NotAVoter {},
/// # }
/// #
/// # fn body() -> Result<(), ContractError> {
/// # let voting_power = 123;
/// use cosmwasm_std::ensure_ne;
///
/// ensure_ne!(voting_power, 0, ContractError::NotAVoter {});
///
/// // is the same as
///
/// if voting_power != 0 {
///   return Err(ContractError::NotAVoter {});
/// }
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ensure_ne {
    ($a:expr, $b:expr, $e:expr) => {
        // Not implemented via `ensure!` because the caller would have to import both macros.
        if !($a != $b) {
            return Err(std::convert::From::from($e));
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::StdError;

    #[test]
    fn ensure_works() {
        fn check(a: usize, b: usize) -> Result<(), StdError> {
            ensure!(a == b, StdError::generic_err("foobar"));
            Ok(())
        }

        let err = check(5, 6).unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));

        check(5, 5).unwrap();
    }

    #[test]
    fn ensure_can_infer_error_type() {
        let check = |a, b| {
            ensure!(a == b, StdError::generic_err("foobar"));
            Ok(())
        };

        let err = check(5, 6).unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));

        check(5, 5).unwrap();
    }

    #[test]
    fn ensure_can_convert_into() {
        #[derive(Debug)]
        struct ContractError;

        impl From<StdError> for ContractError {
            fn from(_original: StdError) -> Self {
                ContractError
            }
        }

        fn check(a: usize, b: usize) -> Result<(), ContractError> {
            ensure!(a == b, StdError::generic_err("foobar"));
            Ok(())
        }

        let err = check(5, 6).unwrap_err();
        assert!(matches!(err, ContractError));

        check(5, 5).unwrap();
    }

    #[test]
    fn ensure_eq_works() {
        let check = |a, b| {
            ensure_eq!(a, b, StdError::generic_err("foobar"));
            Ok(())
        };

        let err = check("123", "456").unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));
        check("123", "123").unwrap();
    }

    #[test]
    fn ensure_eq_gets_precedence_right() {
        // If this was expanded to `true || false == false` we'd get equality.
        // It must be expanded to `(true || false) == false` and we expect inequality.

        #[allow(clippy::nonminimal_bool)]
        fn check() -> Result<(), StdError> {
            ensure_eq!(true || false, false, StdError::generic_err("foobar"));
            Ok(())
        }

        let _err = check().unwrap_err();
    }

    #[test]
    fn ensure_ne_works() {
        let check = |a, b| {
            ensure_ne!(a, b, StdError::generic_err("foobar"));
            Ok(())
        };

        let err = check("123", "123").unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));
        check("123", "456").unwrap();
    }

    #[test]
    fn ensure_ne_gets_precedence_right() {
        // If this was expanded to `true || false == false` we'd get equality.
        // It must be expanded to `(true || false) == false` and we expect inequality.

        #[allow(clippy::nonminimal_bool)]
        fn check() -> Result<(), StdError> {
            ensure_ne!(true || false, false, StdError::generic_err("foobar"));
            Ok(())
        }

        check().unwrap();
    }
}

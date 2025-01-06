mod client;

use super::*;

#[test]
fn display() {
    assert_eq!(
        ContractErrorKind::ArithmeticOverflow,
        ContractErrorKind::parse_from_error_message(&ContractError::ArithmeticOverflow.to_string())
            .unwrap()
    )
}

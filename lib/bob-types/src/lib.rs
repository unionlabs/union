use hex_literal::hex;
use unionlabs::primitives::H160;

// https://docs.gobob.xyz/learn/reference/contracts/#bob-mainnet-l2
pub const L2_TO_L1_MESSAGE_PASSER: H160 =
    H160::new(hex!("4200000000000000000000000000000000000016"));

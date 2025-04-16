use alloy::primitives::U256;
use hex_literal::hex;
use unionlabs::primitives::H160;

// Slot index of L2OutputOracle.l2outputs
pub const L2_OUTPUTS_SLOT: U256 = U256::from_limbs([3, 0, 0, 0]);

// https://docs.gobob.xyz/learn/reference/contracts/#bob-mainnet-l2
pub const L2_TO_L1_MESSAGE_PASSER: H160 =
    H160::new(hex!("4200000000000000000000000000000000000016"));

alloy::sol! {
    #![sol(rpc)]
    #![sol(all_derives)]

    // https://github.com/ethereum-optimism/optimism/blob/99a53381019d3571359d989671ccf70f8d69dfd9/packages/contracts-bedrock/src/libraries/Types.sol#L13C4-L17C6
    struct OutputProposal {
        bytes32 outputRoot;
        uint128 timestamp;
        uint128 l2BlockNumber;
    }

    interface L2OutputOracle {
        function SUBMISSION_INTERVAL() external view returns (uint256);
        function L2_BLOCK_TIME() external view returns (uint256);
        function FINALIZATION_PERIOD_SECONDS() external view returns (uint256);
        function latestBlockNumber() external view returns (uint256);
        function latestOutputIndex() external view returns (uint256);
        function getL2Output(uint256 l2OutputIndex) external view returns (OutputProposal memory);
        function getL2OutputIndexAfter(uint256 _l2BlockNumber) external view returns (uint256);
    }
}

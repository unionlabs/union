pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../contracts/lib/CometblsHelp.sol";
import "../../contracts/clients/CometblsClientV2.sol";
import "../../contracts/proto/tendermint/types/canonical.sol";

contract CodecTest is Test {
    function test_consensusState_encode_decode_iso(
        uint64 timestamp,
        bytes32 appHash,
        bytes32 nextValidatorsHash
    ) public {
        OptimizedConsensusState memory consensusState = OptimizedConsensusState({
            timestamp: timestamp,
            appHash: appHash,
            nextValidatorsHash: nextValidatorsHash
        });

        OptimizedConsensusState memory consensusState2 =
            abi.decode(abi.encode(consensusState), (OptimizedConsensusState));

        assertEq(consensusState.timestamp, consensusState2.timestamp);
        assertEq(consensusState.appHash, consensusState2.appHash);
        assertEq(
            consensusState.nextValidatorsHash,
            consensusState2.nextValidatorsHash
        );
    }
}

import "forge-std/Test.sol";
import "../../contracts/lib/CometblsHelp.sol";

contract CodecTest is Test {
    function test_consensusState_encode_decode_iso(
        uint64 timestamp,
        bytes32 root,
        bytes32 nextValidatorsHash
    ) public {
        OptimizedConsensusState
            memory consensusState = OptimizedConsensusState({
                timestamp: timestamp,
                root: root,
                nextValidatorsHash: nextValidatorsHash
            });

        OptimizedConsensusState memory consensusState2 = abi.decode(
            abi.encode(consensusState),
            (OptimizedConsensusState)
        );

        assertEq(consensusState.timestamp, consensusState2.timestamp);
        assertEq(consensusState.root, consensusState2.root);
        assertEq(
            consensusState.nextValidatorsHash,
            consensusState2.nextValidatorsHash
        );
    }
}

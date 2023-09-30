pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "../../contracts/core/IZKVerifier.sol";
import "../../contracts/lib/CometblsHelp.sol";
import "../../contracts/lib/Encoder.sol";
import "../../contracts/clients/TestnetVerifier.sol";
import "../../contracts/proto/ibc/core/channel/v1/channel.sol";
import "../../contracts/proto/tendermint/types/canonical.sol";

contract VerifierTest is Test {
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for IZKVerifier;

    IZKVerifier testnetVerifier;

    function setUp() public {
        testnetVerifier = new TestnetVerifier();
    }
}

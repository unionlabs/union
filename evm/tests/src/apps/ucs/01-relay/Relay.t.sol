pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "solidity-stringutils/strings.sol";
import "solady/utils/LibString.sol";
import "../../../../../contracts/apps/ucs/01-relay/Relay.sol";

contract RelayTests is Test {
    using LibString for *;

    function testRelay_isRemote_ok() public {
        assertEq(RelayLib.isRemote("a", "b", "a/b/X"), true);
        assertEq(RelayLib.isRemote("aa.bb", "c", "aa.bb/c/X"), true);
    }

    function testRelay_isRemote_ko() public {
        assertEq(RelayLib.isRemote("a", "b", "b/b/X"), false);
        assertEq(RelayLib.isRemote("aa.bb", "c", "aa.b/c/X"), false);
    }

    function testRelay_makeForeignDenom() public {
        assertEq(RelayLib.makeForeignDenom("a", "b", "BLA"), "a/b/BLA");
        assertEq(
            RelayLib.makeForeignDenom("wasm.xyz", "channel-1", "muno"),
            "wasm.xyz/channel-1/muno"
        );
    }

    function testRelay_makeDenomPrefix() public {
        assertEq(RelayLib.makeDenomPrefix("a", "b"), "a/b/");
        assertEq(
            RelayLib.makeDenomPrefix("wasm.xyz", "channel-99"),
            "wasm.xyz/channel-99/"
        );
    }

    function testRelay_hexToAddress(address addr) public {
        assertEq(RelayLib.hexToAddress(addr.toHexString()), addr);
    }
}

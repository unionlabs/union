pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "../../../contracts/apps/ucs/03-zkgm/Zkgm.sol";

contract ZkgmTests is Test {
    function test_lastChannelFromPathOk_1(
        uint32 a
    ) public {
        vm.assume(a > 0);
        assertEq(
            ZkgmLib.lastChannelFromPath(ZkgmLib.updateChannelPath(0, a)), a
        );
    }

    function test_lastChannelFromPathOk_2(uint32 a, uint32 b) public {
        vm.assume(a > 0);
        vm.assume(b > 0);
        assertEq(
            ZkgmLib.lastChannelFromPath(
                ZkgmLib.updateChannelPath(ZkgmLib.updateChannelPath(0, a), b)
            ),
            b
        );
    }

    function test_lastChannelFromPathOk_3(
        uint32 a,
        uint32 b,
        uint32 c
    ) public {
        vm.assume(a > 0);
        vm.assume(b > 0);
        vm.assume(c > 0);
        assertEq(
            ZkgmLib.lastChannelFromPath(
                ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(
                        ZkgmLib.updateChannelPath(0, a), b
                    ),
                    c
                )
            ),
            c
        );
    }

    function test_channelPathOk(
        uint32 a,
        uint32 b,
        uint32 c,
        uint32 d,
        uint32 e,
        uint32 f,
        uint32 g,
        uint32 h
    ) public {
        vm.assume(a > 0);
        vm.assume(b > 0);
        vm.assume(c > 0);
        vm.assume(d > 0);
        vm.assume(e > 0);
        vm.assume(f > 0);
        vm.assume(g > 0);
        vm.assume(h > 0);
        assertEq(
            ZkgmLib.updateChannelPath(
                ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(
                        ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(
                                ZkgmLib.updateChannelPath(
                                    ZkgmLib.updateChannelPath(
                                        ZkgmLib.updateChannelPath(0, a), b
                                    ),
                                    c
                                ),
                                d
                            ),
                            e
                        ),
                        f
                    ),
                    g
                ),
                h
            ),
            uint256(a) | uint256(b) << 32 | uint256(c) << 64 | uint256(d) << 96
                | uint256(e) << 128 | uint256(f) << 160 | uint256(g) << 192
                | uint256(h) << 224
        );
    }
}

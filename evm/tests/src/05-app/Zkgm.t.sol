pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";

import "../../../contracts/apps/ucs/03-zkgm/Zkgm.sol";

contract ZkgmTests is Test {
    UCS03Zkgm zkgm;

    function setUp() public {
        UCS03Zkgm implementation = new UCS03Zkgm();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                UCS03Zkgm.initialize.selector,
                IIBCPacket(address(this)),
                address(this),
                address(0)
            )
        );
        zkgm = UCS03Zkgm(address(proxy));
    }

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

    function test_proxyInitialization_ok(address handler, address owner, address wethAddress) public {
        vm.assume(handler != address(0));
        vm.assume(owner != address(0));
        vm.assume(wethAddress != address(0));
        UCS03Zkgm implementation = new UCS03Zkgm();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                UCS03Zkgm.initialize.selector,
                handler,
                owner,
                wethAddress
            )
        );
        UCS03Zkgm zkgm = UCS03Zkgm(address(proxy));
        assertEq(address(zkgm.ibcHandler()), handler);
        assertEq(zkgm.owner(), owner);
        assertEq(address(zkgm.weth()), wethAddress);
    }
}

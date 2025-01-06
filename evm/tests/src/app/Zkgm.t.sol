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
                address(this)
            )
        );
        zkgm = UCS03Zkgm(address(proxy));
    }

    function decodeFungible(
        bytes calldata b
    ) public returns (FungibleAssetTransferPacket calldata) {
        return ZkgmLib.decodeFungibleAssetTransfer(b);
    }

    function decodeSyscall(
        bytes calldata b
    ) public returns (SyscallPacket calldata) {
        return ZkgmLib.decodeSyscall(b);
    }

    function decode(
        bytes calldata b
    ) public returns (ZkgmPacket calldata) {
        return ZkgmLib.decode(b);
    }

    function check(ZkgmPacket calldata a, bytes calldata b) public {
        bytes memory x = ZkgmLib.encode(a);
        console.logBytes(x);
        console.logBytes(b);
        assertEq(keccak256(x), keccak256(b));
    }

    function test_sendZkgmPacket() public {
        bytes memory rawFungible = abi.encode(
            hex"153919669Edc8A5D0c8D1E4507c9CE60435A1177",
            hex"153919669Edc8A5D0c8D1E4507c9CE60435A1177",
            hex"d1B482D1B947A96E96C9b76d15De34f7f70A20A1",
            uint256(5),
            "ChainLink Token",
            "LINK",
            uint256(9),
            hex"779877a7b0d9e8603169ddbd7836e478b4624789",
            uint256(8),
            false
        );
        FungibleAssetTransferPacket memory transfer =
            this.decodeFungible(rawFungible);
        console.log(transfer.onlyMaker);
        console.log(transfer.askAmount);
        console.logBytes(rawFungible);
        console.logBytes(
            abi.encode(
                ZkgmLib.ZKGM_VERSION_0,
                ZkgmLib.SYSCALL_FUNGIBLE_ASSET_TRANSFER,
                rawFungible
            )
        );

        bytes memory rawZk =
            hex"000000000000000000000000000000000000000000000000000000000000000900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000034000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002c00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000240000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000000900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014779877a7b0d9e8603169ddbd7836e478b462478900000000000000000000000000000000000000000000000000000000000000000000000000000000000000044c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f436861696e4c696e6b20546f6b656e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000142A2868c2fa4F1480A22FfE960aA4dac57f2D7a44000000000000000000000000";

        ZkgmPacket memory p = this.decode(rawZk);
        SyscallPacket memory s = this.decodeSyscall(p.syscall);
        FungibleAssetTransferPacket memory f = this.decodeFungible(s.packet);

        assertEq(f.onlyMaker, false);
        assertEq(f.sentTokenPrefix, 0);

        (address wrapped,) = zkgm.predictWrappedToken(
            0, 5, hex"779877a7b0d9e8603169ddbd7836e478b4624789"
        );

        console.log(wrapped);

        zkgm.onRecvPacket(
            IBCPacket({
                sourceChannelId: 3,
                destinationChannelId: 5,
                data: rawZk,
                timeoutHeight: 0,
                timeoutTimestamp: 0
            }),
            address(this),
            hex""
        );
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
}

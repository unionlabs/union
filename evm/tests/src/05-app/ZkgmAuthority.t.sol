pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "./Zkgm.t.sol";

contract ZkgmAuthorityTest is Test {
    using LibString for *;

    TestIBCHandler handler;
    TestERC20 erc20;
    ZkgmERC20 erc20Impl;
    TestWETH weth;
    TestZkgm zkgm;
    Manager manager;

    address authority;

    function setUp() public {
        authority = makeAddr("authority");

        weth = new TestWETH();
        erc20 = new TestERC20("Test", "T", 18);
        handler = new TestIBCHandler();
        erc20Impl = new ZkgmERC20();

        vm.startPrank(authority);
        manager = Manager(
            address(
                new ERC1967Proxy(
                    address(new Manager()),
                    abi.encodeCall(Manager.initialize, (authority))
                )
            )
        );
        vm.stopPrank();

        ERC1967Proxy proxy = new ERC1967Proxy(
            address(new TestZkgm(handler, weth, erc20Impl)),
            abi.encodeCall(UCS03Zkgm.initialize, (address(manager)))
        );
        zkgm = TestZkgm(payable(address(proxy)));
        zkgm.doCreateStakeNFTManager();

        erc20.mint(address(this), type(uint128).max);
        erc20.approve(address(zkgm), type(uint256).max);
    }

    // Test registerGovernanceToken function access control
    function test_registerGovernanceToken_authorized(
        uint32 channelId,
        bytes memory unwrappedToken,
        bytes32 metadataImage
    ) public {
        vm.assume(channelId > 0);
        vm.assume(unwrappedToken.length > 0);
        vm.assume(metadataImage != bytes32(0));

        vm.prank(authority);
        zkgm.registerGovernanceToken(
            channelId,
            GovernanceToken({
                unwrappedToken: unwrappedToken,
                metadataImage: metadataImage
            })
        );

        (bytes memory storedToken, bytes32 storedImage) =
            zkgm.channelGovernanceToken(channelId);
        assertEq(
            keccak256(storedToken),
            keccak256(unwrappedToken),
            "Unwrapped token not stored correctly"
        );
        assertEq(
            storedImage, metadataImage, "Metadata image not stored correctly"
        );
    }

    function test_registerGovernanceToken_unauthorized(
        address nonAuthority,
        uint32 channelId,
        bytes memory unwrappedToken,
        bytes32 metadataImage
    ) public {
        vm.assume(channelId > 0);
        vm.assume(unwrappedToken.length > 0);
        vm.assume(metadataImage != bytes32(0));
        vm.assume(nonAuthority != authority);
        vm.assume(nonAuthority != address(0));

        vm.prank(nonAuthority);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, nonAuthority
            )
        );
        zkgm.registerGovernanceToken(
            channelId,
            GovernanceToken({
                unwrappedToken: unwrappedToken,
                metadataImage: metadataImage
            })
        );
    }

    function test_registerGovernanceToken_alreadySet(
        uint32 channelId,
        bytes memory unwrappedToken1,
        bytes memory unwrappedToken2,
        bytes32 metadataImage1,
        bytes32 metadataImage2
    ) public {
        vm.assume(channelId > 0);
        vm.assume(unwrappedToken1.length > 0);
        vm.assume(unwrappedToken2.length > 0);
        vm.assume(keccak256(unwrappedToken1) != keccak256(unwrappedToken2));
        vm.assume(metadataImage1 != bytes32(0));
        vm.assume(metadataImage2 != bytes32(0));

        vm.startPrank(authority);
        zkgm.registerGovernanceToken(
            channelId,
            GovernanceToken({
                unwrappedToken: unwrappedToken1,
                metadataImage: metadataImage1
            })
        );

        vm.expectRevert(ZkgmLib.ErrChannelGovernanceTokenAlreadySet.selector);
        zkgm.registerGovernanceToken(
            channelId,
            GovernanceToken({
                unwrappedToken: unwrappedToken2,
                metadataImage: metadataImage2
            })
        );
        vm.stopPrank();
    }

    // Test overwriteGovernanceToken function access control
    function test_overwriteGovernanceToken_authorized(
        uint32 channelId,
        bytes memory unwrappedToken1,
        bytes memory unwrappedToken2,
        bytes32 metadataImage1,
        bytes32 metadataImage2
    ) public {
        vm.assume(channelId > 0);
        vm.assume(unwrappedToken1.length > 0);
        vm.assume(unwrappedToken2.length > 0);
        vm.assume(metadataImage1 != bytes32(0));
        vm.assume(metadataImage2 != bytes32(0));

        vm.startPrank(authority);
        zkgm.registerGovernanceToken(
            channelId,
            GovernanceToken({
                unwrappedToken: unwrappedToken1,
                metadataImage: metadataImage1
            })
        );

        zkgm.overwriteGovernanceToken(
            channelId,
            GovernanceToken({
                unwrappedToken: unwrappedToken2,
                metadataImage: metadataImage2
            })
        );
        vm.stopPrank();

        (bytes memory storedToken, bytes32 storedImage) =
            zkgm.channelGovernanceToken(channelId);
        assertEq(
            keccak256(storedToken),
            keccak256(unwrappedToken2),
            "Unwrapped token not overwritten correctly"
        );
        assertEq(
            storedImage,
            metadataImage2,
            "Metadata image not overwritten correctly"
        );
    }

    function test_overwriteGovernanceToken_unauthorized(
        address nonAuthority,
        uint32 channelId,
        bytes memory unwrappedToken,
        bytes32 metadataImage
    ) public {
        vm.assume(channelId > 0);
        vm.assume(unwrappedToken.length > 0);
        vm.assume(metadataImage != bytes32(0));
        vm.assume(nonAuthority != authority);
        vm.assume(nonAuthority != address(0));

        vm.prank(nonAuthority);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, nonAuthority
            )
        );
        zkgm.overwriteGovernanceToken(
            channelId,
            GovernanceToken({
                unwrappedToken: unwrappedToken,
                metadataImage: metadataImage
            })
        );
    }

    // Test pause function access control
    function test_pause_authorized() public {
        vm.prank(authority);
        zkgm.pause();

        assertTrue(zkgm.paused(), "Contract should be paused");
    }

    function test_pause_unauthorized(
        address nonAuthority
    ) public {
        vm.assume(nonAuthority != authority);
        vm.assume(nonAuthority != address(0));

        vm.prank(nonAuthority);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, nonAuthority
            )
        );
        zkgm.pause();

        assertFalse(zkgm.paused(), "Contract should not be paused");
    }

    // Test unpause function access control
    function test_unpause_authorized() public {
        vm.prank(authority);
        zkgm.pause();
        assertTrue(zkgm.paused(), "Contract should be paused");

        vm.prank(authority);
        zkgm.unpause();

        assertFalse(zkgm.paused(), "Contract should be unpaused");
    }

    function test_unpause_unauthorized(
        address nonAuthority
    ) public {
        vm.assume(nonAuthority != authority);
        vm.assume(nonAuthority != address(0));

        vm.prank(authority);
        zkgm.pause();
        assertTrue(zkgm.paused(), "Contract should be paused");

        vm.prank(nonAuthority);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, nonAuthority
            )
        );
        zkgm.unpause();

        assertTrue(zkgm.paused(), "Contract should remain paused");
    }

    function test_setBucketConfig_unauthorized(
        address nonAuthority,
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset
    ) public {
        vm.assume(token != address(0));
        vm.assume(nonAuthority != authority);
        vm.assume(nonAuthority != address(0));

        vm.prank(nonAuthority);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, nonAuthority
            )
        );
        zkgm.setBucketConfig(token, capacity, refillRate, reset);
    }

    // Test upgradeToAndCall function (UUPSUpgradeable implementation)
    function test_upgradeToAndCall_authorized() public {
        TestZkgm newImplementation = new TestZkgm(handler, weth, erc20Impl);

        vm.prank(authority);
        zkgm.upgradeToAndCall(address(newImplementation), hex"");

        bytes32 implementationSlot =
            0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;
        bytes32 currentImpl = vm.load(address(zkgm), implementationSlot);
        assertEq(
            address(uint160(uint256(currentImpl))),
            address(newImplementation),
            "Implementation not upgraded correctly"
        );
    }

    function test_upgradeToAndCall_unauthorized(
        address nonAuthority
    ) public {
        vm.assume(nonAuthority != authority);
        vm.assume(nonAuthority != address(0));

        TestZkgm newImplementation = new TestZkgm(handler, weth, erc20Impl);

        vm.prank(nonAuthority);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, nonAuthority
            )
        );
        zkgm.upgradeToAndCall(address(newImplementation), hex"");
    }

    // Test behavior of functions when contract is paused
    function test_paused_functionality(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        address caller
    ) public {
        vm.assume(channelId > 0);

        Instruction memory instruction = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_MULTIPLEX,
            operand: ZkgmLib.encodeMultiplex(
                Multiplex({
                    sender: abi.encodePacked(caller),
                    eureka: false,
                    contractAddress: abi.encodePacked(address(0x1234)),
                    contractCalldata: hex""
                })
            )
        });

        vm.prank(authority);
        zkgm.pause();
        assertTrue(zkgm.paused(), "Contract should be paused");

        vm.prank(address(handler));
        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        zkgm.send(channelId, timeoutHeight, timeoutTimestamp, salt, instruction);

        IBCPacket memory packet = IBCPacket({
            sourceChannelId: channelId,
            destinationChannelId: channelId,
            data: ZkgmLib.encode(
                ZkgmPacket({salt: salt, path: 0, instruction: instruction})
            ),
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });

        vm.prank(address(handler));
        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        zkgm.onRecvPacket(caller, packet, address(0), "");

        vm.prank(address(handler));
        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        zkgm.onRecvIntentPacket(caller, packet, address(0), "");

        vm.prank(address(handler));
        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        zkgm.onAcknowledgementPacket(caller, packet, "", address(0));

        vm.prank(address(handler));
        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        zkgm.onTimeoutPacket(caller, packet, address(0));
    }

    receive() external payable {}
}

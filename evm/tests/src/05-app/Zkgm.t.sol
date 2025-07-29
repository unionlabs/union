pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "solady/utils/LibBytes.sol";
import "solady/utils/LibString.sol";

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";

import "../../../contracts/core/Types.sol";
import "../../../contracts/core/25-handler/IBCHandler.sol";
import "../../../contracts/core/04-channel/IBCPacket.sol";
import "../../../contracts/core/05-port/IIBCModule.sol";
import "../../../contracts/apps/ucs/03-zkgm/IWETH.sol";
import "../../../contracts/apps/ucs/03-zkgm/Zkgm.sol";
import "../../../contracts/apps/Base.sol";
import "../../../contracts/Manager.sol";
import "../../../contracts/U.sol";

contract TestZkgm is UCS03Zkgm {
    constructor(
        IIBCModulePacket _ibcHandler,
        IWETH _weth,
        ZkgmERC20 _erc20Impl
    )
        UCS03Zkgm(
            _ibcHandler,
            new UCS03ZkgmSendImpl(
                _ibcHandler, _weth, _erc20Impl, "Ether", "ETH", 18
            ),
            new UCS03ZkgmStakeImpl(_ibcHandler),
            new UCS03ZkgmTokenOrderImpl(_weth, _erc20Impl, true)
        )
    {}

    function doExecuteForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        Forward calldata forward
    ) public returns (bytes memory) {
        return _executeForward(
            ibcPacket,
            relayer,
            relayerMsg,
            salt,
            path,
            ZkgmLib.INSTR_VERSION_0,
            forward,
            false
        );
    }

    function doExecuteCall(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        bytes32 salt,
        Call calldata call
    ) public returns (bytes memory) {
        return _executeCall(
            caller, ibcPacket, relayer, relayerMsg, path, salt, call, false
        );
    }

    function doIncreaseOutstandingV2(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        bytes calldata quoteToken,
        uint256 amount
    ) public {
        _increaseOutstandingV2(sourceChannelId, path, token, quoteToken, amount);
    }

    function doDecreaseOutstandingV2(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        bytes calldata quoteToken,
        uint256 amount
    ) public {
        _decreaseOutstandingV2(sourceChannelId, path, token, quoteToken, amount);
    }

    function doSetBucketConfig(
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset
    ) public {
        _setBucketConfig(token, capacity, refillRate, reset);
    }

    function doUpdateStake(
        uint256 tokenId,
        uint32 channelId,
        bytes memory validator,
        uint256 amount,
        ZkgmStakeState state,
        uint256 unstakingCompletion
    ) public {
        // Update the stake state directly in the mapping
        stakes[tokenId] = ZkgmStake({
            state: state,
            channelId: channelId,
            validator: validator,
            amount: amount,
            unstakingCompletion: unstakingCompletion
        });
    }

    function doCreateStakeNFTManager() public {
        _getStakeNFTManager();
    }
}

contract TestIBCHandler is IIBCModulePacket {
    event OnSendPacket(IBCPacket packet);

    using LibBytes for *;

    error ErrInvalidChannel();
    error ErrUnknownPacket();

    mapping(uint32 => uint32) public channels;
    mapping(bytes32 => bytes) public acks;

    function setChannel(uint32 src, uint32 dst) public {
        channels[src] = dst;
    }

    function sendPacket(
        uint32 sourceChannel,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external override returns (IBCPacket memory) {
        uint32 destinationChannelId = channels[sourceChannel];
        if (destinationChannelId == 0) {
            revert ErrInvalidChannel();
        }
        IBCPacket memory packet = IBCPacket({
            sourceChannelId: sourceChannel,
            destinationChannelId: destinationChannelId,
            data: data,
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
        acks[IBCPacketLib.commitPacket(packet)] = hex"01";
        emit OnSendPacket(packet);
        return packet;
    }

    function writeAcknowledgement(
        IBCPacket calldata packet,
        bytes memory acknowledgement
    ) external override {
        bytes32 commitmentKey = IBCPacketLib.commitPacket(packet);
        if (!acks[commitmentKey].eq(hex"01")) {
            revert ErrUnknownPacket();
        }
        acks[commitmentKey] = acknowledgement;
    }
}

contract TestERC20 is ERC20 {
    uint8 _decimals;

    constructor(
        string memory name,
        string memory symbol,
        uint8 d
    ) ERC20(name, symbol) {
        _decimals = d;
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    function mint(address to, uint256 amount) public {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) public {
        _burn(from, amount);
    }
}

contract TestWETH is IWETH, TestERC20 {
    error ETHTransferFailed();

    constructor() TestERC20("Wrapped Ether", "WETH", 18) {}

    function deposit() public payable virtual {
        _mint(msg.sender, msg.value);
    }

    function withdraw(
        uint256 amount
    ) public virtual {
        _burn(msg.sender, amount);
        assembly {
            if iszero(
                call(
                    gas(), caller(), amount, codesize(), 0x00, codesize(), 0x00
                )
            ) {
                mstore(0x00, 0xb12d13eb) // `ETHTransferFailed()`.
                revert(0x1c, 0x04)
            }
        }
    }

    receive() external payable virtual {
        deposit();
    }
}

contract TestCallTarget is IZkgmable, IIBCModuleRecv {
    error ErrNotZkgm();

    event OnZkgm(
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes sender,
        bytes message
    );
    event OnIntentZkgm(
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes sender,
        bytes message
    );
    event OnRecvPacket(IBCPacket packet, address relayer, bytes relayerMsg);
    event OnRecvIntentPacket(
        IBCPacket packet, address relayer, bytes relayerMsg
    );

    address zkgm;

    constructor(
        address _zkgm
    ) {
        zkgm = _zkgm;
    }

    modifier onlyZkgm() {
        _checkZkgm();
        _;
    }

    function _checkZkgm() internal view {
        if (zkgm != msg.sender) {
            revert ErrNotZkgm();
        }
    }

    function onZkgm(
        address caller,
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata sender,
        bytes calldata message,
        address relayer,
        bytes calldata relayerMsg
    ) public onlyZkgm {
        emit OnZkgm(
            path, sourceChannelId, destinationChannelId, sender, message
        );
    }

    function onIntentZkgm(
        address caller,
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata sender,
        bytes calldata message,
        address relayer,
        bytes calldata relayerMsg
    ) public onlyZkgm {
        emit OnIntentZkgm(
            path, sourceChannelId, destinationChannelId, sender, message
        );
    }

    function onRecvPacket(
        address caller,
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) public onlyZkgm returns (bytes memory) {
        emit OnRecvPacket(packet, relayer, relayerMsg);
        return hex"01";
    }

    function onRecvIntentPacket(
        address caller,
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) public onlyZkgm returns (bytes memory) {
        emit OnRecvIntentPacket(packet, relayer, relayerMsg);
        return hex"01";
    }
}

contract ZkgmTests is Test {
    using LibString for *;

    Manager manager;
    TestCallTarget callTarget;
    TestIBCHandler handler;
    TestERC20 erc20;
    ZkgmERC20 erc20Impl;
    TestWETH weth;
    TestZkgm zkgm;

    Instruction dummyCall = Instruction({
        version: ZkgmLib.INSTR_VERSION_0,
        opcode: ZkgmLib.OP_CALL,
        operand: ZkgmLib.encodeCall(
            Call({
                sender: abi.encodePacked(address(0)),
                eureka: false,
                contractAddress: abi.encodePacked(address(0)),
                contractCalldata: hex""
            })
        )
    });

    function setUp() public virtual {
        weth = new TestWETH();
        erc20 = new TestERC20("Test", "T", 18);
        handler = new TestIBCHandler();
        erc20Impl = new ZkgmERC20();
        manager = Manager(
            address(
                new ERC1967Proxy(
                    address(new Manager()),
                    abi.encodeCall(Manager.initialize, (address(this)))
                )
            )
        );
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(new TestZkgm(handler, weth, erc20Impl)),
            abi.encodeCall(UCS03Zkgm.initialize, (address(manager)))
        );
        zkgm = TestZkgm(payable(address(proxy)));
        zkgm.doCreateStakeNFTManager();
        callTarget = new TestCallTarget(address(zkgm));
    }

    receive() external payable {}

    function test_proxyInitialization_ok(
        address wethAddress,
        address handlerAddress,
        address erc20ImplAddress,
        address authorityAddress
    ) public {
        assumeUnusedAddress(handlerAddress);
        assumeUnusedAddress(authorityAddress);
        TestZkgm implementation = new TestZkgm(
            IIBCModulePacket(handlerAddress),
            IWETH(wethAddress),
            ZkgmERC20(erc20ImplAddress)
        );
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeCall(UCS03Zkgm.initialize, (authorityAddress))
        );
        TestZkgm _zkgm = TestZkgm(payable(address(proxy)));
        assertEq(address(_zkgm.IBC_HANDLER()), handlerAddress);
        assertEq(_zkgm.authority(), authorityAddress);
    }

    function test_channelPath_ok(
        uint32 a,
        uint32 b,
        uint32 c,
        uint32 d,
        uint32 e,
        uint32 f,
        uint32 g,
        uint32 h
    ) public {
        // channel ids are non-zero
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

    function test_reverseChannelPath_2_ok(uint32 a, uint32 b) public {
        // channel ids are non-zero
        vm.assume(a > 0);
        vm.assume(b > 0);
        uint256 channelPath =
            ZkgmLib.updateChannelPath(ZkgmLib.updateChannelPath(0, a), b);
        assertEq(
            ZkgmLib.reverseChannelPath(channelPath),
            uint256(b) | uint256(a) << 32
        );
    }

    function test_reverseChannelPath_ok(
        uint32 a,
        uint32 b,
        uint32 c,
        uint32 d,
        uint32 e,
        uint32 f,
        uint32 g,
        uint32 h
    ) public {
        // channel ids are non-zero
        vm.assume(a > 0);
        vm.assume(b > 0);
        vm.assume(c > 0);
        vm.assume(d > 0);
        vm.assume(e > 0);
        vm.assume(f > 0);
        vm.assume(g > 0);
        vm.assume(h > 0);
        uint256 channelPath = ZkgmLib.updateChannelPath(
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
        );
        assertEq(
            ZkgmLib.reverseChannelPath(channelPath),
            uint256(h) | uint256(g) << 32 | uint256(f) << 64 | uint256(e) << 96
                | uint256(d) << 128 | uint256(c) << 160 | uint256(b) << 192
                | uint256(a) << 224
        );
    }

    function test_popChannelFromPath_ok(
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
        uint256 channelPath = ZkgmLib.updateChannelPath(
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
        );
        uint256 expectedBaseChannelPath = ZkgmLib.updateChannelPath(
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
        );
        (uint256 baseChannelPath, uint32 finalChannelId) =
            ZkgmLib.popChannelFromPath(channelPath);
        assertEq(baseChannelPath, expectedBaseChannelPath);
        assertEq(finalChannelId, h);
    }

    function test_popChannelFromPath_ok_2(
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
        uint256 expectedBaseChannelPath = ZkgmLib.updateChannelPath(
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
        );
        uint256 channelPath = ZkgmLib.updateChannelPath(
            ZkgmLib.updateChannelPath(expectedBaseChannelPath, g), h
        );
        (uint256 baseChannelPath, uint32 finalChannelId) =
            ZkgmLib.popChannelFromPath(channelPath);
        (uint256 baseChannelPath2, uint32 finalChannelId2) =
            ZkgmLib.popChannelFromPath(baseChannelPath);
        assertEq(bytes32(baseChannelPath2), bytes32(expectedBaseChannelPath));
        assertEq(finalChannelId, h);
        assertEq(finalChannelId2, g);
    }

    function test_dequeueChannelFromPath_ok(
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
        uint256 channelPath = ZkgmLib.updateChannelPath(
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
        );
        uint256 expectedBaseChannelPath = ZkgmLib.updateChannelPath(
            ZkgmLib.updateChannelPath(
                ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(
                        ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(
                                ZkgmLib.updateChannelPath(0, b), c
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
        );
        (uint256 tailChannelPath, uint32 firstChannelId) =
            ZkgmLib.dequeueChannelFromPath(channelPath);
        assertEq(tailChannelPath, expectedBaseChannelPath);
        assertEq(firstChannelId, a);
    }

    function test_dequeueChannelFromPath_ok_2(
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
        uint256 channelPath = ZkgmLib.updateChannelPath(
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
        );
        uint256 expectedBaseChannelPath = ZkgmLib.updateChannelPath(
            ZkgmLib.updateChannelPath(
                ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(
                        ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(0, c), d
                        ),
                        e
                    ),
                    f
                ),
                g
            ),
            h
        );
        (uint256 tailChannelPath, uint32 firstChannelId) =
            ZkgmLib.dequeueChannelFromPath(channelPath);
        (uint256 tailChannelPath2, uint32 secondChannelId) =
            ZkgmLib.dequeueChannelFromPath(tailChannelPath);
        assertEq(tailChannelPath2, expectedBaseChannelPath);
        assertEq(firstChannelId, a);
        assertEq(secondChannelId, b);
    }

    function test_onChanOpenInit_onlyIBC(
        address caller,
        uint32 connectionId,
        uint32 channelId,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onChanOpenInit(
            caller, connectionId, channelId, ZkgmLib.IBC_VERSION_STR, relayer
        );
    }

    function test_tintForwardSalt_ok(
        bytes32 salt
    ) public {
        salt = bytes32(salt >> 8);
        assertFalse(ZkgmLib.isForwardedPacket(salt));
        assertTrue(ZkgmLib.isForwardedPacket(ZkgmLib.tintForwardSalt(salt)));
    }

    function test_tintForwardSalt_ok_2() public {
        test_tintForwardSalt_ok(
            0xdefe464db3fcf737aba09147ad0258e1f0913e3633c065053e744057b42dfefe
        );
    }

    function test_onChanOpenInit_ok(
        address caller,
        uint32 connectionId,
        uint32 channelId,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.prank(address(handler));
        zkgm.onChanOpenInit(
            caller, connectionId, channelId, ZkgmLib.IBC_VERSION_STR, relayer
        );
    }

    function test_onChanOpenInit_invalidVersion(
        address caller,
        uint32 connectionId,
        uint32 channelId,
        address relayer,
        string memory version
    ) public {
        vm.assume(channelId != 0);
        vm.prank(address(handler));
        vm.expectRevert(ZkgmLib.ErrInvalidIBCVersion.selector);
        zkgm.onChanOpenInit(caller, connectionId, channelId, version, relayer);
    }

    function test_onChanOpenTry_onlyIBC(
        address caller,
        uint32 connectionId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        address relayer
    ) public {
        vm.assume(channelId > 0);
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onChanOpenTry(
            caller,
            connectionId,
            channelId,
            counterpartyChannelId,
            ZkgmLib.IBC_VERSION_STR,
            ZkgmLib.IBC_VERSION_STR,
            relayer
        );
    }

    function test_onChanOpenTry_ok(
        address caller,
        uint32 connectionId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.prank(address(handler));
        zkgm.onChanOpenTry(
            caller,
            connectionId,
            channelId,
            counterpartyChannelId,
            ZkgmLib.IBC_VERSION_STR,
            ZkgmLib.IBC_VERSION_STR,
            relayer
        );
    }

    function test_onChanOpenTry_invalidVersion(
        address caller,
        uint32 connectionId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        string memory version,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.prank(address(handler));
        vm.expectRevert(ZkgmLib.ErrInvalidIBCVersion.selector);
        zkgm.onChanOpenTry(
            caller,
            connectionId,
            channelId,
            counterpartyChannelId,
            version,
            ZkgmLib.IBC_VERSION_STR,
            relayer
        );
    }

    function test_onChanOpenTry_invalidCounterpartyVersion(
        address caller,
        uint32 connectionId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        string memory counterpartyVersion,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.prank(address(handler));
        vm.expectRevert(ZkgmLib.ErrInvalidIBCVersion.selector);
        zkgm.onChanOpenTry(
            caller,
            connectionId,
            channelId,
            counterpartyChannelId,
            ZkgmLib.IBC_VERSION_STR,
            counterpartyVersion,
            relayer
        );
    }

    function test_onChanCloseInit_onlyIBC(
        address caller,
        uint32 channelId,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onChanCloseInit(caller, channelId, relayer);
    }

    function test_onChanCloseInit_impossible(
        address caller,
        uint32 channelId,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.prank(address(handler));
        vm.expectRevert(ZkgmLib.ErrInfiniteGame.selector);
        zkgm.onChanCloseInit(caller, channelId, relayer);
    }

    function test_onChanCloseConfirm_onlyIBC(
        address caller,
        uint32 channelId,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onChanCloseConfirm(caller, channelId, relayer);
    }

    function test_onChanCloseConfirm_impossible(
        address caller,
        uint32 channelId,
        address relayer
    ) public {
        vm.assume(channelId != 0);
        vm.prank(address(handler));
        vm.expectRevert(ZkgmLib.ErrInfiniteGame.selector);
        zkgm.onChanCloseConfirm(caller, channelId, relayer);
    }

    function test_onRecvPacket_onlyIBC(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes calldata relayerMsg
    ) public {
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onRecvPacket(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg
        );
    }

    function test_execute_onlySelf(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) public {
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        vm.expectRevert(ZkgmLib.ErrUnauthorized.selector);
        zkgm.execute(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            intent
        );
    }

    function test_verify_forward_ok() public {
        handler.setChannel(1, 10);
        zkgm.send(
            1,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_FORWARD,
                operand: ZkgmLib.encodeForward(
                    Forward({
                        path: ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(0, 10), 1
                        ),
                        timeoutHeight: type(uint64).max,
                        timeoutTimestamp: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_CALL,
                            operand: ZkgmLib.encodeCall(
                                Call({
                                    sender: abi.encodePacked(this),
                                    eureka: false,
                                    contractAddress: abi.encodePacked(this),
                                    contractCalldata: hex""
                                })
                            )
                        })
                    })
                )
            })
        );
    }

    function test_verify_forward_invalidInstruction(
        uint32 channelId
    ) public {
        vm.expectRevert(ZkgmLib.ErrInvalidForwardInstruction.selector);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_FORWARD,
                operand: ZkgmLib.encodeForward(
                    Forward({
                        path: ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(0, 10), 1
                        ),
                        timeoutHeight: type(uint64).max,
                        timeoutTimestamp: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_FORWARD,
                            operand: hex""
                        })
                    })
                )
            })
        );
    }

    function test_verify_call_ok(
        uint32 channelId,
        uint32 counterpartyChannelId,
        bytes memory contractAddress,
        bytes memory contractCalldata
    ) public {
        vm.assume(channelId > 0);
        vm.assume(counterpartyChannelId > 0);
        handler.setChannel(channelId, counterpartyChannelId);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_CALL,
                operand: ZkgmLib.encodeCall(
                    Call({
                        sender: abi.encodePacked(address(this)),
                        eureka: false,
                        contractAddress: contractAddress,
                        contractCalldata: contractCalldata
                    })
                )
            })
        );
    }

    function test_verify_call_invalidSender(
        uint32 channelId,
        address sender,
        bytes memory contractAddress,
        bytes memory contractCalldata
    ) public {
        assumeUnusedAddress(sender);
        vm.expectRevert(ZkgmLib.ErrInvalidCallSender.selector);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_CALL,
                operand: ZkgmLib.encodeCall(
                    Call({
                        sender: abi.encodePacked(sender),
                        eureka: false,
                        contractAddress: contractAddress,
                        contractCalldata: contractCalldata
                    })
                )
            })
        );
    }

    function test_verify_batch_ok(
        uint32 channelId,
        uint32 counterpartyChannelId,
        bytes memory contractAddress,
        bytes memory contractCalldata
    ) public {
        vm.assume(channelId > 0);
        vm.assume(counterpartyChannelId > 0);
        Instruction[] memory instructions = new Instruction[](1);
        instructions[0] = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_CALL,
            operand: ZkgmLib.encodeCall(
                Call({
                    sender: abi.encodePacked(address(this)),
                    eureka: false,
                    contractAddress: contractAddress,
                    contractCalldata: contractCalldata
                })
            )
        });
        handler.setChannel(channelId, counterpartyChannelId);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_BATCH,
                operand: ZkgmLib.encodeBatch(Batch({instructions: instructions}))
            })
        );
    }

    function test_verify_batch_invalidInstruction(
        uint32 channelId
    ) public {
        Instruction[] memory instructions = new Instruction[](1);
        instructions[0] = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_BATCH,
            operand: hex""
        });
        vm.expectRevert(ZkgmLib.ErrInvalidBatchInstruction.selector);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_BATCH,
                operand: ZkgmLib.encodeBatch(Batch({instructions: instructions}))
            })
        );
    }

    function test_verify_order_transfer_wrapped_burn_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        address receiver,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(receiver);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(quoteAmount <= baseAmount);
        }
        handler.setChannel(destinationChannelId, sourceChannelId);
        address quoteToken = test_onRecvPacket_transferNative_newWrapped(
            caller,
            sourceChannelId,
            destinationChannelId,
            relayer,
            relayerMsg,
            0,
            salt,
            sender,
            receiver,
            baseToken,
            baseTokenMeta,
            baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(receiver, address(0), quoteAmount);
        vm.prank(receiver);
        zkgm.send(
            destinationChannelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV1(
                    TokenOrderV1({
                        sender: abi.encodePacked(receiver),
                        receiver: sender,
                        baseToken: abi.encodePacked(quoteToken),
                        baseTokenPath: ZkgmLib.updateChannelPath(
                            0, destinationChannelId
                        ),
                        baseTokenSymbol: baseTokenMeta.symbol,
                        baseTokenName: baseTokenMeta.name,
                        baseTokenDecimals: baseTokenMeta.decimals,
                        baseAmount: quoteAmount,
                        quoteToken: abi.encodePacked(baseToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function _metadataImage(
        TokenMeta memory baseTokenMeta
    ) internal returns (bytes32) {
        TokenMetadata memory metadata = TokenMetadata({
            implementation: abi.encodePacked(erc20Impl),
            initializer: abi.encodeCall(
                ZkgmERC20.initialize,
                (
                    zkgm.authority(),
                    address(zkgm),
                    baseTokenMeta.name,
                    baseTokenMeta.symbol,
                    baseTokenMeta.decimals
                )
            )
        });
        return EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
    }

    function test_verify_order_v2_transfer_wrapped_burn_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        address receiver,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(receiver);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(0 < quoteAmount && quoteAmount <= baseAmount);
        }
        handler.setChannel(destinationChannelId, sourceChannelId);
        address quoteToken = test_onRecvPacket_transferNative_newWrapped_v2(
            caller,
            sourceChannelId,
            destinationChannelId,
            relayer,
            relayerMsg,
            0,
            salt,
            sender,
            receiver,
            baseToken,
            baseTokenMeta,
            baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(receiver, address(0), quoteAmount);
        bytes32 metadataImage = _metadataImage(baseTokenMeta);
        vm.prank(receiver);
        zkgm.send(
            destinationChannelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_2,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV2(
                    TokenOrderV2({
                        sender: abi.encodePacked(receiver),
                        receiver: sender,
                        baseToken: abi.encodePacked(quoteToken),
                        kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                        metadata: abi.encodePacked(metadataImage),
                        baseAmount: quoteAmount,
                        quoteToken: abi.encodePacked(baseToken),
                        quoteAmount: quoteAmount
                    })
                )
            })
        );
    }

    function test_verify_order_transfer_native_escrow_increaseOutstanding_ok(
        uint32 channelId,
        uint32 counterpartyChannelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        assumeUnusedAddress(caller);
        vm.assume(channelId > 0);
        vm.assume(counterpartyChannelId > 0);
        handler.setChannel(channelId, counterpartyChannelId);
        address baseToken = address(erc20);
        if (baseAmount > 0) {
            erc20.mint(caller, baseAmount);
            vm.prank(caller);
            erc20.approve(address(zkgm), baseAmount);
        }
        string memory symbol = erc20.symbol();
        string memory name = erc20.name();
        uint8 decimals = erc20.decimals();
        vm.expectEmit();
        emit IERC20.Transfer(caller, address(zkgm), baseAmount);
        assertEq(zkgm.channelBalanceV2(channelId, 0, baseToken, quoteToken), 0);
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV1(
                    TokenOrderV1({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseTokenPath: 0,
                        baseTokenSymbol: symbol,
                        baseTokenName: name,
                        baseTokenDecimals: decimals,
                        baseAmount: baseAmount,
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
        assertEq(
            zkgm.channelBalanceV2(channelId, 0, baseToken, quoteToken),
            baseAmount
        );
    }

    function test_verify_order_v2_transfer_native_escrow_increaseOutstanding_ok(
        uint32 channelId,
        uint32 counterpartyChannelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        assumeUnusedAddress(caller);
        vm.assume(channelId > 0);
        vm.assume(counterpartyChannelId > 0);
        handler.setChannel(channelId, counterpartyChannelId);
        address baseToken = address(erc20);
        if (baseAmount > 0) {
            erc20.mint(caller, baseAmount);
            vm.prank(caller);
            erc20.approve(address(zkgm), baseAmount);
        }
        TokenMetadata memory metadata = _metadata(
            TokenMeta({
                symbol: erc20.symbol(),
                name: erc20.name(),
                decimals: erc20.decimals()
            })
        );
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        vm.expectEmit();
        emit IERC20.Transfer(caller, address(zkgm), baseAmount);
        assertEq(
            zkgm.channelBalanceV2(
                channelId, 0, baseToken, abi.encodePacked(quoteToken)
            ),
            0
        );
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_2,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV2(
                    TokenOrderV2({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseAmount: baseAmount,
                        kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                        metadata: ZkgmLib.encodeTokenMetadata(metadata),
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelId, 0, baseToken, abi.encodePacked(quoteToken)
            ),
            baseAmount
        );
    }

    function test_verify_order_transfer_native_noAllowance(
        uint32 channelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(caller);
            vm.assume(baseAmount > 0);
        }
        address baseToken = address(erc20);
        string memory symbol = erc20.symbol();
        string memory name = erc20.name();
        uint8 decimals = erc20.decimals();
        vm.expectRevert();
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV1(
                    TokenOrderV1({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseTokenPath: 0,
                        baseTokenSymbol: symbol,
                        baseTokenName: name,
                        baseTokenDecimals: decimals,
                        baseAmount: baseAmount,
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function test_verify_order_v2_transfer_native_noAllowance(
        uint32 channelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(caller);
            vm.assume(baseAmount > 0);
        }
        address baseToken = address(erc20);
        TokenMetadata memory metadata = _metadata(
            TokenMeta({
                symbol: erc20.symbol(),
                name: erc20.name(),
                decimals: erc20.decimals()
            })
        );
        vm.expectRevert();
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_2,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV2(
                    TokenOrderV2({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseAmount: baseAmount,
                        kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                        metadata: ZkgmLib.encodeTokenMetadata(metadata),
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function test_verify_order_transfer_invalidSymbol(
        uint32 channelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount,
        string memory symbol
    ) public {
        {
            assumeUnusedAddress(caller);
            vm.assume(!symbol.eq(erc20.symbol()));
        }
        address baseToken = address(erc20);
        if (baseAmount > 0) {
            erc20.mint(caller, baseAmount);
            vm.prank(caller);
            erc20.approve(address(zkgm), baseAmount);
        }
        string memory name = erc20.name();
        uint8 decimals = erc20.decimals();
        vm.expectRevert(ZkgmLib.ErrInvalidAssetSymbol.selector);
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV1(
                    TokenOrderV1({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseTokenPath: 0,
                        baseTokenSymbol: symbol,
                        baseTokenName: name,
                        baseTokenDecimals: decimals,
                        baseAmount: baseAmount,
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function test_verify_order_transfer_invalidName(
        uint32 channelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount,
        string memory name
    ) public {
        {
            assumeUnusedAddress(caller);
            vm.assume(!name.eq(erc20.name()));
        }
        address baseToken = address(erc20);
        if (baseAmount > 0) {
            erc20.mint(caller, baseAmount);
            vm.prank(caller);
            erc20.approve(address(zkgm), baseAmount);
        }
        string memory symbol = erc20.symbol();
        uint8 decimals = erc20.decimals();
        vm.expectRevert(ZkgmLib.ErrInvalidAssetName.selector);
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV1(
                    TokenOrderV1({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseTokenPath: 0,
                        baseTokenSymbol: symbol,
                        baseTokenName: name,
                        baseTokenDecimals: decimals,
                        baseAmount: baseAmount,
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function test_verify_order_transfer_invalidDecimals(
        uint32 channelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount,
        uint8 decimals
    ) public {
        {
            assumeUnusedAddress(caller);
            vm.assume(decimals != erc20.decimals());
        }
        address baseToken = address(erc20);
        if (baseAmount > 0) {
            erc20.mint(caller, baseAmount);
            vm.prank(caller);
            erc20.approve(address(zkgm), baseAmount);
        }
        string memory symbol = erc20.symbol();
        string memory name = erc20.name();
        vm.expectRevert(ZkgmLib.ErrInvalidAssetDecimals.selector);
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV1(
                    TokenOrderV1({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseTokenPath: 0,
                        baseTokenSymbol: symbol,
                        baseTokenName: name,
                        baseTokenDecimals: decimals,
                        baseAmount: baseAmount,
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function test_verify_order_transfer_native_invalidOrigin(
        uint32 channelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount,
        uint256 baseTokenPath
    ) public {
        {
            assumeUnusedAddress(caller);
            vm.assume(baseTokenPath != 0);
        }
        address baseToken = address(erc20);
        if (baseAmount > 0) {
            erc20.mint(caller, baseAmount);
            vm.prank(caller);
            erc20.approve(address(zkgm), baseAmount);
        }
        string memory symbol = erc20.symbol();
        string memory name = erc20.name();
        uint8 decimals = erc20.decimals();
        vm.expectRevert(ZkgmLib.ErrInvalidAssetOrigin.selector);
        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV1(
                    TokenOrderV1({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseTokenPath: baseTokenPath,
                        baseTokenSymbol: symbol,
                        baseTokenName: name,
                        baseTokenDecimals: decimals,
                        baseAmount: baseAmount,
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function test_verify_order_v2_transfer_customMetadata_ok(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount,
        uint8 customMetadataType,
        bytes memory customMetadata
    ) public {
        {
            assumeUnusedAddress(caller);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(customMetadataType > ZkgmLib.TOKEN_ORDER_KIND_UNESCROW);
        }
        address baseToken = address(erc20);
        if (baseAmount > 0) {
            erc20.mint(caller, baseAmount);
            vm.prank(caller);
            erc20.approve(address(zkgm), baseAmount);
        }
        handler.setChannel(sourceChannelId, destinationChannelId);
        vm.prank(caller);
        zkgm.send(
            sourceChannelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_2,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV2(
                    TokenOrderV2({
                        sender: sender,
                        receiver: receiver,
                        baseToken: abi.encodePacked(baseToken),
                        baseAmount: baseAmount,
                        kind: customMetadataType,
                        metadata: customMetadata,
                        quoteToken: abi.encodePacked(quoteToken),
                        quoteAmount: baseAmount
                    })
                )
            })
        );
    }

    function test_verify_order_v2_transfer_wrapped_customMetadata(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        address receiver,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount,
        bytes calldata customMetadata
    ) public {
        {
            assumeUnusedAddress(receiver);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(quoteAmount <= baseAmount);
            vm.assume(quoteAmount > 0);
        }
        handler.setChannel(destinationChannelId, sourceChannelId);
        address quoteToken = test_onRecvPacket_transferNative_newWrapped_v2(
            caller,
            sourceChannelId,
            destinationChannelId,
            relayer,
            relayerMsg,
            0,
            salt,
            sender,
            receiver,
            baseToken,
            baseTokenMeta,
            baseAmount
        );

        vm.prank(receiver);
        zkgm.send(
            destinationChannelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_2,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV2(
                    TokenOrderV2({
                        sender: abi.encodePacked(receiver),
                        receiver: sender,
                        baseToken: abi.encodePacked(quoteToken),
                        kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                        metadata: customMetadata,
                        baseAmount: quoteAmount,
                        quoteToken: abi.encodePacked(baseToken),
                        quoteAmount: quoteAmount
                    })
                )
            })
        );
    }

    function test_executeForward_ok(
        uint32 previousSourceChannelId,
        uint32 previousDestinationChannelId,
        uint32 nextSourceChannelId,
        uint32 nextDestinationChannelId,
        bytes32 salt,
        uint128 path,
        address relayer,
        bytes memory relayerMsg
    ) public {
        {
            vm.assume(previousSourceChannelId != 0);
            vm.assume(previousDestinationChannelId != 0);
            vm.assume(nextSourceChannelId != 0);
            vm.assume(nextDestinationChannelId != 0);
        }
        handler.setChannel(nextSourceChannelId, nextDestinationChannelId);
        // We expect the protocol to re-emit a packet with the updated path and the sub-instruction
        vm.expectEmit();
        emit TestIBCHandler.OnSendPacket(
            IBCPacket({
                sourceChannelId: nextSourceChannelId,
                destinationChannelId: nextDestinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: ZkgmLib.deriveForwardSalt(salt),
                        path: ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(
                                path, previousDestinationChannelId
                            ),
                            nextSourceChannelId
                        ),
                        instruction: dummyCall
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            })
        );
        bytes memory ack = zkgm.doExecuteForward(
            IBCPacket({
                sourceChannelId: previousSourceChannelId,
                destinationChannelId: previousDestinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            salt,
            uint256(path),
            Forward({
                path: ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(0, previousDestinationChannelId),
                    nextSourceChannelId
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0,
                instruction: dummyCall
            })
        );
        assertEq(ZkgmLib.ACK_EMPTY, ack);
    }

    function test_executeForward_double_ok(
        uint32 previousSourceChannelId,
        uint32 previousDestinationChannelId,
        uint32 nextSourceChannelId,
        uint32 nextDestinationChannelId,
        uint32 previousDestinationChannelId2,
        uint32 nextSourceChannelId2,
        bytes32 salt,
        uint128 path,
        address relayer,
        bytes memory relayerMsg
    ) public {
        {
            vm.assume(previousSourceChannelId != 0);
            vm.assume(previousDestinationChannelId != 0);
            vm.assume(nextSourceChannelId != 0);
            vm.assume(nextDestinationChannelId != 0);
            vm.assume(previousDestinationChannelId2 != 0);
            vm.assume(nextSourceChannelId2 != 0);
        }
        handler.setChannel(nextSourceChannelId, nextDestinationChannelId);
        // We expect the protocol to re-emit a forward
        vm.expectEmit();
        emit TestIBCHandler.OnSendPacket(
            IBCPacket({
                sourceChannelId: nextSourceChannelId,
                destinationChannelId: nextDestinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: ZkgmLib.deriveForwardSalt(salt),
                        path: ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(
                                path, previousDestinationChannelId
                            ),
                            nextSourceChannelId
                        ),
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_FORWARD,
                            operand: ZkgmLib.encodeForward(
                                Forward({
                                    path: ZkgmLib.updateChannelPath(
                                        ZkgmLib.updateChannelPath(
                                            0, previousDestinationChannelId2
                                        ),
                                        nextSourceChannelId2
                                    ),
                                    timeoutHeight: type(uint64).max,
                                    timeoutTimestamp: 0,
                                    instruction: dummyCall
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            })
        );
        bytes memory ack = zkgm.doExecuteForward(
            IBCPacket({
                sourceChannelId: previousSourceChannelId,
                destinationChannelId: previousDestinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            salt,
            uint256(path),
            Forward({
                path: ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(
                        ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(
                                0, previousDestinationChannelId
                            ),
                            nextSourceChannelId
                        ),
                        previousDestinationChannelId2
                    ),
                    nextSourceChannelId2
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0,
                instruction: dummyCall
            })
        );
        assertEq(ZkgmLib.ACK_EMPTY, ack);
    }

    function test_executeForward_invalidPrecomputedChannel(
        uint32 previousSourceChannelId,
        uint32 previousDestinationChannelId,
        uint32 fakeDestinationChannelId,
        uint32 nextSourceChannelId,
        uint32 nextDestinationChannelId,
        bytes32 salt,
        uint128 path,
        address relayer,
        bytes memory relayerMsg
    ) public {
        {
            vm.assume(previousSourceChannelId != 0);
            vm.assume(previousDestinationChannelId != 0);
            vm.assume(fakeDestinationChannelId != 0);
            vm.assume(fakeDestinationChannelId != previousDestinationChannelId);
            vm.assume(nextSourceChannelId != 0);
            vm.assume(nextDestinationChannelId != 0);
        }
        handler.setChannel(nextSourceChannelId, nextDestinationChannelId);
        vm.expectRevert(ZkgmLib.ErrInvalidForwardDestinationChannelId.selector);
        zkgm.doExecuteForward(
            IBCPacket({
                sourceChannelId: previousSourceChannelId,
                destinationChannelId: previousDestinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            salt,
            uint256(path),
            Forward({
                path: ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(0, fakeDestinationChannelId),
                    nextSourceChannelId
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0,
                instruction: dummyCall
            })
        );
    }

    function test_executeForward_invalidNextSourceChannelId(
        uint32 previousSourceChannelId,
        uint32 previousDestinationChannelId,
        uint32 nextSourceChannelId,
        uint32 wrongNextSourceChannelId,
        uint32 nextDestinationChannelId,
        bytes32 salt,
        uint128 path,
        address relayer,
        bytes memory relayerMsg
    ) public {
        {
            vm.assume(previousSourceChannelId != 0);
            vm.assume(previousDestinationChannelId != 0);
            vm.assume(nextSourceChannelId != 0);
            vm.assume(wrongNextSourceChannelId != 0);
            vm.assume(wrongNextSourceChannelId != nextSourceChannelId);
            vm.assume(nextDestinationChannelId != 0);
        }
        handler.setChannel(nextSourceChannelId, nextDestinationChannelId);
        vm.expectRevert(TestIBCHandler.ErrInvalidChannel.selector);
        zkgm.doExecuteForward(
            IBCPacket({
                sourceChannelId: previousSourceChannelId,
                destinationChannelId: previousDestinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            salt,
            uint256(path),
            Forward({
                path: ZkgmLib.updateChannelPath(
                    ZkgmLib.updateChannelPath(0, previousDestinationChannelId),
                    wrongNextSourceChannelId
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0,
                instruction: dummyCall
            })
        );
    }

    function test_call_eureka_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint256 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory contractCalldata
    ) public {
        {
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        vm.expectEmit();
        emit TestCallTarget.OnZkgm(
            path,
            sourceChannelId,
            destinationChannelId,
            sender,
            contractCalldata
        );
        bytes memory ack = zkgm.doExecuteCall(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            path,
            salt,
            Call({
                sender: sender,
                eureka: false,
                contractAddress: abi.encodePacked(address(callTarget)),
                contractCalldata: contractCalldata
            })
        );
        assertEq(ack, abi.encode(ZkgmLib.ACK_SUCCESS));
    }

    function test_call_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint256 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory contractCalldata
    ) public {
        {
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        vm.expectEmit();
        emit TestCallTarget.OnRecvPacket(
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encodeCallCalldataMemory(
                    path, sender, contractCalldata
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg
        );
        bytes memory ack = zkgm.doExecuteCall(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: hex"",
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            path,
            salt,
            Call({
                sender: sender,
                eureka: true,
                contractAddress: abi.encodePacked(address(callTarget)),
                contractCalldata: contractCalldata
            })
        );
        assertEq(ack, hex"01");
    }

    function expectAckFailure(
        address caller,
        IBCPacket memory packet,
        address relayer,
        bytes memory relayerMsg,
        bool onlyMaker,
        bool intent
    ) internal {
        if (onlyMaker) {
            vm.expectRevert(ZkgmLib.ErrOnlyMaker.selector);
        }
        bytes memory ack;
        if (intent) {
            ack = zkgm.onRecvIntentPacket(caller, packet, relayer, relayerMsg);
        } else {
            ack = zkgm.onRecvPacket(caller, packet, relayer, relayerMsg);
        }
        if (!onlyMaker) {
            assertEq(
                ack,
                ZkgmLib.encodeAck(
                    Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
                )
            );
        }
    }

    function expectAckSuccess(
        address caller,
        IBCPacket memory packet,
        address relayer,
        bytes memory relayerMsg,
        bytes memory expectedAck,
        bool intent
    ) internal {
        vm.prank(address(handler));
        bytes memory ack;
        if (intent) {
            ack = zkgm.onRecvIntentPacket(caller, packet, relayer, relayerMsg);
        } else {
            ack = zkgm.onRecvPacket(caller, packet, relayer, relayerMsg);
        }
        assertEq(
            ack,
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_SUCCESS, innerAck: expectedAck})
            )
        );
    }

    function test_call_eureka_invalidContract(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        uint256 path,
        bytes memory sender,
        bytes memory contractCalldata,
        bool intent
    ) public {
        {
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        vm.prank(address(handler));
        expectAckFailure(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: path,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_CALL,
                            operand: ZkgmLib.encodeCall(
                                Call({
                                    sender: sender,
                                    eureka: false,
                                    contractAddress: abi.encodePacked(address(0)),
                                    contractCalldata: contractCalldata
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            false,
            intent
        );
    }

    function expectOnRecvOrderFailure(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        TokenOrderV1 memory order,
        bool onlyMaker,
        bool intent
    ) internal {
        vm.prank(address(handler));
        expectAckFailure(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: path,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_1,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV1(order)
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            onlyMaker,
            intent
        );
    }

    function expectOnRecvOrderProtocolFillSuccess(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        TokenOrderV1 memory order
    ) internal {
        expectOnRecvTransferSuccessCustomAck(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            order,
            TokenOrderAck({
                fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                marketMaker: ZkgmLib.ACK_EMPTY
            }),
            false
        );
    }

    function expectOnRecvOrderProtocolFillSuccessV2(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        TokenOrderV2 memory order
    ) internal {
        expectOnRecvTransferSuccessCustomAckV2(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            order,
            TokenOrderAck({
                fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                marketMaker: ZkgmLib.ACK_EMPTY
            }),
            false
        );
    }

    function expectOnRecvOrderMarketMakerFillSuccess(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        TokenOrderV1 memory order
    ) internal {
        expectOnRecvTransferSuccessCustomAck(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            order,
            TokenOrderAck({
                fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                marketMaker: relayerMsg
            }),
            false
        );
    }

    function expectOnIntentRecvOrderMarketMakerFillSuccess(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        TokenOrderV1 memory order
    ) internal {
        expectOnRecvTransferSuccessCustomAck(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            order,
            TokenOrderAck({
                fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                marketMaker: relayerMsg
            }),
            true
        );
    }

    function buildOrderPacketV2(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        TokenOrderV1 memory order
    ) internal returns (IBCPacket memory) {
        return IBCPacket({
            sourceChannelId: sourceChannelId,
            destinationChannelId: destinationChannelId,
            data: ZkgmLib.encode(
                ZkgmPacket({
                    salt: salt,
                    path: path,
                    instruction: Instruction({
                        version: ZkgmLib.INSTR_VERSION_1,
                        opcode: ZkgmLib.OP_TOKEN_ORDER,
                        operand: ZkgmLib.encodeTokenOrderV1(order)
                    })
                })
            ),
            timeoutHeight: 0,
            timeoutTimestamp: type(uint64).max
        });
    }

    function buildOrderPacketV2(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        TokenOrderV2 memory order
    ) internal returns (IBCPacket memory) {
        return IBCPacket({
            sourceChannelId: sourceChannelId,
            destinationChannelId: destinationChannelId,
            data: ZkgmLib.encode(
                ZkgmPacket({
                    salt: salt,
                    path: path,
                    instruction: Instruction({
                        version: ZkgmLib.INSTR_VERSION_2,
                        opcode: ZkgmLib.OP_TOKEN_ORDER,
                        operand: ZkgmLib.encodeTokenOrderV2(order)
                    })
                })
            ),
            timeoutHeight: 0,
            timeoutTimestamp: type(uint64).max
        });
    }

    function expectOnRecvTransferSuccessCustomAck(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        TokenOrderV1 memory order,
        TokenOrderAck memory expectedAck,
        bool intent
    ) internal {
        expectAckSuccess(
            caller,
            buildOrderPacketV2(
                sourceChannelId, destinationChannelId, path, salt, order
            ),
            relayer,
            relayerMsg,
            ZkgmLib.encodeTokenOrderAck(expectedAck),
            intent
        );
    }

    function expectOnRecvTransferSuccessCustomAckV2(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        TokenOrderV2 memory order,
        TokenOrderAck memory expectedAck,
        bool intent
    ) internal {
        expectAckSuccess(
            caller,
            buildOrderPacketV2(
                sourceChannelId, destinationChannelId, path, salt, order
            ),
            relayer,
            relayerMsg,
            ZkgmLib.encodeTokenOrderAck(expectedAck),
            intent
        );
    }

    function test_onRecvPacket_transferNative_wrap_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        // NOTE: we use u192 to avoid having the channel path being full (max u256)
        // as we need to append the destination channel in the test (leave a u32
        // slot in the u256).
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        {
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        {
            if (baseAmount > 0) {
                zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
            }
        }
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccess(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
    }

    function _metadata(
        TokenMeta memory tokenMeta
    ) internal returns (TokenMetadata memory) {
        return TokenMetadata({
            implementation: abi.encodePacked(erc20Impl),
            initializer: abi.encodeCall(
                ZkgmERC20.initialize,
                (
                    zkgm.authority(),
                    address(zkgm),
                    tokenMeta.name,
                    tokenMeta.symbol,
                    tokenMeta.decimals
                )
            )
        });
    }

    struct TokenMeta {
        string symbol;
        string name;
        uint8 decimals;
    }

    function test_onRecvPacket_transferNative_v2_wrap_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        // NOTE: we use u192 to avoid having the channel path being full (max u256)
        // as we need to append the destination channel in the test (leave a u32
        // slot in the u256).
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public {
        {
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        (address quoteToken,) = zkgm.predictWrappedTokenV2(
            path, destinationChannelId, baseToken, metadata
        );
        {
            if (baseAmount > 0) {
                zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
            }
        }
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
    }

    function test_onRecvPacket_transferNative_newWrapped(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        address receiver,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public returns (address) {
        {
            assumeUnusedAddress(receiver);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        assertFalse(ZkgmLib.isDeployed(quoteToken));
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(receiver),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenMeta.symbol,
                baseTokenName: baseTokenMeta.name,
                baseTokenDecimals: baseTokenMeta.decimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccess(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertTrue(ZkgmLib.isDeployed(quoteToken));
        assertEq(
            AccessManagedUpgradeable(quoteToken).authority(), zkgm.authority()
        );
        return quoteToken;
    }

    function test_onRecvPacket_transferNative_newWrapped_v2(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        address receiver,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public returns (address) {
        {
            assumeUnusedAddress(receiver);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        (address quoteToken,) = zkgm.predictWrappedTokenV2(
            path, destinationChannelId, baseToken, metadata
        );
        assertFalse(ZkgmLib.isDeployed(quoteToken));
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(receiver),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertTrue(ZkgmLib.isDeployed(quoteToken));
        assertEq(
            AccessManagedUpgradeable(quoteToken).authority(), zkgm.authority()
        );
        return quoteToken;
    }

    function test_onRecvPacket_transferNative_newWrapped_originSet(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        {
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        assertEq(zkgm.tokenOrigin(quoteToken), 0);
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccess(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(
            zkgm.tokenOrigin(quoteToken),
            ZkgmLib.updateChannelPath(path, destinationChannelId)
        );
    }

    function test_onRecvPacket_transferNative_v2_newWrapped_originSet(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public {
        {
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        (address quoteToken,) = zkgm.predictWrappedTokenV2(
            path, destinationChannelId, baseToken, metadata
        );
        assertEq(zkgm.tokenOrigin(quoteToken), 0);
        assertEq(zkgm.metadataImageOf(quoteToken), bytes32(0));
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(
            zkgm.tokenOrigin(quoteToken),
            ZkgmLib.updateChannelPath(path, destinationChannelId)
        );
        assertEq(
            zkgm.metadataImageOf(quoteToken),
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata))
        );
    }

    function test_onRecvPacket_transferNative_wrap_relativeSupplyChange(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        {
            vm.assume(baseAmount > 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }
        vm.expectEmit();
        emit IERC20.Transfer(address(0), address(this), baseAmount);
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccess(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(IERC20(quoteToken).totalSupply(), baseAmount);
    }

    function test_onRecvPacket_transferNative_v2_wrap_relativeSupplyChange(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public {
        {
            vm.assume(baseAmount > 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        (address quoteToken,) = zkgm.predictWrappedTokenV2(
            path, destinationChannelId, baseToken, metadata
        );
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }
        vm.expectEmit();
        emit IERC20.Transfer(address(0), address(this), baseAmount);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(IERC20(quoteToken).totalSupply(), baseAmount);
    }

    function test_onRecvPacket_transferNative_wrap_splitFee(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(quoteAmount < baseAmount);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        if (quoteAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
            vm.expectEmit();
            emit IERC20.Transfer(address(0), address(this), quoteAmount);
        }
        uint256 fee = baseAmount - quoteAmount;
        if (fee > 0) {
            vm.expectEmit();
            emit IERC20.Transfer(address(0), relayer, fee);
        }
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderProtocolFillSuccess(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
    }

    function test_onRecvPacket_transferNative_v2_wrap_splitFee(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(quoteAmount < baseAmount);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        (address quoteToken,) = zkgm.predictWrappedTokenV2(
            path, destinationChannelId, baseToken, metadata
        );
        if (quoteAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
            vm.expectEmit();
            emit IERC20.Transfer(address(0), address(this), quoteAmount);
        }
        uint256 fee = baseAmount - quoteAmount;
        if (fee > 0) {
            vm.expectEmit();
            emit IERC20.Transfer(address(0), relayer, fee);
        }
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
    }

    function test_increaseOutstanding_decreaseOutstanding_iso(
        uint32 sourceChannelId,
        uint256 path,
        address baseToken,
        bytes calldata quoteToken,
        uint256 amount
    ) public {
        assertEq(
            zkgm.channelBalanceV2(sourceChannelId, path, baseToken, quoteToken),
            0
        );
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, baseToken, quoteToken, amount
        );
        assertEq(
            zkgm.channelBalanceV2(sourceChannelId, path, baseToken, quoteToken),
            amount
        );
        zkgm.doDecreaseOutstandingV2(
            sourceChannelId, path, baseToken, quoteToken, amount
        );
        assertEq(
            zkgm.channelBalanceV2(sourceChannelId, path, baseToken, quoteToken),
            0
        );
    }

    function test_increaseOutstanding_decreaseOutstanding_v2_iso(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        bytes calldata quoteToken,
        uint256 amount
    ) public {
        assertEq(
            zkgm.channelBalanceV2(sourceChannelId, path, token, quoteToken), 0
        );
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, token, quoteToken, amount
        );
        assertEq(
            zkgm.channelBalanceV2(sourceChannelId, path, token, quoteToken),
            amount
        );
        zkgm.doDecreaseOutstandingV2(
            sourceChannelId, path, token, quoteToken, amount
        );
        assertEq(
            zkgm.channelBalanceV2(sourceChannelId, path, token, quoteToken), 0
        );
    }

    function test_onRecvPacket_transferNative_unwrap_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        {
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: ZkgmLib.reverseChannelPath(path),
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccess(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
    }

    function test_onRecvPacket_transferNative_unwrap_decreaseOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        {
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }

        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: ZkgmLib.reverseChannelPath(path),
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccess(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                destinationChannelId, path, quoteToken, baseToken
            ),
            0
        );
    }

    function test_onRecvPacket_transferNative_v2_unwrap_decreaseOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public {
        {
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        if (baseAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, baseAmount, 1, false);
        }

        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                destinationChannelId, path, quoteToken, baseToken
            ),
            0
        );
    }

    function test_onRecvPacket_transferNative_unwrap_channel_noOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint32 fakeDestinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        {
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(fakeDestinationChannelId > 0);
            vm.assume(destinationChannelId != fakeDestinationChannelId);
            vm.assume(baseAmount > 0);
        }
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        expectOnRecvOrderFailure(
            caller,
            sourceChannelId,
            fakeDestinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: ZkgmLib.reverseChannelPath(path),
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            }),
            false,
            false
        );
    }

    function test_onRecvPacket_transferNative_unwrap_path_noOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        uint192 differentPath,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        {
            vm.assume(path > 0);
            vm.assume(differentPath > 0);
            vm.assume(path != differentPath);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
        }
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        expectOnRecvOrderFailure(
            caller,
            sourceChannelId,
            destinationChannelId,
            differentPath,
            salt,
            relayer,
            relayerMsg,
            TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: ZkgmLib.reverseChannelPath(path),
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: baseAmount
            }),
            false,
            false
        );
    }

    function test_onRecvPacket_v2_transferNative_unwrap_channel_noOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint32 fakeDestinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public {
        {
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(fakeDestinationChannelId > 0);
            vm.assume(destinationChannelId != fakeDestinationChannelId);
            vm.assume(baseAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );

        vm.prank(address(handler));
        expectAckFailure(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: fakeDestinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: path,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_2,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV2(
                                TokenOrderV2({
                                    sender: sender,
                                    receiver: abi.encodePacked(address(this)),
                                    baseToken: baseToken,
                                    baseAmount: baseAmount,
                                    kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                                    metadata: abi.encodePacked(metadataImage),
                                    quoteToken: abi.encodePacked(quoteToken),
                                    quoteAmount: baseAmount
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            false,
            false
        );
    }

    function test_onRecvPacket_v2_transferNative_unwrap_path_noOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        uint192 differentPath,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public {
        {
            vm.assume(path > 0);
            vm.assume(differentPath > 0);
            vm.assume(path != differentPath);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );

        vm.prank(address(handler));
        expectAckFailure(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: differentPath,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_2,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV2(
                                TokenOrderV2({
                                    sender: sender,
                                    receiver: abi.encodePacked(address(this)),
                                    baseToken: baseToken,
                                    baseAmount: baseAmount,
                                    kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                                    metadata: abi.encodePacked(metadataImage),
                                    quoteToken: abi.encodePacked(quoteToken),
                                    quoteAmount: baseAmount
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            false,
            false
        );
    }

    function test_onRecvPacket_marketMakerFill_ok(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        address quoteToken = address(erc20);
        {
            if (quoteAmount > 0) {
                erc20.mint(marketMaker, quoteAmount);
                vm.prank(marketMaker);
                erc20.approve(address(zkgm), quoteAmount);
                zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
                vm.expectEmit();
                emit IERC20.Transfer(marketMaker, address(this), quoteAmount);
            }
        }
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderMarketMakerFillSuccess(
                marketMaker,
                sourceChannelId,
                destinationChannelId,
                0,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
    }

    function test_onRecvPacket_v2_marketMakerFill_ok(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        address quoteToken = address(erc20);
        {
            if (quoteAmount > 0) {
                erc20.mint(marketMaker, quoteAmount);
                vm.prank(marketMaker);
                erc20.approve(address(zkgm), quoteAmount);
                zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
                vm.expectEmit();
                emit IERC20.Transfer(marketMaker, address(this), quoteAmount);
            }
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvTransferSuccessCustomAckV2(
                marketMaker,
                sourceChannelId,
                destinationChannelId,
                0,
                salt,
                relayer,
                relayerMsg,
                order,
                TokenOrderAck({
                    fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                    marketMaker: relayerMsg
                }),
                false
            );
        }
    }

    function test_onRecvPacket_marketMakerFill_gasStation_ok(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        uint128 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        {
            if (quoteAmount > 0) {
                vm.deal(marketMaker, quoteAmount);
                vm.startPrank(marketMaker);
                weth.deposit{value: quoteAmount}();
                weth.approve(address(zkgm), quoteAmount);
                vm.stopPrank();
                vm.expectEmit();
                emit IERC20.Transfer(marketMaker, address(zkgm), quoteAmount);
                vm.expectEmit();
                emit IERC20.Transfer(address(zkgm), address(0), quoteAmount);
            }
        }
        assertEq(quoteAmount, weth.balanceOf(marketMaker));
        assertEq(0, address(zkgm).balance);
        uint256 selfBalance = address(this).balance;
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderMarketMakerFillSuccess(
                marketMaker,
                sourceChannelId,
                destinationChannelId,
                0,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(0, weth.balanceOf(marketMaker));
        assertEq(0, address(zkgm).balance);
        assertEq(selfBalance + quoteAmount, address(this).balance);
    }

    function test_onRecvPacket_v2_marketMakerFill_gasStation_ok(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint128 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        {
            if (quoteAmount > 0) {
                vm.deal(marketMaker, quoteAmount);
                vm.startPrank(marketMaker);
                weth.deposit{value: quoteAmount}();
                weth.approve(address(zkgm), quoteAmount);
                vm.stopPrank();
                vm.expectEmit();
                emit IERC20.Transfer(marketMaker, address(zkgm), quoteAmount);
                vm.expectEmit();
                emit IERC20.Transfer(address(zkgm), address(0), quoteAmount);
            }
        }
        assertEq(quoteAmount, weth.balanceOf(marketMaker));
        assertEq(0, address(zkgm).balance);
        uint256 selfBalance = address(this).balance;
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS),
                quoteAmount: quoteAmount
            });
            expectOnRecvTransferSuccessCustomAckV2(
                marketMaker,
                sourceChannelId,
                destinationChannelId,
                0,
                salt,
                relayer,
                relayerMsg,
                order,
                TokenOrderAck({
                    fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                    marketMaker: relayerMsg
                }),
                false
            );
        }
        assertEq(0, weth.balanceOf(marketMaker));
        assertEq(0, address(zkgm).balance);
        assertEq(selfBalance + quoteAmount, address(this).balance);
    }

    function test_onRecvPacket_marketMakerFill_noAllowance_reverts_onlyMaker(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(quoteAmount > 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        address quoteToken = address(erc20);
        zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
        expectOnRecvOrderFailure(
            marketMaker,
            sourceChannelId,
            destinationChannelId,
            0,
            salt,
            relayer,
            relayerMsg,
            TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            }),
            true,
            false
        );
    }

    function internalOnAckOrder(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        TokenOrderV1 memory order,
        bytes memory ack
    ) internal {
        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: path,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_1,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV1(order)
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ack,
            relayer
        );
    }

    function internalOnAckOrderV2(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        TokenOrderV2 memory order,
        bytes memory ack
    ) internal {
        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: path,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_2,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV2(order)
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ack,
            relayer
        );
    }

    function test_onAckPacket_onlyIBC(
        address caller,
        IBCPacket memory packet,
        address relayer,
        bytes memory ack
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onAcknowledgementPacket(caller, packet, ack, relayer);
    }

    function test_onAckPacket_transferNative_unwrap_successAck_protocolFill_noop(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        internalOnAckOrder(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: ZkgmLib.reverseChannelPath(path),
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: ZkgmLib.encodeTokenOrderAck(
                        TokenOrderAck({
                            fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                            marketMaker: ZkgmLib.ACK_EMPTY
                        })
                    )
                })
            )
        );
        (, bytes32[] memory writeSlots) = vm.accesses(address(zkgm));
        assertEq(writeSlots.length, 0);
    }

    function test_onAckPacket_transfer_successAck_marketMakerFill_unescrowAndPay(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, address(erc20), quoteToken, baseAmount
        );
        erc20.mint(address(zkgm), baseAmount);
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), relayer, baseAmount);
        internalOnAckOrder(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: abi.encodePacked(erc20),
                baseTokenPath: 0,
                baseTokenSymbol: erc20.symbol(),
                baseTokenName: erc20.name(),
                baseTokenDecimals: erc20.decimals(),
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: ZkgmLib.encodeTokenOrderAck(
                        TokenOrderAck({
                            fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                            marketMaker: abi.encodePacked(relayer)
                        })
                    )
                })
            )
        );
    }

    function test_onAckPacket_transfer_successAck_marketMakerFill_mintAndPay(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        vm.expectEmit();
        emit IERC20.Transfer(address(0), relayer, baseAmount);

        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: abi.encodePacked(erc20),
                baseTokenPath: ZkgmLib.reverseChannelPath(path),
                baseTokenSymbol: erc20.symbol(),
                baseTokenName: erc20.name(),
                baseTokenDecimals: erc20.decimals(),
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnAckOrder(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                order,
                ZkgmLib.encodeAck(
                    Ack({
                        tag: ZkgmLib.ACK_SUCCESS,
                        innerAck: ZkgmLib.encodeTokenOrderAck(
                            TokenOrderAck({
                                fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                                marketMaker: abi.encodePacked(relayer)
                            })
                        )
                    })
                )
            );
        }
    }

    function test_onAckPacket_transfer_failureAck_unescrowRefund(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        address sender,
        bytes memory receiver,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        erc20.mint(address(zkgm), baseAmount);
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, address(erc20), quoteToken, baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), sender, baseAmount);
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseTokenPath: 0,
                baseTokenSymbol: erc20.symbol(),
                baseTokenName: erc20.name(),
                baseTokenDecimals: erc20.decimals(),
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnAckOrder(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                order,
                ZkgmLib.encodeAck(
                    Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
                )
            );
        }
    }

    function test_onAckPacket_transfer_failureAck_unescrowRefund_decreaseOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        address sender,
        bytes memory receiver,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        erc20.mint(address(zkgm), baseAmount);
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, address(erc20), quoteToken, baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), sender, baseAmount);
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseTokenPath: 0,
                baseTokenSymbol: erc20.symbol(),
                baseTokenName: erc20.name(),
                baseTokenDecimals: erc20.decimals(),
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnAckOrder(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                order,
                ZkgmLib.encodeAck(
                    Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
                )
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                sourceChannelId, path, address(erc20), quoteToken
            ),
            0
        );
    }

    function test_onAckPacket_transfer_failureAck_mintRefund(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        address sender,
        bytes memory receiver,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        vm.expectEmit();
        emit IERC20.Transfer(address(0), sender, baseAmount);
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseTokenPath: path,
                baseTokenSymbol: erc20.symbol(),
                baseTokenName: erc20.name(),
                baseTokenDecimals: erc20.decimals(),
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnAckOrder(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                order,
                ZkgmLib.encodeAck(
                    Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
                )
            );
        }
    }

    function test_onAckPacket_v2_transferNative_unwrap_successAck_protocolFill_noop(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        internalOnAckOrderV2(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(
                    EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata))
                ),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: ZkgmLib.encodeTokenOrderAck(
                        TokenOrderAck({
                            fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                            marketMaker: ZkgmLib.ACK_EMPTY
                        })
                    )
                })
            )
        );
        (, bytes32[] memory writeSlots) = vm.accesses(address(zkgm));
        assertEq(writeSlots.length, 0);
    }

    function test_onAckPacket_v2_transfer_successAck_marketMakerFill_unescrowAndPay(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, address(erc20), quoteToken, baseAmount
        );
        erc20.mint(address(zkgm), baseAmount);
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), relayer, baseAmount);
        internalOnAckOrderV2(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: abi.encodePacked(erc20),
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: ZkgmLib.encodeTokenOrderAck(
                        TokenOrderAck({
                            fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                            marketMaker: abi.encodePacked(relayer)
                        })
                    )
                })
            )
        );
    }

    function test_onAckPacket_v2_transfer_successAck_marketMakerFill_mintAndPay(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        vm.expectEmit();
        emit IERC20.Transfer(address(0), relayer, baseAmount);

        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: abi.encodePacked(erc20),
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnAckOrderV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                order,
                ZkgmLib.encodeAck(
                    Ack({
                        tag: ZkgmLib.ACK_SUCCESS,
                        innerAck: ZkgmLib.encodeTokenOrderAck(
                            TokenOrderAck({
                                fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                                marketMaker: abi.encodePacked(relayer)
                            })
                        )
                    })
                )
            );
        }
    }

    function test_onAckPacket_v2_transfer_failureAck_unescrowRefund(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        address sender,
        bytes memory receiver,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        erc20.mint(address(zkgm), baseAmount);
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, address(erc20), quoteToken, baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), sender, baseAmount);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnAckOrderV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                order,
                ZkgmLib.encodeAck(
                    Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
                )
            );
        }
    }

    function test_onAckPacket_v2_transfer_failureAck_mintRefund(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        address sender,
        bytes memory receiver,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        vm.expectEmit();
        emit IERC20.Transfer(address(0), sender, baseAmount);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnAckOrderV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                order,
                ZkgmLib.encodeAck(
                    Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
                )
            );
        }
    }

    function test_onTimeout_onlyIBC(
        address caller,
        IBCPacket memory packet,
        address relayer
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onTimeoutPacket(caller, packet, relayer);
    }

    function test_onRecvIntentPacket_onlyIBC(
        address caller,
        IBCPacket memory packet,
        address relayer,
        bytes memory relayerMsg
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        zkgm.onRecvIntentPacket(caller, packet, relayer, relayerMsg);
    }

    function test_onRecvIntentPacket_marketMakerFill_ok(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        address quoteToken = address(erc20);
        if (quoteAmount > 0) {
            erc20.mint(marketMaker, quoteAmount);
            vm.prank(marketMaker);
            erc20.approve(address(zkgm), quoteAmount);
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
            vm.expectEmit();
            emit IERC20.Transfer(marketMaker, address(this), quoteAmount);
        }
        {
            TokenOrderV1 memory order = TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnIntentRecvOrderMarketMakerFillSuccess(
                marketMaker,
                sourceChannelId,
                destinationChannelId,
                0,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
    }

    function test_onRecvIntentPacket_marketMakerFill_noAllowance_reverts_onlyMaker(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(quoteAmount > 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        address quoteToken = address(erc20);
        zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
        expectOnRecvOrderFailure(
            marketMaker,
            sourceChannelId,
            destinationChannelId,
            0,
            salt,
            relayer,
            relayerMsg,
            TokenOrderV1({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseTokenPath: 0,
                baseTokenSymbol: baseTokenSymbol,
                baseTokenName: baseTokenName,
                baseTokenDecimals: baseTokenDecimals,
                baseAmount: baseAmount,
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            }),
            true,
            true
        );
    }

    function test_onRecvIntentPacket_v2_marketMakerFill_ok(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        address quoteToken = address(erc20);
        if (quoteAmount > 0) {
            erc20.mint(marketMaker, quoteAmount);
            vm.prank(marketMaker);
            erc20.approve(address(zkgm), quoteAmount);
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
            vm.expectEmit();
            emit IERC20.Transfer(marketMaker, address(this), quoteAmount);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvTransferSuccessCustomAckV2(
                marketMaker,
                sourceChannelId,
                destinationChannelId,
                0,
                salt,
                relayer,
                relayerMsg,
                order,
                TokenOrderAck({
                    fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                    marketMaker: relayerMsg
                }),
                true
            );
        }
    }

    function test_onRecvIntentPacket_v2_marketMakerFill_gasStation_ok(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint128 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        {
            if (quoteAmount > 0) {
                vm.deal(marketMaker, quoteAmount);
                vm.startPrank(marketMaker);
                weth.deposit{value: quoteAmount}();
                weth.approve(address(zkgm), quoteAmount);
                vm.stopPrank();
                vm.expectEmit();
                emit IERC20.Transfer(marketMaker, address(zkgm), quoteAmount);
                vm.expectEmit();
                emit IERC20.Transfer(address(zkgm), address(0), quoteAmount);
            }
        }
        assertEq(quoteAmount, weth.balanceOf(marketMaker));
        assertEq(0, address(zkgm).balance);
        uint256 selfBalance = address(this).balance;
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS),
                quoteAmount: quoteAmount
            });
            expectOnRecvTransferSuccessCustomAckV2(
                marketMaker,
                sourceChannelId,
                destinationChannelId,
                0,
                salt,
                relayer,
                relayerMsg,
                order,
                TokenOrderAck({
                    fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                    marketMaker: relayerMsg
                }),
                true
            );
        }
        assertEq(0, weth.balanceOf(marketMaker));
        assertEq(0, address(zkgm).balance);
        assertEq(selfBalance + quoteAmount, address(this).balance);
    }

    function test_onRecvIntentPacket_v2_marketMakerFill_noAllowance_reverts_onlyMaker(
        address marketMaker,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(marketMaker);
            vm.assume(quoteAmount > 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
        }
        address quoteToken = address(erc20);
        zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
        TokenMetadata memory metadata = _metadata(baseTokenMeta);

        vm.prank(address(handler));
        expectAckFailure(
            marketMaker,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_2,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV2(
                                TokenOrderV2({
                                    sender: sender,
                                    receiver: abi.encodePacked(address(this)),
                                    baseToken: baseToken,
                                    baseAmount: baseAmount,
                                    kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                                    metadata: ZkgmLib.encodeTokenMetadata(metadata),
                                    quoteToken: abi.encodePacked(quoteToken),
                                    quoteAmount: quoteAmount
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            true,
            true
        );
    }

    // ========== STAKING TESTS ==========

    function setupGovernanceToken(
        uint32 channelId
    ) internal returns (address) {
        // Register governance token for the channel
        GovernanceToken memory govToken = GovernanceToken({
            unwrappedToken: hex"BABE",
            metadataImage: bytes32(uint256(0x123))
        });
        zkgm.registerGovernanceToken(channelId, govToken);
        (ZkgmERC20 governanceTokenImage,) = zkgm.getGovernanceToken(channelId);
        vm.cloneAccount(
            address(new TestERC20("Governance Token", "GOV", 18)),
            address(governanceTokenImage)
        );
        return address(governanceTokenImage);
    }

    function test_verify_stake_ok(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);
        handler.setChannel(channelId, channelId);

        // Mint tokens to staker and approve
        TestERC20(governanceToken).mint(staker, amount);
        vm.prank(staker);
        TestERC20(governanceToken).approve(address(zkgm), amount);

        vm.expectEmit();
        emit IERC20.Transfer(staker, address(zkgm), amount);

        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_STAKE,
                operand: ZkgmLib.encodeStake(
                    Stake({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        sender: abi.encodePacked(staker),
                        beneficiary: beneficiary,
                        validator: validator,
                        amount: amount
                    })
                )
            })
        );

        // Verify NFT was minted and stake state is correct
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        assertEq(stakeNFT.ownerOf(tokenId), address(zkgm));

        (
            ZkgmStakeState state,
            uint32 stakeChannelId,
            bytes memory stakeValidator,
            uint256 stakeAmount,
            uint256 unstakingCompletion
        ) = zkgm.stakes(tokenId);

        assertEq(uint256(state), uint256(ZkgmStakeState.STAKING));
        assertEq(stakeChannelId, channelId);
        assertEq(stakeValidator, validator);
        assertEq(stakeAmount, amount);
        assertEq(unstakingCompletion, 0);
    }

    function test_verify_stake_invalidGovernanceToken(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint256 amount,
        address wrongToken
    ) public {
        {
            assumeUnusedAddress(staker);
            assumeUnusedAddress(wrongToken);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);

        vm.expectRevert(ZkgmLib.ErrInvalidStakeGovernanceToken.selector);
        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_STAKE,
                operand: ZkgmLib.encodeStake(
                    Stake({
                        tokenId: tokenId,
                        governanceToken: abi.encodePacked(wrongToken),
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        sender: abi.encodePacked(staker),
                        beneficiary: beneficiary,
                        validator: validator,
                        amount: amount
                    })
                )
            })
        );
    }

    function test_verify_stake_invalidWrappedToken(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint256 amount,
        bytes32 wrongMetadataImage
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
            vm.assume(wrongMetadataImage != bytes32(uint256(0x123)));
        }

        address governanceToken = setupGovernanceToken(channelId);

        vm.expectRevert(ZkgmLib.ErrInvalidStakeGovernanceToken.selector);
        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_STAKE,
                operand: ZkgmLib.encodeStake(
                    Stake({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: hex"",
                        sender: abi.encodePacked(staker),
                        beneficiary: beneficiary,
                        validator: validator,
                        amount: amount
                    })
                )
            })
        );
    }

    function test_verify_stake_cannotBeForwarded(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);

        vm.expectRevert(ZkgmLib.ErrInvalidForwardInstruction.selector);
        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_FORWARD,
                operand: ZkgmLib.encodeForward(
                    Forward({
                        path: ZkgmLib.updateChannelPath(0, channelId),
                        timeoutHeight: type(uint64).max,
                        timeoutTimestamp: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_STAKE,
                            operand: ZkgmLib.encodeStake(
                                Stake({
                                    tokenId: tokenId,
                                    governanceToken: abi.encodePacked(address(erc20)),
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    beneficiary: beneficiary,
                                    validator: validator,
                                    amount: amount
                                })
                            )
                        })
                    })
                )
            })
        );
    }

    function test_verify_unstake_ok(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);
        handler.setChannel(channelId, channelId);

        // First stake to create the NFT
        TestERC20(governanceToken).mint(staker, amount);
        vm.prank(staker);
        TestERC20(governanceToken).approve(address(zkgm), amount);

        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_STAKE,
                operand: ZkgmLib.encodeStake(
                    Stake({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        sender: abi.encodePacked(staker),
                        beneficiary: abi.encodePacked(staker),
                        validator: validator,
                        amount: amount
                    })
                )
            })
        );

        // Simulate successful staking acknowledgment
        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            address(this),
            IBCPacket({
                sourceChannelId: channelId,
                destinationChannelId: channelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_STAKE,
                            operand: ZkgmLib.encodeStake(
                                Stake({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    beneficiary: abi.encodePacked(staker),
                                    validator: validator,
                                    amount: amount
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_SUCCESS, innerAck: ZkgmLib.ACK_EMPTY})
            ),
            address(this)
        );

        // Now the NFT should be owned by staker and state should be STAKED
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        assertEq(stakeNFT.ownerOf(tokenId), staker);

        // Transfer NFT back to staker for unstaking
        vm.prank(staker);
        stakeNFT.approve(address(zkgm), tokenId);

        // Now test unstaking
        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(uint256(1)),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_UNSTAKE,
                operand: ZkgmLib.encodeUnstake(
                    Unstake({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        sender: abi.encodePacked(staker),
                        validator: validator
                    })
                )
            })
        );

        // Verify NFT is now owned by zkgm contract
        assertEq(stakeNFT.ownerOf(tokenId), address(zkgm));
    }

    function test_verify_unstake_notStakable(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);

        // Create a stake in STAKING state (not yet STAKED, so not unstakable)
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId, channelId, validator, amount, ZkgmStakeState.STAKING, 0
        );

        // Try to unstake while still in STAKING state
        vm.expectRevert(ZkgmLib.ErrStakeNotUnstakable.selector);
        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_UNSTAKE,
                operand: ZkgmLib.encodeUnstake(
                    Unstake({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        sender: abi.encodePacked(staker),
                        validator: validator
                    })
                )
            })
        );
    }

    function test_verify_withdrawStake_ok(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);
        handler.setChannel(sourceChannelId, destinationChannelId);

        // Create a stake in UNSTAKING state with completion time in the past
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            validator,
            amount,
            ZkgmStakeState.UNSTAKING,
            block.timestamp - 1
        );

        // Transfer NFT to staker first
        vm.prank(address(zkgm));
        stakeNFT.transferFrom(address(zkgm), staker, tokenId);

        vm.prank(staker);
        stakeNFT.approve(address(zkgm), tokenId);

        vm.prank(staker);
        zkgm.send(
            sourceChannelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_WITHDRAW_STAKE,
                operand: ZkgmLib.encodeWithdrawStake(
                    WithdrawStake({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        sender: abi.encodePacked(staker),
                        beneficiary: beneficiary
                    })
                )
            })
        );

        // Verify NFT is now owned by zkgm contract
        assertEq(stakeNFT.ownerOf(tokenId), address(zkgm));
    }

    function test_verify_withdrawStake_notWithdrawable(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);

        // Create a stake in STAKED state (not UNSTAKING with completion time passed)
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId, channelId, hex"C0DE", 1000, ZkgmStakeState.STAKED, 0
        );

        vm.expectRevert(ZkgmLib.ErrStakeNotWithdrawable.selector);
        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_WITHDRAW_STAKE,
                operand: ZkgmLib.encodeWithdrawStake(
                    WithdrawStake({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        sender: abi.encodePacked(staker),
                        beneficiary: beneficiary
                    })
                )
            })
        );
    }

    function test_verify_withdrawRewards_ok(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);
        handler.setChannel(channelId, channelId);

        // Create a stake in STAKED state
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId, channelId, validator, 1000, ZkgmStakeState.STAKED, 0
        );

        // Transfer NFT to staker first
        vm.prank(address(zkgm));
        stakeNFT.transferFrom(address(zkgm), staker, tokenId);

        vm.prank(staker);
        stakeNFT.approve(address(zkgm), tokenId);

        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_WITHDRAW_REWARDS,
                operand: ZkgmLib.encodeWithdrawRewards(
                    WithdrawRewards({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        validator: validator,
                        sender: abi.encodePacked(staker),
                        beneficiary: beneficiary
                    })
                )
            })
        );

        // Verify NFT is now owned by zkgm contract
        assertEq(stakeNFT.ownerOf(tokenId), address(zkgm));

        // Verify state changed to WITHDRAWING_REWARDS
        (ZkgmStakeState state,,,,) = zkgm.stakes(tokenId);
        assertEq(uint256(state), uint256(ZkgmStakeState.WITHDRAWING_REWARDS));
    }

    function test_verify_withdrawRewards_notWithdrawable(
        uint32 channelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);

        // Create a stake in UNSTAKING state (not STAKED, so rewards not withdrawable)
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId, channelId, validator, 1000, ZkgmStakeState.UNSTAKING, 0
        );

        vm.expectRevert(ZkgmLib.ErrStakingRewardNotWithdrawable.selector);
        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_WITHDRAW_REWARDS,
                operand: ZkgmLib.encodeWithdrawRewards(
                    WithdrawRewards({
                        tokenId: tokenId,
                        governanceToken: hex"BABE",
                        governanceTokenWrapped: abi.encodePacked(governanceToken),
                        validator: validator,
                        sender: abi.encodePacked(staker),
                        beneficiary: beneficiary
                    })
                )
            })
        );
    }

    function test_onAckPacket_stake_success(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        address beneficiary,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            assumeUnusedAddress(beneficiary);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);

        // Setup initial stake state
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            validator,
            amount,
            ZkgmStakeState.STAKING,
            0
        );

        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            address(this),
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_STAKE,
                            operand: ZkgmLib.encodeStake(
                                Stake({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    beneficiary: abi.encodePacked(beneficiary),
                                    validator: validator,
                                    amount: amount
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_SUCCESS, innerAck: ZkgmLib.ACK_EMPTY})
            ),
            address(this)
        );

        // Verify NFT transferred to beneficiary and state updated
        assertEq(stakeNFT.ownerOf(tokenId), address(bytes20(beneficiary)));
        (ZkgmStakeState state,,,,) = zkgm.stakes(tokenId);
        assertEq(uint256(state), uint256(ZkgmStakeState.STAKED));
    }

    function test_onAckPacket_stake_failure(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);
        TestERC20(governanceToken).mint(address(zkgm), amount);

        // Setup the stake state that would exist before failure acknowledgment
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            validator,
            amount,
            ZkgmStakeState.STAKING,
            0
        );

        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), staker, amount);

        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            address(this),
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_STAKE,
                            operand: ZkgmLib.encodeStake(
                                Stake({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    beneficiary: beneficiary,
                                    validator: validator,
                                    amount: amount
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
            ),
            address(this)
        );
    }

    function test_onAckPacket_unstake_success(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        bytes memory validator,
        uint256 amount,
        uint256 completionTime
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
            vm.assume(completionTime > block.timestamp);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);

        // Setup the stake state that would exist before unstake acknowledgment
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            validator,
            amount,
            ZkgmStakeState.STAKED,
            0
        );

        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            address(this),
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_UNSTAKE,
                            operand: ZkgmLib.encodeUnstake(
                                Unstake({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    validator: validator
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: abi.encode(completionTime)
                })
            ),
            address(this)
        );

        // Verify NFT transferred back to staker and state updated
        assertEq(stakeNFT.ownerOf(tokenId), staker);
        (ZkgmStakeState state,,,, uint256 unstakingCompletion) =
            zkgm.stakes(tokenId);
        assertEq(uint256(state), uint256(ZkgmStakeState.UNSTAKING));
        assertEq(unstakingCompletion, completionTime);
    }

    function test_onAckPacket_withdrawStake_success(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        address beneficiary,
        uint256 stakedAmount,
        uint256 withdrawnAmount
    ) public {
        {
            assumeUnusedAddress(staker);
            assumeUnusedAddress(beneficiary);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(stakedAmount > 0);
            vm.assume(withdrawnAmount > 0);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);

        // Setup stake state
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            hex"C0DE",
            stakedAmount,
            ZkgmStakeState.UNSTAKING,
            0
        );

        TestERC20(governanceToken).mint(address(zkgm), stakedAmount);

        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), beneficiary, stakedAmount);
        if (withdrawnAmount >= stakedAmount) {
            if (withdrawnAmount > stakedAmount) {
                vm.expectEmit();
                emit IERC20.Transfer(
                    address(0), beneficiary, withdrawnAmount - stakedAmount
                );
            }
        } else {
            vm.expectEmit();
            emit IERC20.Transfer(
                beneficiary, address(0), stakedAmount - withdrawnAmount
            );
        }

        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            address(this),
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_WITHDRAW_STAKE,
                            operand: ZkgmLib.encodeWithdrawStake(
                                WithdrawStake({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    beneficiary: abi.encodePacked(beneficiary)
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: ZkgmLib.encodeWithdrawStakeAck(
                        WithdrawStakeAck({amount: withdrawnAmount})
                    )
                })
            ),
            address(this)
        );

        // Verify state updated to UNSTAKED
        (ZkgmStakeState state,,,,) = zkgm.stakes(tokenId);
        assertEq(uint256(state), uint256(ZkgmStakeState.UNSTAKED));
    }

    function test_onAckPacket_withdrawRewards_success(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        address beneficiary,
        bytes memory validator,
        uint256 rewardAmount
    ) public {
        {
            assumeUnusedAddress(staker);
            assumeUnusedAddress(beneficiary);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(rewardAmount > 0);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);

        // Setup stake state
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            validator,
            rewardAmount,
            ZkgmStakeState.WITHDRAWING_REWARDS,
            0
        );

        vm.expectEmit();
        emit IERC20.Transfer(address(0), beneficiary, rewardAmount);

        vm.prank(address(handler));
        zkgm.onAcknowledgementPacket(
            address(this),
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_WITHDRAW_REWARDS,
                            operand: ZkgmLib.encodeWithdrawRewards(
                                WithdrawRewards({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    validator: validator,
                                    sender: abi.encodePacked(staker),
                                    beneficiary: abi.encodePacked(beneficiary)
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: abi.encode(rewardAmount)
                })
            ),
            address(this)
        );

        // Verify NFT transferred back to sender and state updated
        assertEq(stakeNFT.ownerOf(tokenId), staker);
        (ZkgmStakeState state,,,,) = zkgm.stakes(tokenId);
        assertEq(uint256(state), uint256(ZkgmStakeState.STAKED));
    }

    function test_onTimeoutPacket_stake(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);
        TestERC20(governanceToken).mint(address(zkgm), amount);

        // Setup the stake state that would exist before timeout
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            validator,
            amount,
            ZkgmStakeState.STAKING,
            0
        );

        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), staker, amount);

        vm.prank(address(handler));
        zkgm.onTimeoutPacket(
            address(this),
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_STAKE,
                            operand: ZkgmLib.encodeStake(
                                Stake({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    beneficiary: beneficiary,
                                    validator: validator,
                                    amount: amount
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            address(this)
        );
    }

    function test_onTimeoutPacket_unstake(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 tokenId,
        address staker,
        bytes memory validator,
        uint256 amount
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(tokenId > 0);
            vm.assume(amount > 0);
        }

        address governanceToken = setupGovernanceToken(sourceChannelId);

        // Setup stake state
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        vm.prank(address(zkgm));
        stakeNFT.mint(tokenId, address(zkgm));
        zkgm.doUpdateStake(
            tokenId,
            sourceChannelId,
            validator,
            amount,
            ZkgmStakeState.STAKED,
            0
        );

        vm.prank(address(handler));
        zkgm.onTimeoutPacket(
            address(this),
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: 0,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_0,
                            opcode: ZkgmLib.OP_UNSTAKE,
                            operand: ZkgmLib.encodeUnstake(
                                Unstake({
                                    tokenId: tokenId,
                                    governanceToken: hex"BABE",
                                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                                    sender: abi.encodePacked(staker),
                                    validator: validator
                                })
                            )
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            address(this)
        );

        // Verify NFT transferred back to staker
        assertEq(stakeNFT.ownerOf(tokenId), staker);
    }

    function test_registerGovernanceToken_ok(
        uint32 channelId,
        bytes memory unwrappedToken,
        bytes32 metadataImage
    ) public {
        vm.assume(channelId > 0);

        GovernanceToken memory govToken = GovernanceToken({
            unwrappedToken: unwrappedToken,
            metadataImage: metadataImage
        });

        zkgm.registerGovernanceToken(channelId, govToken);

        (bytes memory storedToken, bytes32 storedImage) =
            zkgm.channelGovernanceToken(channelId);
        assertEq(storedToken, unwrappedToken);
        assertEq(storedImage, metadataImage);
    }

    function test_registerGovernanceToken_alreadySet(
        uint32 channelId,
        bytes memory unwrappedToken1,
        bytes32 metadataImage1,
        bytes memory unwrappedToken2,
        bytes32 metadataImage2
    ) public {
        vm.assume(channelId > 0);
        vm.assume(unwrappedToken1.length > 0);
        vm.assume(unwrappedToken2.length > 0);

        GovernanceToken memory govToken1 = GovernanceToken({
            unwrappedToken: unwrappedToken1,
            metadataImage: metadataImage1
        });

        GovernanceToken memory govToken2 = GovernanceToken({
            unwrappedToken: unwrappedToken2,
            metadataImage: metadataImage2
        });

        zkgm.registerGovernanceToken(channelId, govToken1);

        vm.expectRevert(ZkgmLib.ErrChannelGovernanceTokenAlreadySet.selector);
        zkgm.registerGovernanceToken(channelId, govToken2);
    }

    function test_staking_batch_ok(
        uint32 channelId,
        uint256 tokenId1,
        uint256 tokenId2,
        address staker,
        bytes memory beneficiary,
        bytes memory validator,
        uint248 amount1,
        uint248 amount2
    ) public {
        {
            assumeUnusedAddress(staker);
            vm.assume(channelId > 0);
            vm.assume(tokenId1 > 0);
            vm.assume(tokenId2 > 0);
            vm.assume(tokenId1 != tokenId2);
            vm.assume(amount1 > 0);
            vm.assume(amount2 > 0);
        }

        address governanceToken = setupGovernanceToken(channelId);
        handler.setChannel(channelId, channelId);

        uint256 totalAmount = uint256(amount1) + uint256(amount2);
        TestERC20(governanceToken).mint(staker, totalAmount);
        vm.prank(staker);
        TestERC20(governanceToken).approve(address(zkgm), totalAmount);

        Instruction[] memory instructions = new Instruction[](2);
        instructions[0] = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_STAKE,
            operand: ZkgmLib.encodeStake(
                Stake({
                    tokenId: tokenId1,
                    governanceToken: hex"BABE",
                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                    sender: abi.encodePacked(staker),
                    beneficiary: beneficiary,
                    validator: validator,
                    amount: amount1
                })
            )
        });
        instructions[1] = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_STAKE,
            operand: ZkgmLib.encodeStake(
                Stake({
                    tokenId: tokenId2,
                    governanceToken: hex"BABE",
                    governanceTokenWrapped: abi.encodePacked(governanceToken),
                    sender: abi.encodePacked(staker),
                    beneficiary: beneficiary,
                    validator: validator,
                    amount: amount2
                })
            )
        });

        vm.expectEmit();
        emit IERC20.Transfer(staker, address(zkgm), amount1);
        vm.expectEmit();
        emit IERC20.Transfer(staker, address(zkgm), amount2);

        vm.prank(staker);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_BATCH,
                operand: ZkgmLib.encodeBatch(Batch({instructions: instructions}))
            })
        );

        // Verify both NFTs were minted
        ZkgmERC721 stakeNFT = zkgm.predictStakeManagerAddress();
        assertEq(stakeNFT.ownerOf(tokenId1), address(zkgm));
        assertEq(stakeNFT.ownerOf(tokenId2), address(zkgm));
    }

    function test_create_foa() public {
        TokenOrderV1 memory foa = TokenOrderV1({
            sender: abi.encodePacked("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
            receiver: abi.encodePacked(
                address(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD)
            ),
            baseToken: hex"6d756e6f",
            baseTokenPath: 0,
            baseTokenSymbol: "muno",
            baseTokenName: "muno",
            baseTokenDecimals: 6,
            baseAmount: 100,
            quoteToken: hex"16628cB81ffDA9B8470e16299eFa5F76bF45A579",
            quoteAmount: 100
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_1,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV1(foa)
        });
        console.logBytes(ZkgmLib.encodeInstruction(inst));
    }

    function test_create_foa_v2_preimage_evm() public {
        TokenMetadata memory metadata = TokenMetadata({
            implementation: abi.encodePacked(
                0x999709eB04e8A30C7aceD9fd920f7e04EE6B97bA
            ),
            initializer: abi.encodeCall(
                ZkgmERC20.initialize,
                (
                    address(0x6C1D11bE06908656D16EBFf5667F1C45372B7c89),
                    address(0x05FD55C1AbE31D3ED09A76216cA8F0372f4B2eC5),
                    "Uno",
                    "U",
                    6
                )
            )
        });
        TokenOrderV2 memory foa = TokenOrderV2({
            sender: abi.encodePacked("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
            receiver: abi.encodePacked(
                address(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD)
            ),
            baseToken: hex"6d756e6f",
            kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
            metadata: ZkgmLib.encodeTokenMetadata(metadata),
            baseAmount: 100,
            quoteToken: hex"49aCf968c7E8807B39e980b2a924E97C8ead3a22",
            quoteAmount: 100
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_2,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV2(foa)
        });
        console.log("Initializer");
        console.logBytes(metadata.initializer);
        console.log("Instruction");
        console.logBytes(ZkgmLib.encodeInstruction(inst));
    }

    function test_create_foa_v2_image_evm() public {
        TokenMetadata memory metadata = TokenMetadata({
            implementation: abi.encodePacked(
                0x999709eB04e8A30C7aceD9fd920f7e04EE6B97bA
            ),
            initializer: abi.encodeCall(
                ZkgmERC20.initialize,
                (
                    address(0x6C1D11bE06908656D16EBFf5667F1C45372B7c89),
                    address(0x05FD55C1AbE31D3ED09A76216cA8F0372f4B2eC5),
                    "Uno",
                    "U",
                    6
                )
            )
        });
        bytes32 image = EfficientHashLib.hash(
            abi.encode(metadata.implementation, metadata.initializer)
        );
        TokenOrderV2 memory foa = TokenOrderV2({
            sender: abi.encodePacked("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
            receiver: abi.encodePacked(
                address(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD)
            ),
            baseToken: hex"6d756e6f",
            kind: ZkgmLib.TOKEN_ORDER_KIND_ESCROW,
            metadata: abi.encodePacked(image),
            baseAmount: 100,
            quoteToken: hex"49aCf968c7E8807B39e980b2a924E97C8ead3a22",
            quoteAmount: 100
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_2,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV2(foa)
        });
        console.log("Image");
        console.logBytes32(image);
        console.log("Instruction");
        console.logBytes(ZkgmLib.encodeInstruction(inst));
    }

    function test_create_foa_v2_preimage_evm_u() public {
        TokenMetadata memory metadata = TokenMetadata({
            implementation: abi.encodePacked(
                0x9C968B805a625303Ad43Fce99Ae72306256FE5F9
            ),
            initializer: abi.encodeCall(
                U.initialize,
                (
                    address(0x40cDFf51aE7487e0b4A4D6e5f86eB15Fb7c1d9f4),
                    address(0x5FbE74A283f7954f10AA04C2eDf55578811aeb03),
                    "Union",
                    "U",
                    18,
                    hex"0b885dae80342524f34d46b19744e304ec88c99a"
                )
            )
        });
        TokenOrderV2 memory foa = TokenOrderV2({
            sender: abi.encodePacked("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
            receiver: abi.encodePacked(
                address(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD)
            ),
            baseToken: hex"6d756e6f",
            kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
            metadata: ZkgmLib.encodeTokenMetadata(metadata),
            baseAmount: 100,
            quoteToken: hex"ba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
            quoteAmount: 100
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_2,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV2(foa)
        });
        console.log("Image");
        console.log("Instruction");
        console.logBytes(ZkgmLib.encodeInstruction(inst));
    }

    function test_create_foa_v2_preimage_cosmwasm() public {
        // Admin of the CW20-compatible token
        string memory admin = "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2";
        // CW20 code id
        uint64 codeId = 5;
        // Note on cosmwasm the minter must be the zkgm cw20 minter
        string memory initMsg =
            "{\"init\":{\"name\":\"Uno\",\"symbol\":\"UNO\",\"decimals\":6,\"initial_balances\":[],\"mint\":{\"minter\":\"union1sctpgdvs23pxv43zclww5jdzghsfuph9rkstjegx35wjkvzv6wtqpq7xxg\",\"cap\":null},\"marketing\":null}}";
        TokenMetadata memory metadata = TokenMetadata({
            implementation: abi.encode(admin, codeId),
            initializer: bytes(initMsg)
        });
        TokenOrderV2 memory foa = TokenOrderV2({
            sender: abi.encodePacked(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD),
            receiver: abi.encodePacked(
                "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
            ),
            baseToken: hex"49aCf968c7E8807B39e980b2a924E97C8ead3a22",
            kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
            metadata: ZkgmLib.encodeTokenMetadata(metadata),
            baseAmount: 10,
            quoteToken: bytes(
                "union1uyxeud073ttss4stt92hvt4wgzzyrssqata8058305km6xp7vzgs85kpst"
            ),
            quoteAmount: 10
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_2,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV2(foa)
        });
        console.log("Initializer");
        console.logBytes(metadata.initializer);
        console.log("Instruction");
        console.log(inst.version);
        console.log(inst.opcode);
        console.logBytes(inst.operand);
    }

    function test_create_foa_v2_image_cosmwasm() public {
        // Admin of the CW20-compatible token
        string memory admin = "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2";
        // CW20 code id
        uint64 codeId = 5;
        // Note on cosmwasm the minter must be the zkgm cw20 minter
        string memory initMsg =
            "{\"init\":{\"name\":\"Uno\",\"symbol\":\"UNO\",\"decimals\":6,\"initial_balances\":[],\"mint\":{\"minter\":\"union1sctpgdvs23pxv43zclww5jdzghsfuph9rkstjegx35wjkvzv6wtqpq7xxg\",\"cap\":null},\"marketing\":null}}";
        TokenMetadata memory metadata = TokenMetadata({
            implementation: abi.encode(admin, codeId),
            initializer: bytes(initMsg)
        });
        bytes32 image = EfficientHashLib.hash(
            abi.encode(metadata.implementation, metadata.initializer)
        );
        console.log("Image:");
        console.logBytes32(image);
        TokenOrderV2 memory foa = TokenOrderV2({
            sender: abi.encodePacked(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD),
            receiver: abi.encodePacked(
                "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
            ),
            baseToken: hex"49aCf968c7E8807B39e980b2a924E97C8ead3a22",
            kind: ZkgmLib.TOKEN_ORDER_KIND_ESCROW,
            metadata: abi.encodePacked(image),
            baseAmount: 10,
            quoteToken: bytes(
                "union1uyxeud073ttss4stt92hvt4wgzzyrssqata8058305km6xp7vzgs85kpst"
            ),
            quoteAmount: 10
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_2,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV2(foa)
        });
        console.log("Instruction");
        console.log(inst.version);
        console.log(inst.opcode);
        console.logBytes(inst.operand);
    }

    function test_create_stake() public {
        Stake memory stake = Stake({
            tokenId: 1,
            governanceToken: bytes("muno"),
            // TODO: wrapped token repr here
            governanceTokenWrapped: hex"",
            sender: abi.encodePacked(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD),
            beneficiary: abi.encodePacked(
                0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD
            ),
            validator: bytes("unionvaloper1qu9x4j72r88s6ee9z6tu6enuqjvtpzujks5qk8"),
            amount: 5
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_STAKE,
            operand: ZkgmLib.encodeStake(stake)
        });
        console.log("Instruction");
        console.log(inst.version);
        console.log(inst.opcode);
        console.logBytes(inst.operand);
    }

    function test_create_unstake() public {
        Unstake memory unstake = Unstake({
            tokenId: 1,
            governanceToken: bytes("muno"),
            // TODO: wrapped token repr here
            governanceTokenWrapped: hex"",
            sender: abi.encodePacked(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD),
            validator: hex"756e696f6e76616c6f7065723161737873323935667579376a7068387038657174633272387a78676764633230793776663730"
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_UNSTAKE,
            operand: ZkgmLib.encodeUnstake(unstake)
        });
        console.log("Instruction");
        console.log(inst.version);
        console.log(inst.opcode);
        console.logBytes(inst.operand);
    }

    function test_create_withdraw_stake() public {
        WithdrawStake memory withdrawStake = WithdrawStake({
            tokenId: 1,
            governanceToken: bytes("muno"),
            // TODO: wrapped token repr here
            governanceTokenWrapped: abi.encodePacked(hex""),
            sender: abi.encodePacked(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD),
            beneficiary: abi.encodePacked(
                0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD
            )
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_WITHDRAW_STAKE,
            operand: ZkgmLib.encodeWithdrawStake(withdrawStake)
        });
        console.log("Instruction");
        console.log(inst.version);
        console.log(inst.opcode);
        console.logBytes(inst.operand);
    }

    function test_create_withdraw_rewards() public {
        WithdrawRewards memory withdrawRewards = WithdrawRewards({
            tokenId: 1,
            governanceToken: bytes("muno"),
            // TODO: wrapped token repr here
            governanceTokenWrapped: abi.encodePacked(hex""),
            validator: hex"756e696f6e76616c6f7065723161737873323935667579376a7068387038657174633272387a78676764633230793776663730",
            sender: abi.encodePacked(0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD),
            beneficiary: abi.encodePacked(
                0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD
            )
        });
        Instruction memory inst = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_WITHDRAW_REWARDS,
            operand: ZkgmLib.encodeWithdrawRewards(withdrawRewards)
        });
        console.log("Instruction");
        console.log(inst.version);
        console.log(inst.opcode);
        console.logBytes(inst.operand);
    }

    function test_batch_mixed_v1_v2_orders_ok(
        uint32 channelId,
        uint32 counterpartyChannelId,
        address caller,
        bytes memory sender,
        bytes memory receiver,
        uint128 baseAmount1,
        uint128 baseAmount2,
        bytes memory quoteToken1,
        bytes memory quoteToken2,
        uint256 quoteAmount1,
        uint256 quoteAmount2
    ) public {
        assumeUnusedAddress(caller);
        vm.assume(channelId > 0);
        vm.assume(counterpartyChannelId > 0);
        vm.assume(baseAmount1 > 0);
        vm.assume(baseAmount2 > 0);
        handler.setChannel(channelId, counterpartyChannelId);

        address baseToken = address(erc20);
        uint256 totalAmount = uint256(baseAmount1) + uint256(baseAmount2);
        erc20.mint(caller, totalAmount);
        vm.prank(caller);
        erc20.approve(address(zkgm), totalAmount);

        TokenMetadata memory metadata = _metadata(
            TokenMeta({
                symbol: erc20.symbol(),
                name: erc20.name(),
                decimals: erc20.decimals()
            })
        );

        Instruction[] memory instructions = new Instruction[](2);
        // V1 order
        instructions[0] = Instruction({
            version: ZkgmLib.INSTR_VERSION_1,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV1(
                TokenOrderV1({
                    sender: sender,
                    receiver: receiver,
                    baseToken: abi.encodePacked(baseToken),
                    baseTokenPath: 0,
                    baseTokenSymbol: erc20.symbol(),
                    baseTokenName: erc20.name(),
                    baseTokenDecimals: erc20.decimals(),
                    baseAmount: baseAmount1,
                    quoteToken: abi.encodePacked(quoteToken1),
                    quoteAmount: quoteAmount1
                })
            )
        });
        // V2 order
        instructions[1] = Instruction({
            version: ZkgmLib.INSTR_VERSION_2,
            opcode: ZkgmLib.OP_TOKEN_ORDER,
            operand: ZkgmLib.encodeTokenOrderV2(
                TokenOrderV2({
                    sender: sender,
                    receiver: receiver,
                    baseToken: abi.encodePacked(baseToken),
                    baseAmount: baseAmount2,
                    kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                    metadata: ZkgmLib.encodeTokenMetadata(metadata),
                    quoteToken: abi.encodePacked(quoteToken2),
                    quoteAmount: quoteAmount2
                })
            )
        });

        vm.expectEmit();
        emit IERC20.Transfer(caller, address(zkgm), baseAmount1);

        vm.expectEmit();
        emit IERC20.Transfer(caller, address(zkgm), baseAmount2);

        vm.prank(caller);
        zkgm.send(
            channelId,
            0,
            type(uint64).max,
            bytes32(0),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_BATCH,
                operand: ZkgmLib.encodeBatch(Batch({instructions: instructions}))
            })
        );

        assertEq(
            zkgm.channelBalanceV2(channelId, 0, baseToken, quoteToken1),
            baseAmount1
        );
        assertEq(
            zkgm.channelBalanceV2(channelId, 0, baseToken, quoteToken2),
            baseAmount2
        );
    }

    function test_v1_v2_same_token_different_metadata_compatibility(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes31 salt,
        bytes memory sender,
        address receiver,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount
    ) public {
        {
            assumeUnusedAddress(receiver);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
        }
        handler.setChannel(destinationChannelId, sourceChannelId);

        // First create a V1 wrapped token
        address v1QuoteToken = test_onRecvPacket_transferNative_newWrapped(
            caller,
            sourceChannelId,
            destinationChannelId,
            relayer,
            relayerMsg,
            path,
            salt,
            sender,
            receiver,
            baseToken,
            baseTokenMeta,
            baseAmount
        );

        // Now create a V2 wrapped token for the same base token but different metadata
        TokenMeta memory differentMeta = TokenMeta({
            symbol: string.concat(baseTokenMeta.symbol, "V2"),
            name: string.concat(baseTokenMeta.name, " V2"),
            decimals: baseTokenMeta.decimals
        });

        address v2QuoteToken = test_onRecvPacket_transferNative_newWrapped_v2(
            caller,
            sourceChannelId,
            destinationChannelId,
            relayer,
            relayerMsg,
            path,
            bytes32(uint256(uint248(salt)) + 1), // Different salt
            sender,
            receiver,
            baseToken,
            differentMeta,
            baseAmount
        );

        // Verify they are different tokens
        assertTrue(v1QuoteToken != v2QuoteToken);

        // Verify V1 token has the metadata image from _makeDefaultTokenMetadata
        TokenMetadata memory v1Metadata = _metadata(baseTokenMeta);
        bytes32 expectedV1MetadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(v1Metadata));
        assertEq(zkgm.metadataImageOf(v1QuoteToken), expectedV1MetadataImage);

        // Verify V2 token has metadata image
        TokenMetadata memory v2Metadata = _metadata(differentMeta);
        bytes32 expectedMetadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(v2Metadata));
        assertEq(zkgm.metadataImageOf(v2QuoteToken), expectedMetadataImage);

        // Verify both have same origin path
        uint256 expectedOrigin =
            ZkgmLib.updateChannelPath(path, destinationChannelId);
        assertEq(zkgm.tokenOrigin(v1QuoteToken), expectedOrigin);
        assertEq(zkgm.tokenOrigin(v2QuoteToken), expectedOrigin);
    }

    function test_v1_to_v2_upgrade_scenario(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes31 salt,
        bytes memory sender,
        address receiver,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 unwrapAmount
    ) public {
        {
            assumeUnusedAddress(receiver);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(unwrapAmount > 0);
            vm.assume(unwrapAmount <= baseAmount);
        }
        handler.setChannel(destinationChannelId, sourceChannelId);

        // Step 1: Create V1 wrapped token
        address v1QuoteToken = test_onRecvPacket_transferNative_newWrapped(
            caller,
            sourceChannelId,
            destinationChannelId,
            relayer,
            relayerMsg,
            0,
            bytes32(salt),
            sender,
            receiver,
            baseToken,
            baseTokenMeta,
            baseAmount
        );

        // Step 2: User unwraps some V1 tokens using V2 with V1 compatibility
        vm.expectEmit();
        emit IERC20.Transfer(receiver, address(0), unwrapAmount);

        vm.prank(receiver);
        zkgm.send(
            destinationChannelId,
            0,
            type(uint64).max,
            bytes32(uint256(bytes32(salt)) + 1),
            Instruction({
                version: ZkgmLib.INSTR_VERSION_2,
                opcode: ZkgmLib.OP_TOKEN_ORDER,
                operand: ZkgmLib.encodeTokenOrderV2(
                    TokenOrderV2({
                        sender: abi.encodePacked(receiver),
                        receiver: sender,
                        baseToken: abi.encodePacked(v1QuoteToken),
                        kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                        metadata: hex"",
                        baseAmount: unwrapAmount,
                        quoteToken: abi.encodePacked(baseToken),
                        quoteAmount: unwrapAmount
                    })
                )
            })
        );

        // Verify V1 token supply decreased
        assertEq(IERC20(v1QuoteToken).totalSupply(), baseAmount - unwrapAmount);

        // Verify receiver balance decreased
        assertEq(
            IERC20(v1QuoteToken).balanceOf(receiver), baseAmount - unwrapAmount
        );
    }

    function test_onRecvPacket_transferNative_v2_unwrap_baseAmountEqualQuoteAmount_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 amount
    ) public {
        {
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
            vm.assume(amount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), amount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            amount
        );
        if (amount > 0) {
            zkgm.doSetBucketConfig(quoteToken, amount, 1, false);
        }

        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: amount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: amount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                destinationChannelId, path, quoteToken, baseToken
            ),
            0
        );
    }

    function test_onRecvPacket_transferNative_v2_unwrap_baseAmountGreaterThanQuoteAmount_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
            vm.assume(baseAmount > quoteAmount);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        if (quoteAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
        }

        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                destinationChannelId, path, quoteToken, baseToken
            ),
            0
        );
    }

    function test_onRecvPacket_transferNative_v2_unwrap_baseAmountLessThanQuoteAmount_failureAck(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
            vm.assume(baseAmount < quoteAmount);
            vm.assume(baseAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));

        // Create a V2 order that should fail due to baseAmount < quoteAmount
        TokenOrderV2 memory order = TokenOrderV2({
            sender: sender,
            receiver: abi.encodePacked(address(this)),
            baseToken: baseToken,
            baseAmount: baseAmount,
            kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
            metadata: hex"",
            quoteToken: abi.encodePacked(address(erc20)),
            quoteAmount: quoteAmount
        });

        // Expect a failure acknowledgment instead of a revert
        vm.prank(address(handler));
        expectAckFailure(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: salt,
                        path: path,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_2,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV2(order)
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            true,
            false
        );
    }

    function test_onRecvPacket_transferNative_v2_unwrap_v1Token_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
            vm.assume(baseAmount >= quoteAmount);
            vm.assume(quoteAmount > 0);
        }
        bytes32 metadataImage = 0;
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        if (quoteAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
        }
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        // For V1 tokens, we use the regular channelBalance
        assertEq(
            zkgm.channelBalanceV2(
                destinationChannelId, path, quoteToken, baseToken
            ),
            0
        );
    }

    function test_onRecvPacket_transferNative_v2_unwrap_splitFee(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(relayer);
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
            vm.assume(baseAmount > quoteAmount);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        if (quoteAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
        }

        uint256 fee = baseAmount - quoteAmount;
        if (quoteAmount > 0) {
            vm.expectEmit();
            emit IERC20.Transfer(address(zkgm), address(this), quoteAmount);
        }
        if (fee > 0) {
            vm.expectEmit();
            emit IERC20.Transfer(address(zkgm), relayer, fee);
        }

        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                destinationChannelId, path, quoteToken, baseToken
            ),
            0
        );
    }

    function test_onRecvPacket_transferNative_v2_unwrap_gasStation_ok(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        uint192 path,
        bytes32 salt,
        bytes memory sender,
        bytes memory baseToken,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        uint128 quoteAmount
    ) public {
        {
            assumeUnusedAddress(caller);
            assumeUnusedAddress(relayer);
            vm.assume(path != 0);
            vm.assume(sourceChannelId != 0);
            vm.assume(destinationChannelId != 0);
            vm.assume(baseAmount >= quoteAmount);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        // Fake an increase of outstanding balance as if we transferred out.
        vm.deal(address(this), baseAmount);
        weth.deposit{value: baseAmount}();
        weth.transfer(address(zkgm), baseAmount);
        address quoteToken = ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS;
        zkgm.doIncreaseOutstandingV2(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        if (quoteAmount > 0) {
            zkgm.doSetBucketConfig(quoteToken, quoteAmount, 1, false);
        }

        uint256 selfBalance = address(this).balance;
        uint256 fee = baseAmount - quoteAmount;

        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: sender,
                receiver: abi.encodePacked(address(this)),
                baseToken: baseToken,
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            expectOnRecvOrderProtocolFillSuccessV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                salt,
                relayer,
                relayerMsg,
                order
            );
        }
        assertEq(address(this).balance, selfBalance + quoteAmount);
        assertEq(
            zkgm.channelBalanceV2(
                destinationChannelId, path, quoteToken, baseToken
            ),
            0
        );
    }

    function internalOnTimeoutOrderV2(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        address relayer,
        TokenOrderV2 memory order
    ) internal {
        vm.prank(address(handler));
        zkgm.onTimeoutPacket(
            caller,
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encode(
                    ZkgmPacket({
                        salt: bytes32(0),
                        path: path,
                        instruction: Instruction({
                            version: ZkgmLib.INSTR_VERSION_2,
                            opcode: ZkgmLib.OP_TOKEN_ORDER,
                            operand: ZkgmLib.encodeTokenOrderV2(order)
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer
        );
    }

    function test_onTimeoutPacket_v2_transfer_unescrowRefund(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        uint192 path,
        address sender,
        bytes memory receiver,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        erc20.mint(address(zkgm), baseAmount);
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, address(erc20), quoteToken, baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), sender, baseAmount);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnTimeoutOrderV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                relayer,
                order
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                sourceChannelId,
                path,
                address(erc20),
                abi.encodePacked(quoteToken)
            ),
            0
        );
    }

    function test_onTimeoutPacket_v2_transfer_mintRefund(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        uint192 path,
        address sender,
        bytes memory receiver,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        vm.expectEmit();
        emit IERC20.Transfer(address(0), sender, baseAmount);
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_UNESCROW,
                metadata: abi.encodePacked(metadataImage),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnTimeoutOrderV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                relayer,
                order
            );
        }
    }

    function test_onTimeoutPacket_v2_transfer_decreaseOutstanding(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        uint192 path,
        address sender,
        bytes memory receiver,
        TokenMeta memory baseTokenMeta,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) public {
        {
            assumeUnusedAddress(sender);
            assumeUnusedAddress(relayer);
            vm.assume(path > 0);
            vm.assume(sourceChannelId > 0);
            vm.assume(destinationChannelId > 0);
            vm.assume(baseAmount > 0);
            vm.assume(quoteAmount > 0);
        }
        TokenMetadata memory metadata = _metadata(baseTokenMeta);
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        erc20.mint(address(zkgm), baseAmount);
        zkgm.doIncreaseOutstandingV2(
            sourceChannelId, path, address(erc20), quoteToken, baseAmount
        );
        assertEq(
            zkgm.channelBalanceV2(
                sourceChannelId,
                path,
                address(erc20),
                abi.encodePacked(quoteToken)
            ),
            baseAmount
        );
        {
            TokenOrderV2 memory order = TokenOrderV2({
                sender: abi.encodePacked(sender),
                receiver: receiver,
                baseToken: abi.encodePacked(erc20),
                baseAmount: baseAmount,
                kind: ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE,
                metadata: ZkgmLib.encodeTokenMetadata(metadata),
                quoteToken: abi.encodePacked(quoteToken),
                quoteAmount: quoteAmount
            });
            internalOnTimeoutOrderV2(
                caller,
                sourceChannelId,
                destinationChannelId,
                path,
                relayer,
                order
            );
        }
        assertEq(
            zkgm.channelBalanceV2(
                sourceChannelId,
                path,
                address(erc20),
                abi.encodePacked(quoteToken)
            ),
            0
        );
    }
}

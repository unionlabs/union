pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "solady/utils/LibBytes.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import "../../../contracts/core/Types.sol";
import "../../../contracts/core/25-handler/IBCHandler.sol";
import "../../../contracts/core/04-channel/IBCPacket.sol";
import "../../../contracts/core/05-port/IIBCModule.sol";
import "../../../contracts/apps/ucs/03-zkgm/Zkgm.sol";
import "../../../contracts/apps/Base.sol";

contract TestZkgm is UCS03Zkgm {
    function doExecuteForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        Forward calldata forward
    ) public returns (bytes memory) {
        return executeForward(
            ibcPacket,
            relayer,
            relayerMsg,
            salt,
            path,
            ZkgmLib.INSTR_VERSION_0,
            forward
        );
    }

    function doExecuteMultiplex(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        bytes32 salt,
        Multiplex calldata multiplex
    ) public returns (bytes memory) {
        return executeMultiplex(
            caller, ibcPacket, relayer, relayerMsg, path, salt, multiplex
        );
    }

    function doVerify(
        uint32 channelId,
        uint256 path,
        Instruction calldata instruction
    ) public {
        verifyInternal(channelId, path, instruction);
    }

    function doIncreaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) public {
        increaseOutstanding(sourceChannelId, path, token, amount);
    }

    function doDecreaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) public {
        decreaseOutstanding(sourceChannelId, path, token, amount);
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
}

contract TestMultiplexTarget is IEurekaModule, IIBCModuleRecv {
    error ErrNotZkgm();

    event OnZkgm(
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes sender,
        bytes message
    );
    event OnRecvPacket(IBCPacket packet, address relayer, bytes relayerMsg);

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

    function onRecvPacket(
        address caller,
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) public onlyZkgm returns (bytes memory) {
        emit OnRecvPacket(packet, relayer, relayerMsg);
        return hex"01";
    }
}

contract ZkgmTests is Test {
    TestMultiplexTarget multiplexTarget;
    TestIBCHandler handler;
    TestERC20 erc20;
    TestZkgm zkgm;

    Instruction dummyMultiplex = Instruction({
        version: ZkgmLib.INSTR_VERSION_0,
        opcode: ZkgmLib.OP_MULTIPLEX,
        operand: ZkgmLib.encodeMultiplex(
            Multiplex({
                sender: abi.encodePacked(address(0)),
                eureka: false,
                contractAddress: abi.encodePacked(address(0)),
                contractCalldata: hex""
            })
        )
    });

    function setUp() public {
        erc20 = new TestERC20("Test", "T", 18);
        handler = new TestIBCHandler();
        TestZkgm implementation = new TestZkgm();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeCall(UCS03Zkgm.initialize, (handler, address(this)))
        );
        zkgm = TestZkgm(address(proxy));
        multiplexTarget = new TestMultiplexTarget(address(zkgm));
    }

    function test_proxyInitialization_ok(
        address handlerAddress,
        address ownerAddress
    ) public {
        vm.assume(handlerAddress != address(0));
        vm.assume(ownerAddress != address(0));
        TestZkgm implementation = new TestZkgm();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeCall(
                UCS03Zkgm.initialize,
                (IIBCModulePacket(handlerAddress), ownerAddress)
            )
        );
        TestZkgm _zkgm = TestZkgm(address(proxy));

        assertEq(address(_zkgm.ibcHandler()), handlerAddress);
        assertEq(_zkgm.owner(), ownerAddress);
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

    function test_reverseChannelPath_iso(
        uint256 path
    ) public {
        assertEq(
            ZkgmLib.reverseChannelPath(ZkgmLib.reverseChannelPath(path)), path
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
        vm.assume(
            salt
                < 0xffff000000000000000000000000000000000000000000000000000000000000
        );
        assertFalse(ZkgmLib.isForwardedPacket(salt));
        assertTrue(ZkgmLib.isForwardedPacket(ZkgmLib.tintForwardSalt(salt)));
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
        string calldata version
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
        vm.assume(channelId != 0);
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
        string calldata version,
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
        string calldata counterpartyVersion,
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
        bytes calldata relayerMsg
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
            relayerMsg
        );
    }

    function test_verify_forward_ok() public {
        handler.setChannel(1, 10);
        zkgm.doVerify(
            1,
            0,
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
                            opcode: ZkgmLib.OP_MULTIPLEX,
                            operand: ZkgmLib.encodeMultiplex(
                                Multiplex({
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

    function test_verify_forward_invalidVersion(
        uint32 channelId,
        uint8 version
    ) public {
        vm.assume(version != ZkgmLib.INSTR_VERSION_0);
        vm.expectRevert(ZkgmLib.ErrUnsupportedVersion.selector);
        zkgm.doVerify(
            channelId,
            0,
            Instruction({
                version: version,
                opcode: ZkgmLib.OP_FORWARD,
                operand: ZkgmLib.encodeForward(
                    Forward({
                        path: ZkgmLib.updateChannelPath(
                            ZkgmLib.updateChannelPath(0, 10), 1
                        ),
                        timeoutHeight: type(uint64).max,
                        timeoutTimestamp: 0,
                        instruction: dummyMultiplex
                    })
                )
            })
        );
    }

    function test_verify_multiplex_ok(
        uint32 channelId,
        bytes memory contractAddress,
        bytes memory contractCalldata
    ) public {
        zkgm.doVerify(
            channelId,
            0,
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_MULTIPLEX,
                operand: ZkgmLib.encodeMultiplex(
                    Multiplex({
                        sender: abi.encodePacked(address(this)),
                        eureka: false,
                        contractAddress: contractAddress,
                        contractCalldata: contractCalldata
                    })
                )
            })
        );
    }

    function test_verify_multiplex_ko(
        uint32 channelId,
        address sender,
        bytes memory contractAddress,
        bytes memory contractCalldata
    ) public {
        vm.assume(sender != address(this));
        vm.expectRevert(ZkgmLib.ErrInvalidMultiplexSender.selector);
        zkgm.doVerify(
            channelId,
            0,
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_MULTIPLEX,
                operand: ZkgmLib.encodeMultiplex(
                    Multiplex({
                        sender: abi.encodePacked(sender),
                        eureka: false,
                        contractAddress: contractAddress,
                        contractCalldata: contractCalldata
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
        vm.assume(previousSourceChannelId != 0);
        vm.assume(previousDestinationChannelId != 0);
        vm.assume(nextSourceChannelId != 0);
        vm.assume(nextDestinationChannelId != 0);
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
                        instruction: dummyMultiplex
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
                instruction: dummyMultiplex
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
        vm.assume(previousSourceChannelId != 0);
        vm.assume(previousDestinationChannelId != 0);
        vm.assume(nextSourceChannelId != 0);
        vm.assume(nextDestinationChannelId != 0);
        vm.assume(previousDestinationChannelId2 != 0);
        vm.assume(nextSourceChannelId2 != 0);
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
                                    instruction: dummyMultiplex
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
                instruction: dummyMultiplex
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
        vm.assume(previousSourceChannelId != 0);
        vm.assume(previousDestinationChannelId != 0);
        vm.assume(fakeDestinationChannelId != 0);
        vm.assume(fakeDestinationChannelId != previousDestinationChannelId);
        vm.assume(nextSourceChannelId != 0);
        vm.assume(nextDestinationChannelId != 0);
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
                instruction: dummyMultiplex
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
        vm.assume(previousSourceChannelId != 0);
        vm.assume(previousDestinationChannelId != 0);
        vm.assume(nextSourceChannelId != 0);
        vm.assume(wrongNextSourceChannelId != 0);
        vm.assume(wrongNextSourceChannelId != nextSourceChannelId);
        vm.assume(nextDestinationChannelId != 0);
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
                instruction: dummyMultiplex
            })
        );
    }

    function test_multiplex_eureka_ok(
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
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        vm.expectEmit();
        emit TestMultiplexTarget.OnZkgm(
            path,
            sourceChannelId,
            destinationChannelId,
            sender,
            contractCalldata
        );
        bytes memory ack = zkgm.doExecuteMultiplex(
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
            Multiplex({
                sender: sender,
                eureka: false,
                contractAddress: abi.encodePacked(address(multiplexTarget)),
                contractCalldata: contractCalldata
            })
        );
        assertEq(ack, abi.encode(ZkgmLib.ACK_SUCCESS));
    }

    function test_multiplex_ok(
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
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        vm.expectEmit();
        emit TestMultiplexTarget.OnRecvPacket(
            IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: destinationChannelId,
                data: ZkgmLib.encodeMultiplexCalldataMemory(
                    path, sender, contractCalldata
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg
        );
        bytes memory ack = zkgm.doExecuteMultiplex(
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
            Multiplex({
                sender: sender,
                eureka: true,
                contractAddress: abi.encodePacked(address(multiplexTarget)),
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
        bool onlyMaker
    ) internal {
        if (onlyMaker) {
            vm.expectRevert(ZkgmLib.ErrOnlyMaker.selector);
        }
        bytes memory ack =
            zkgm.onRecvPacket(caller, packet, relayer, relayerMsg);
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
        bytes memory expectedAck
    ) internal {
        vm.prank(address(handler));
        assertEq(
            zkgm.onRecvPacket(caller, packet, relayer, relayerMsg),
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_SUCCESS, innerAck: expectedAck})
            )
        );
    }

    function test_multiplex_eureka_invalidContract(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address relayer,
        bytes memory relayerMsg,
        bytes32 salt,
        uint256 path,
        bytes memory sender,
        bytes memory contractCalldata
    ) public {
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
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
                            opcode: ZkgmLib.OP_MULTIPLEX,
                            operand: ZkgmLib.encodeMultiplex(
                                Multiplex({
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
            false
        );
    }

    function expectOnRecvTransferFailure(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        FungibleAssetOrder memory order,
        bool onlyMaker
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
                            opcode: ZkgmLib.OP_FUNGIBLE_ASSET_ORDER,
                            operand: ZkgmLib.encodeFungibleAssetOrder(order)
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            onlyMaker
        );
    }

    function expectOnRecvTransferSuccess(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        FungibleAssetOrder memory order
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
            FungibleAssetOrderAck({
                fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                marketMaker: ZkgmLib.ACK_EMPTY
            })
        );
    }

    function expectOnRecvTransferSuccessCustomAck(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        bytes memory relayerMsg,
        FungibleAssetOrder memory order,
        FungibleAssetOrderAck memory expectedAck
    ) internal {
        expectAckSuccess(
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
                            opcode: ZkgmLib.OP_FUNGIBLE_ASSET_ORDER,
                            operand: ZkgmLib.encodeFungibleAssetOrder(order)
                        })
                    })
                ),
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: 0
            }),
            relayer,
            relayerMsg,
            ZkgmLib.encodeFungibleAssetOrderAck(expectedAck)
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
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        expectOnRecvTransferSuccess(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            })
        );
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
        bytes memory baseToken,
        string memory baseTokenSymbol,
        string memory baseTokenName,
        uint8 baseTokenDecimals,
        uint256 baseAmount
    ) public {
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        assertFalse(ZkgmLib.isDeployed(quoteToken));
        expectOnRecvTransferSuccess(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            })
        );
        assertTrue(ZkgmLib.isDeployed(quoteToken));
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
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        assertEq(zkgm.tokenOrigin(quoteToken), 0);
        expectOnRecvTransferSuccess(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            })
        );
        assertEq(
            zkgm.tokenOrigin(quoteToken),
            ZkgmLib.updateChannelPath(path, destinationChannelId)
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
        vm.assume(baseAmount > 0);
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        vm.expectEmit();
        emit IERC20.Transfer(address(0), address(this), baseAmount);
        expectOnRecvTransferSuccess(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            })
        );
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
        vm.assume(relayer != address(0));
        vm.assume(quoteAmount < baseAmount);
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        (address quoteToken,) =
            zkgm.predictWrappedToken(path, destinationChannelId, baseToken);
        if (quoteAmount > 0) {
            vm.expectEmit();
            emit IERC20.Transfer(address(0), address(this), quoteAmount);
        }
        uint256 fee = baseAmount - quoteAmount;
        if (fee > 0) {
            vm.expectEmit();
            emit IERC20.Transfer(address(0), relayer, fee);
        }
        expectOnRecvTransferSuccess(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            })
        );
    }

    function test_increaseOutstanding_decreaseOutstanding_iso(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) public {
        assertEq(zkgm.channelBalance(sourceChannelId, path, token), 0);
        zkgm.doIncreaseOutstanding(sourceChannelId, path, token, amount);
        assertEq(zkgm.channelBalance(sourceChannelId, path, token), amount);
        zkgm.doDecreaseOutstanding(sourceChannelId, path, token, amount);
        assertEq(zkgm.channelBalance(sourceChannelId, path, token), 0);
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
        vm.assume(path != 0);
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstanding(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseAmount
        );
        expectOnRecvTransferSuccess(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            })
        );
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
        vm.assume(path != 0);
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstanding(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseAmount
        );
        expectOnRecvTransferSuccess(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            })
        );
        assertEq(zkgm.channelBalance(destinationChannelId, path, quoteToken), 0);
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
        vm.assume(path > 0);
        vm.assume(sourceChannelId > 0);
        vm.assume(destinationChannelId > 0);
        vm.assume(fakeDestinationChannelId > 0);
        vm.assume(destinationChannelId != fakeDestinationChannelId);
        vm.assume(baseAmount > 0);
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstanding(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseAmount
        );
        expectOnRecvTransferFailure(
            caller,
            sourceChannelId,
            fakeDestinationChannelId,
            path,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
        vm.assume(path > 0);
        vm.assume(differentPath > 0);
        vm.assume(path != differentPath);
        vm.assume(sourceChannelId > 0);
        vm.assume(destinationChannelId > 0);
        vm.assume(baseAmount > 0);
        // Fake an increase of outstanding balance as if we transferred out.
        erc20.mint(address(zkgm), baseAmount);
        address quoteToken = address(erc20);
        zkgm.doIncreaseOutstanding(
            destinationChannelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseAmount
        );
        expectOnRecvTransferFailure(
            caller,
            sourceChannelId,
            destinationChannelId,
            differentPath,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
        vm.assume(marketMaker != address(0));
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        if (quoteAmount > 0) {
            erc20.mint(marketMaker, quoteAmount);
            vm.prank(marketMaker);
            erc20.approve(address(zkgm), quoteAmount);
            vm.expectEmit();
            emit IERC20.Transfer(marketMaker, address(this), quoteAmount);
        }
        address quoteToken = address(erc20);
        expectOnRecvTransferSuccessCustomAck(
            marketMaker,
            sourceChannelId,
            destinationChannelId,
            0,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            FungibleAssetOrderAck({
                fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                marketMaker: relayerMsg
            })
        );
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
        vm.assume(quoteAmount > 0);
        vm.assume(marketMaker != address(0));
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        address quoteToken = address(erc20);
        expectOnRecvTransferFailure(
            marketMaker,
            sourceChannelId,
            destinationChannelId,
            0,
            salt,
            relayer,
            relayerMsg,
            FungibleAssetOrder({
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
            true
        );
    }

    function internalOnAckOrder(
        address caller,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        uint256 path,
        bytes32 salt,
        address relayer,
        FungibleAssetOrder memory order,
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
                            opcode: ZkgmLib.OP_FUNGIBLE_ASSET_ORDER,
                            operand: ZkgmLib.encodeFungibleAssetOrder(order)
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
        vm.assume(path != 0);
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        internalOnAckOrder(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            FungibleAssetOrder({
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
                    innerAck: ZkgmLib.encodeFungibleAssetOrderAck(
                        FungibleAssetOrderAck({
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
        vm.assume(relayer != address(0));
        vm.assume(path > 0);
        vm.assume(sourceChannelId > 0);
        vm.assume(destinationChannelId > 0);
        vm.assume(baseAmount > 0);
        vm.assume(quoteAmount > 0);
        zkgm.doIncreaseOutstanding(
            sourceChannelId, path, address(erc20), baseAmount
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
            FungibleAssetOrder({
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
                    innerAck: ZkgmLib.encodeFungibleAssetOrderAck(
                        FungibleAssetOrderAck({
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
        vm.assume(relayer != address(0));
        vm.assume(path > 0);
        vm.assume(sourceChannelId > 0);
        vm.assume(destinationChannelId > 0);
        vm.assume(baseAmount > 0);
        vm.assume(quoteAmount > 0);
        vm.expectEmit();
        emit IERC20.Transfer(address(0), relayer, baseAmount);
        internalOnAckOrder(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            FungibleAssetOrder({
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
            }),
            ZkgmLib.encodeAck(
                Ack({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: ZkgmLib.encodeFungibleAssetOrderAck(
                        FungibleAssetOrderAck({
                            fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                            marketMaker: abi.encodePacked(relayer)
                        })
                    )
                })
            )
        );
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
        vm.assume(sender != address(0));
        vm.assume(relayer != address(0));
        vm.assume(path > 0);
        vm.assume(sourceChannelId > 0);
        vm.assume(destinationChannelId > 0);
        vm.assume(baseAmount > 0);
        vm.assume(quoteAmount > 0);
        erc20.mint(address(zkgm), baseAmount);
        zkgm.doIncreaseOutstanding(
            sourceChannelId, path, address(erc20), baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), sender, baseAmount);
        internalOnAckOrder(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            FungibleAssetOrder({
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
            }),
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
            )
        );
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
        vm.assume(sender != address(0));
        vm.assume(relayer != address(0));
        vm.assume(path > 0);
        vm.assume(sourceChannelId > 0);
        vm.assume(destinationChannelId > 0);
        vm.assume(baseAmount > 0);
        vm.assume(quoteAmount > 0);
        erc20.mint(address(zkgm), baseAmount);
        zkgm.doIncreaseOutstanding(
            sourceChannelId, path, address(erc20), baseAmount
        );
        vm.expectEmit();
        emit IERC20.Transfer(address(zkgm), sender, baseAmount);
        internalOnAckOrder(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            FungibleAssetOrder({
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
            }),
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
            )
        );
        assertEq(zkgm.channelBalance(sourceChannelId, path, address(erc20)), 0);
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
        vm.assume(sender != address(0));
        vm.assume(relayer != address(0));
        vm.assume(path > 0);
        vm.assume(sourceChannelId > 0);
        vm.assume(destinationChannelId > 0);
        vm.assume(baseAmount > 0);
        vm.assume(quoteAmount > 0);
        vm.expectEmit();
        emit IERC20.Transfer(address(0), sender, baseAmount);
        internalOnAckOrder(
            caller,
            sourceChannelId,
            destinationChannelId,
            path,
            salt,
            relayer,
            FungibleAssetOrder({
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
            }),
            ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
            )
        );
    }
}

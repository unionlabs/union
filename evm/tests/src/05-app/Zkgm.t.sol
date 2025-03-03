pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "solady/utils/LibBytes.sol";

import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/token/ERC20/ERC20.sol";

import "../../../contracts/core/Types.sol";
import "../../../contracts/core/25-handler/IBCHandler.sol";
import "../../../contracts/core/04-channel/IBCPacket.sol";
import "../../../contracts/apps/ucs/03-zkgm/Zkgm.sol";

contract TestIBCHandler is IIBCModulePacket {
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
        acks[IBCPacketLib.commitPacketMemory(packet)] = hex"01";
        return packet;
    }

    function writeAcknowledgement(
        IBCPacket calldata packet,
        bytes memory acknowledgement
    ) external override {
        bytes32 commitmentKey = IBCPacketLib.commitPacketMemory(packet);
        if (!acks[commitmentKey].eq(hex"01")) {
            revert ErrUnknownPacket();
        }
        acks[commitmentKey] = acknowledgement;
    }
}

contract TestWETH is ERC20 {
    error ETHTransferFailed();

    constructor() ERC20("Wrapped Ether", "WETH") {}

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

contract ZkgmTests is Test {
    TestWETH weth;
    UCS03Zkgm zkgm;

    function setUp() public {
        weth = new TestWETH();
        UCS03Zkgm implementation = new UCS03Zkgm();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                UCS03Zkgm.initialize.selector,
                IIBCPacket(address(this)),
                address(this),
                address(weth)
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

    function test_proxyInitialization_ok(
        address handler,
        address owner,
        address wethAddress
    ) public {
        vm.assume(handler != address(0));
        vm.assume(owner != address(0));
        vm.assume(wethAddress != address(0));
        UCS03Zkgm implementation = new UCS03Zkgm();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                UCS03Zkgm.initialize.selector, handler, owner, wethAddress
            )
        );
        UCS03Zkgm _zkgm = UCS03Zkgm(address(proxy));

        assertEq(address(_zkgm.ibcHandler()), handler);
        assertEq(_zkgm.owner(), owner);
        assertEq(address(_zkgm.weth()), wethAddress);
    }
}

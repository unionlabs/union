// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/apps/ucs/02-nft/NFT.sol";
import "@openzeppelin/token/ERC721/ERC721.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";

contract MockIBCHandler {
    function sendPacket(
        uint32 sourceChannel,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external returns (IBCPacket memory packet) {
        return IBCPacket({
            sourceChannel: sourceChannel,
            destinationChannel: 0,
            data: bytes(""),
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
    }
}

// Mock ERC721 contract
contract MockERC721 is ERC721 {
    constructor(
        string memory name,
        string memory symbol
    ) ERC721(name, symbol) {}

    function mint(address to, uint256 tokenId) external {
        _mint(to, tokenId);
    }

    function burn(
        uint256 tokenId
    ) external {
        _burn(tokenId);
    }
}

contract UCS02NFTTests is Test {
    UCS02NFT ucs02NFT;
    MockIBCHandler mockIBCHandler;
    address admin = address(0xABcD);
    address user = address(0x1234);
    address relayer = address(0x5678);
    address randomUser = address(0x9999);

    error StringsInsufficientHexLength(uint256 value, uint256 length);

    bytes16 private constant HEX_DIGITS = "0123456789abcdef";

    event mymsg(string data, uint256 data1);

    function toHexString(
        uint256 value,
        uint256 length
    ) internal pure returns (string memory) {
        uint256 localValue = value;
        bytes memory buffer = new bytes(2 * length + 2);
        buffer[0] = "0";
        buffer[1] = "x";
        for (uint256 i = 2 * length + 1; i > 1; --i) {
            buffer[i] = HEX_DIGITS[localValue & 0xf];
            localValue >>= 4;
        }
        if (localValue != 0) {
            revert StringsInsufficientHexLength(value, length);
        }
        return string(buffer);
    }

    function toHexString(
        address addr
    ) internal pure returns (string memory) {
        return toHexString(uint256(uint160(addr)), 20);
    }

    function setUp() public {
        // Deploy the mock IBC handler
        mockIBCHandler = new MockIBCHandler();

        // Deploy the UCS02NFT implementation
        UCS02NFT implementation = new UCS02NFT();

        // Deploy the proxy and initialize it with the implementation
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                UCS02NFT.initialize.selector,
                IIBCPacket(address(mockIBCHandler)),
                admin
            )
        );

        // Cast proxy to UCS02NFT to interact with it
        ucs02NFT = UCS02NFT(address(proxy));
    }

    function test_initialize() public {
        assertEq(
            ucs02NFT.ibcAddress(),
            address(mockIBCHandler),
            "IBC Handler mismatch"
        );
        assertEq(ucs02NFT.owner(), admin, "Admin mismatch");
    }

    function test_sendLocalNative() public {
        MockERC721 mockERC721 = new MockERC721("TestNFT", "TNFT");

        // Mint tokens for the user
        mockERC721.mint(user, 1);
        mockERC721.mint(user, 2);

        uint256[] memory tokenIds = new uint256[](2);
        tokenIds[0] = 1;
        tokenIds[1] = 2;

        // Approve and send NFTs
        vm.startPrank(user);
        mockERC721.setApprovalForAll(address(ucs02NFT), true);
        ucs02NFT.send(1, "receiver_address", address(mockERC721), tokenIds, 0);
        vm.stopPrank();

        // Verify tokens are transferred to the UCS02NFT contract
        assertEq(
            mockERC721.ownerOf(1), address(ucs02NFT), "Token 1 not transferred"
        );
        assertEq(
            mockERC721.ownerOf(2), address(ucs02NFT), "Token 2 not transferred"
        );
    }

    function test_sendRemoteNative() public {
        MockERC721 mockERC721 = new MockERC721("TestNFT", "TNFT");

        // Mint tokens for the user
        mockERC721.mint(user, 1);
        mockERC721.mint(user, 2);

        uint256[] memory tokenIds = new uint256[](2);
        tokenIds[0] = 1;
        tokenIds[1] = 2;

        string[] memory tokenUris = new string[](2);
        tokenUris[0] = "ipfs://mockURI1";
        tokenUris[1] = "ipfs://mockURI2";
        // Set up remote-native scenario
        vm.startPrank(address(mockIBCHandler));
        ucs02NFT.onRecvPacket(
            IBCPacket({
                sourceChannel: 1,
                destinationChannel: 1,
                data: abi.encode(
                    NFTPacket({
                        classOwner: "",
                        classId: "remoteNFT",
                        className: "TestNFT",
                        classSymbol: "TNFT",
                        tokenIds: tokenIds,
                        tokenUris: tokenUris,
                        sender: "sender_address",
                        receiver: toHexString(user),
                        memo: ""
                    })
                ),
                timeoutHeight: 0,
                timeoutTimestamp: 0
            }),
            relayer,
            bytes("")
        );
        vm.stopPrank();

        // Verify tokens are minted in the new NFT class
        assertEq(mockERC721.ownerOf(1), user, "Token 1 not minted correctly");
        assertEq(mockERC721.ownerOf(2), user, "Token 2 not minted correctly");
    }

    function test_onAcknowledgementPacket_refund_remote_native() public {
        MockERC721 mockERC721 = new MockERC721("TestNFT", "TNFT");

        // Simulate a received remote token

        uint256[] memory tokenIds = new uint256[](2);
        tokenIds[0] = 1;
        tokenIds[1] = 2;

        string[] memory tokenUris = new string[](2);
        tokenUris[0] = "ipfs://mockURI1";
        tokenUris[1] = "ipfs://mockURI2";

        vm.startPrank(address(mockIBCHandler));
        ucs02NFT.onRecvPacket(
            IBCPacket({
                sourceChannel: 1,
                destinationChannel: 1,
                data: NFTPacketLib.encode(
                    NFTPacket({
                        classOwner: "",
                        classId: "remoteNFT",
                        className: "TestNFT",
                        classSymbol: "TNFT",
                        tokenIds: tokenIds,
                        tokenUris: tokenUris,
                        sender: "sender_address",
                        receiver: toHexString(user),
                        memo: ""
                    })
                ),
                timeoutHeight: 0,
                timeoutTimestamp: 0
            }),
            relayer,
            bytes("")
        );
        vm.stopPrank();

        // // Verify that the mapping is populated
        // string memory nftDenom = NFTLib.makeForeignDenom(1, "remoteNFT");
        // address nftClass = ucs02NFT.denomToNft(1, nftDenom);
        // assertTrue(nftClass != address(0), "denomToNft mapping not populated");

        tokenIds[0] = 3;
        tokenIds[1] = 4;
        // Simulate acknowledgment failure for refund
        string memory ourdenom = NFTLib.makeForeignDenom(1, "remoteNFT");
        bytes memory ack = abi.encodePacked(NFTLib.ACK_FAILURE);
        vm.startPrank(address(mockIBCHandler));
        ucs02NFT.onAcknowledgementPacket(
            IBCPacket({
                sourceChannel: 1,
                destinationChannel: 1,
                data: NFTPacketLib.encode(
                    NFTPacket({
                        classOwner: "",
                        classId: ourdenom,
                        className: "TestNFT",
                        classSymbol: "TNFT",
                        tokenIds: tokenIds,
                        tokenUris: tokenUris,
                        sender: toHexString(user),
                        receiver: "receiver_address",
                        memo: ""
                    })
                ),
                timeoutHeight: 0,
                timeoutTimestamp: 0
            }),
            ack,
            address(this)
        );
        vm.stopPrank();

        // Verify refund
        MockERC721 new_erc721 =
            MockERC721(0x4f81992FCe2E1846dD528eC0102e6eE1f61ed3e2);
        assertEq(new_erc721.ownerOf(3), user, "Token 3 not refunded correctly");
        assertEq(new_erc721.ownerOf(4), user, "Token 4 not refunded correctly");
    }

    function test_onAcknowledgementPacket_refund_MustTransferAtLeastOneToken()
        public
    {
        MockERC721 mockERC721 = new MockERC721("TestNFT", "TNFT");

        mockERC721.mint(user, 1);
        mockERC721.mint(user, 2);

        uint256[] memory tokenIds = new uint256[](0);

        vm.startPrank(user);
        mockERC721.setApprovalForAll(address(ucs02NFT), true);
        vm.expectRevert(
            abi.encodeWithSelector(NFTLib.MustTransferAtLeastOneToken.selector)
        );
        ucs02NFT.send(1, "receiver_address", address(mockERC721), tokenIds, 0);
        vm.stopPrank();
    }

    function test_onAcknowledgementPacket_refund() public {
        MockERC721 mockERC721 = new MockERC721("TestNFT", "TNFT");

        // Mint tokens for the user
        mockERC721.mint(user, 1);
        mockERC721.mint(user, 2);

        uint256[] memory tokenIds = new uint256[](0);

        // Simulate acknowledgment failure
        bytes memory ack = abi.encodePacked(NFTLib.ACK_FAILURE);
        string[] memory tokenUris = new string[](0);
        // tokenUris[0] = "ipfs://mockURI1";
        // tokenUris[1] = "ipfs://mockURI2";
        vm.startPrank(address(mockIBCHandler));
        ucs02NFT.onAcknowledgementPacket(
            IBCPacket({
                sourceChannel: 1,
                destinationChannel: 1,
                data: NFTPacketLib.encode(
                    NFTPacket({
                        classOwner: "",
                        classId: "mockNFT",
                        className: "TestNFT",
                        classSymbol: "TNFT",
                        tokenIds: tokenIds,
                        tokenUris: tokenUris,
                        sender: toHexString(user),
                        receiver: "receiver_address",
                        memo: ""
                    })
                ),
                timeoutHeight: 0,
                timeoutTimestamp: 0
            }),
            ack,
            address(this)
        );
        vm.stopPrank();
    }

    function test_onChanOpenAck_validVersion() public {
        string memory validVersion = "ucs02-nft-1";

        vm.startPrank(address(mockIBCHandler));
        ucs02NFT.onChanOpenAck(1, 1, validVersion, address(0));
        vm.stopPrank();

        // No revert means the test passes for valid versions
        assertTrue(true, "onChanOpenAck did not revert for a valid version");
    }

    function test_onChanOpenAck_invalidVersion_reverts() public {
        string memory invalidVersion = "invalid-version";

        vm.startPrank(address(mockIBCHandler));
        vm.expectRevert(
            abi.encodeWithSelector(
                NFTLib.ErrInvalidCounterpartyProtocolVersion.selector
            )
        );
        ucs02NFT.onChanOpenAck(1, 1, invalidVersion, address(0));
        vm.stopPrank();
    }

    function test_onChanCloseInit_reverts() public {
        vm.startPrank(address(mockIBCHandler));
        vm.expectRevert(abi.encodeWithSelector(NFTLib.ErrUnstoppable.selector));
        ucs02NFT.onChanCloseInit(1, address(0));
        vm.stopPrank();
    }

    function test_onChanCloseConfirm_reverts() public {
        vm.startPrank(address(mockIBCHandler));
        vm.expectRevert(abi.encodeWithSelector(NFTLib.ErrUnstoppable.selector));
        ucs02NFT.onChanCloseConfirm(1, address(0));
        vm.stopPrank();
    }

    function test_onlyIBC() public {
        vm.startPrank(address(relayer));
        vm.expectRevert(abi.encodeWithSelector(IBCAppLib.ErrNotIBC.selector));
        ucs02NFT.onChanCloseConfirm(1, address(0));
        vm.stopPrank();
    }

    function test_onChanOpenTry_invalidVersion_reverts() public {
        vm.startPrank(address(mockIBCHandler));
        string memory validVersion = "ucs02-nft-1";
        string memory invalidCounterpartyVersion = "invalid-version";

        // Expect revert due to invalid counterparty version
        vm.expectRevert(
            abi.encodeWithSelector(
                NFTLib.ErrInvalidCounterpartyProtocolVersion.selector
            )
        );
        ucs02NFT.onChanOpenTry(
            1, 1, 1, validVersion, invalidCounterpartyVersion, user
        );
        vm.stopPrank();
    }

    function test_onChanOpenTry_invalidVersion_reverts_protocol() public {
        vm.startPrank(address(mockIBCHandler));
        string memory validVersion = "invalid-version-ucs01-relay-1";
        string memory invalidCounterpartyVersion = "invalid-version";

        // Expect revert due to invalid counterparty version
        vm.expectRevert(
            abi.encodeWithSelector(NFTLib.ErrInvalidProtocolVersion.selector)
        );
        ucs02NFT.onChanOpenTry(
            1, 1, 1, validVersion, invalidCounterpartyVersion, user
        );
        vm.stopPrank();
    }
}

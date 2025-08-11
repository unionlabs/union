pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/utils/Address.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibBit.sol";
import "solady/utils/LibString.sol";
import "solady/utils/LibBytes.sol";
import "solady/utils/LibCall.sol";
import "solady/utils/EfficientHashLib.sol";
import "solady/utils/SafeTransferLib.sol";
import "solady/utils/LibTransient.sol";

import "../../Base.sol";
import "../../../core/04-channel/IBCPacket.sol";
import "../../../core/05-port/IIBCModule.sol";
import "../../../core/24-host/IBCCommitment.sol";
import "../../../internal/Versioned.sol";

import "./TokenBucket.sol";
import "./IWETH.sol";
import "./IZkgmable.sol";
import "./IZkgmERC20.sol";
import "./ZkgmERC20.sol";
import "./ZkgmERC721.sol";
import "./IZkgm.sol";
import "./Lib.sol";

abstract contract UCS03ZkgmStore is AccessManagedUpgradeable, IZkgmStore {
    using ZkgmLib for *;
    using LibString for *;
    using LibBytes for *;
    using SafeERC20 for *;
    using Address for *;
    using LibCall for *;

    bytes32 internal constant STAKE_NFT_MANAGER_SALT =
        keccak256("union.salt.zkgm.stakeNFTManager");

    string internal constant STAKE_NFT_NAME = "Zkgm Staking Position";
    string internal constant STAKE_NFT_SYMBOL = "ZKGMSP";

    IIBCModulePacket private _deprecated_ibcHandler;
    mapping(bytes32 => IBCPacket) public inFlightPacket;
    mapping(address => uint256) public tokenOrigin;
    mapping(uint32 => mapping(uint256 => mapping(address => uint256))) public
        _deprecated_channelBalanceV1;
    mapping(uint32 => GovernanceToken) public channelGovernanceToken;
    mapping(uint256 => ZkgmStake) public stakes;
    mapping(address => bytes32) public metadataImageOf;
    mapping(
        uint32
            => mapping(uint256 => mapping(address => mapping(bytes => uint256)))
    ) public channelBalanceV2;

    function decodeZkgmERC20InitializeCall(
        bytes calldata call
    )
        external
        pure
        returns (address, address, string memory, string memory, uint8)
    {
        bytes4 selector = bytes4(call.slice(0, 4));
        bytes4 expectedSelector = ZkgmERC20.initialize.selector;
        require(selector == expectedSelector);
        return
            abi.decode(call.slice(4), (address, address, string, string, uint8));
    }

    function decodeRelayerMessage(
        bytes calldata relayerMsg
    ) external pure returns (bool, bytes memory) {
        return abi.decode(relayerMsg, (bool, bytes));
    }

    function _getGovernanceToken(
        uint32 channelId
    ) internal view returns (ZkgmERC20, GovernanceToken memory) {
        GovernanceToken memory governanceToken =
            channelGovernanceToken[channelId];
        if (governanceToken.unwrappedToken.length == 0) {
            revert ZkgmLib.ErrChannelGovernanceTokenNotSet();
        }
        (address wrappedGovernanceToken,) =
        _predictWrappedTokenFromMetadataImageV2(
            0,
            channelId,
            governanceToken.unwrappedToken,
            governanceToken.metadataImage
        );
        return (ZkgmERC20(wrappedGovernanceToken), governanceToken);
    }

    function getGovernanceToken(
        uint32 channelId
    ) public view returns (ZkgmERC20, GovernanceToken memory) {
        return _getGovernanceToken(channelId);
    }

    function _predictStakeManagerAddress() internal view returns (ZkgmERC721) {
        return ZkgmERC721(
            CREATE3.predictDeterministicAddress(STAKE_NFT_MANAGER_SALT)
        );
    }

    function predictStakeManagerAddress() public view returns (ZkgmERC721) {
        return _predictStakeManagerAddress();
    }

    function _getStakeNFTManager() internal returns (ZkgmERC721) {
        ZkgmERC721 stakeManager = _predictStakeManagerAddress();
        if (!ZkgmLib.isDeployed(address(stakeManager))) {
            CREATE3.deployDeterministic(
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        new ZkgmERC721(),
                        abi.encodeCall(
                            ZkgmERC721.initialize,
                            (
                                authority(),
                                address(this),
                                STAKE_NFT_NAME,
                                STAKE_NFT_SYMBOL
                            )
                        )
                    )
                ),
                STAKE_NFT_MANAGER_SALT
            );
        }
        return stakeManager;
    }

    function _increaseOutstandingV2(
        uint32 sourceChannelId,
        uint256 path,
        address baseToken,
        bytes calldata quoteToken,
        uint256 amount
    ) internal {
        channelBalanceV2[sourceChannelId][path][baseToken][quoteToken] += amount;
    }

    function _decreaseOutstandingV2(
        uint32 sourceChannelId,
        uint256 path,
        address baseToken,
        bytes calldata quoteToken,
        uint256 amount
    ) internal {
        channelBalanceV2[sourceChannelId][path][baseToken][quoteToken] -= amount;
    }

    // Predict a wrapped token address given the path/channel and counterparty
    // address of the token. The computed address is fully deterministic w.r.t
    // to (ucs03Address, path, channel, token).
    function _predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt =
            EfficientHashLib.hash(abi.encode(path, channel, token));
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }

    function _predictWrappedTokenFromMetadataImageV2(
        uint256 path,
        uint32 channel,
        bytes memory token,
        bytes32 metadataImage
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt = EfficientHashLib.hash(
            abi.encode(path, channel, token, metadataImage)
        );
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }

    function _predictWrappedTokenV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        TokenMetadata memory metadata
    ) internal returns (address, bytes32) {
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeTokenMetadata(metadata));
        return _predictWrappedTokenFromMetadataImageV2(
            path, channel, token, metadataImage
        );
    }
}

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
        channelBalance;
    mapping(uint32 => GovernanceToken) public channelGovernanceToken;
    mapping(uint256 => ZkgmStake) public stakes;
    mapping(address => bytes32) public metadataImageOf;
    mapping(
        uint32
            => mapping(
                uint256 => mapping(address => mapping(bytes32 => uint256))
            )
    ) public channelBalanceV2;

    function _getGovernanceToken(
        uint32 channelId
    ) internal view returns (ZkgmERC20, bytes memory) {
        GovernanceToken memory governanceToken =
            channelGovernanceToken[channelId];
        if (governanceToken.unwrappedToken.length == 0) {
            revert ZkgmLib.ErrChannelGovernanceTokenNotSet();
        }
        (address wrappedGovernanceToken,,) =
        _predictWrappedTokenFromMetadataImageV2(
            0,
            channelId,
            governanceToken.unwrappedToken,
            governanceToken.metadataImage,
            false
        );
        return
            (ZkgmERC20(wrappedGovernanceToken), governanceToken.unwrappedToken);
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

    // Increase the outstanding balance of a channel. This ensure that malicious
    // channels can't unescrow/mint more tokens than previously escrowed/burnt.
    function _increaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) internal {
        channelBalance[sourceChannelId][path][token] += amount;
    }

    function _increaseOutstandingV2(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        bytes32 metadataImage,
        uint256 amount
    ) internal {
        channelBalanceV2[sourceChannelId][path][token][metadataImage] += amount;
    }

    // Decrease the outstanding balance of a (channel, path). If the function is
    // called when receiving funds, hence, to decrease we need to first inverse
    // the path. If we increased the balance for (0, [1, 2, 3]) and funds are
    // sent back over [3, 2, 1], this will only work if the path is the inverse.
    // If the function is called on refund, simplify subtract the refunded
    // amount.
    function _decreaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) internal {
        channelBalance[sourceChannelId][path][token] -= amount;
    }

    function _decreaseOutstandingV2(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        bytes32 metadataImage,
        uint256 amount
    ) internal {
        channelBalanceV2[sourceChannelId][path][token][metadataImage] -= amount;
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
        bytes32 metadataImage,
        bool checkV1
    ) internal view returns (address, bytes32, bool) {
        if (checkV1) {
            if (
                metadataImage
                    == ZkgmLib.FUNGIBLE_ASSET_METADATA_IMAGE_PREDICT_V1
            ) {
                (address quoteTokenV1, bytes32 saltV1) =
                    _predictWrappedTokenMemory(path, channel, token);
                return (quoteTokenV1, saltV1, true);
            }
        }
        bytes32 wrappedTokenSalt = EfficientHashLib.hash(
            abi.encode(path, channel, token, metadataImage)
        );
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt, false);
    }

    function _predictWrappedTokenV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        FungibleAssetMetadata memory metadata
    ) internal returns (address, bytes32) {
        bytes32 metadataImage =
            EfficientHashLib.hash(ZkgmLib.encodeFungibleAssetMetadata(metadata));
        (address wrappedToken, bytes32 salt,) =
        _predictWrappedTokenFromMetadataImageV2(
            path, channel, token, metadataImage, false
        );
        return (wrappedToken, salt);
    }

    function _predictWrappedTokenMemory(
        uint256 path,
        uint32 channel,
        bytes memory token
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt =
            EfficientHashLib.hash(abi.encode(path, channel, token));
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }
}

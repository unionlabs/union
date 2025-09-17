pragma solidity ^0.8.27;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";

import "solady/utils/LibBytes.sol";

import "./internal/Versioned.sol";
import "./apps/ucs/03-zkgm/ISolver.sol";
import "./core/04-channel/IBCPacket.sol";

contract CrosschainEscrowVault is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    Versioned,
    ISolver
{
    using LibBytes for *;
    using SafeERC20 for IERC20;

    // ERC-7528: Native token address
    address public constant NATIVE_TOKEN =
        0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;

    error CrosschainEscrowVault_OnlyZkgm();
    error CrosschainEscrowVault_IntentWhitelistedOnly();
    error CrosschainEscrowVault_CounterpartyIsNotFungible();
    error CrosschainEscrowVault_BaseAmountMustCoverQuoteAmount();
    error CrosschainEscrowVault_InvalidQuoteToken();
    error CrosschainEscrowVault_TransferFailed();

    bytes32 internal constant CrosschainEscrowVault_STORAGE_SLOT = keccak256(
        abi.encode(
            uint256(keccak256("union.storage.crosschainEscrowVault")) - 1
        )
    ) & ~bytes32(uint256(0xff));

    struct FungibleCounterparty {
        bytes beneficiary; // Market maker address on counterparty chain
        address escrowedToken; // Token contract address (NATIVE_TOKEN for native ETH)
    }

    struct CrosschainEscrowVaultStorage {
        address zkgm;
        mapping(
            uint256 => mapping(uint32 => mapping(bytes => FungibleCounterparty))
        ) fungibleCounterparties;
        mapping(bytes32 => bool) intentWhitelist;
    }

    function _getCrosschainEscrowVaultStorage()
        private
        pure
        returns (CrosschainEscrowVaultStorage storage $)
    {
        bytes32 slot = CrosschainEscrowVault_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _authority,
        address _zkgm
    ) external initializer {
        __CrosschainEscrowVault_init(_authority, _zkgm);
    }

    function __CrosschainEscrowVault_init(
        address _authority,
        address _zkgm
    ) private onlyInitializing {
        __UUPSUpgradeable_init();
        __AccessManaged_init(_authority);
        CrosschainEscrowVaultStorage storage $ =
            _getCrosschainEscrowVaultStorage();
        $.zkgm = _zkgm;
    }

    modifier onlyZkgm() {
        if (msg.sender != _getCrosschainEscrowVaultStorage().zkgm) {
            revert CrosschainEscrowVault_OnlyZkgm();
        }
        _;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}

    function setFungibleCounterparty(
        uint256 path,
        uint32 channelId,
        bytes calldata token,
        FungibleCounterparty calldata counterparty
    ) public restricted {
        _getCrosschainEscrowVaultStorage().fungibleCounterparties[path][channelId][token]
        = counterparty;
    }

    function whitelistIntent(
        bytes32[] calldata packetHashes,
        bool whitelist
    ) public restricted {
        CrosschainEscrowVaultStorage storage $ =
            _getCrosschainEscrowVaultStorage();
        for (uint256 i = 0; i < packetHashes.length; i++) {
            $.intentWhitelist[packetHashes[i]] = whitelist;
        }
    }

    function solve(
        IBCPacket calldata packet,
        TokenOrderV2 calldata order,
        uint256 path,
        address, /* caller */
        address relayer,
        bytes calldata, /* relayerMsg */
        bool intent
    ) external override onlyZkgm returns (bytes memory) {
        _assertIntentWhitelisted(intent, packet);

        FungibleCounterparty memory counterparty =
        _verifyOrderAndGetCounterparty(path, packet.destinationChannelId, order);

        _unescrow(
            counterparty.escrowedToken,
            address(bytes20(order.receiver)),
            order.quoteAmount
        );
        _unescrow(
            counterparty.escrowedToken,
            relayer,
            order.baseAmount - order.quoteAmount
        );

        return counterparty.beneficiary;
    }

    function _assertIntentWhitelisted(
        bool intent,
        IBCPacket calldata packet
    ) internal view {
        if (intent) {
            bytes32 packetHash = IBCPacketLib.commitPacket(packet);
            if (!_getCrosschainEscrowVaultStorage().intentWhitelist[packetHash])
            {
                revert CrosschainEscrowVault_IntentWhitelistedOnly();
            }
        }
    }

    function _verifyOrderAndGetCounterparty(
        uint256 path,
        uint32 channelId,
        TokenOrderV2 calldata order
    ) internal view returns (FungibleCounterparty memory counterparty) {
        counterparty = _getCrosschainEscrowVaultStorage().fungibleCounterparties[path][channelId][order
            .baseToken];

        if (counterparty.beneficiary.length == 0) {
            revert CrosschainEscrowVault_CounterpartyIsNotFungible();
        }

        if (!order.quoteToken.eq(abi.encodePacked(counterparty.escrowedToken)))
        {
            revert CrosschainEscrowVault_InvalidQuoteToken();
        }

        if (order.quoteAmount > order.baseAmount) {
            revert CrosschainEscrowVault_BaseAmountMustCoverQuoteAmount();
        }
    }

    function _unescrow(address token, address to, uint256 amount) internal {
        if (amount == 0) return;

        if (token == NATIVE_TOKEN) {
            (bool success,) = to.call{value: amount}("");
            if (!success) {
                revert CrosschainEscrowVault_TransferFailed();
            }
        } else {
            IERC20(token).safeTransfer(to, amount);
        }
    }

    function zkgm() public view returns (address) {
        return _getCrosschainEscrowVaultStorage().zkgm;
    }

    function fungibleCounterparty(
        uint256 path,
        uint32 channelId,
        bytes calldata baseToken
    ) public view returns (FungibleCounterparty memory) {
        return _getCrosschainEscrowVaultStorage().fungibleCounterparties[path][channelId][baseToken];
    }

    function intentWhitelist(
        bytes32 packetHash
    ) public view returns (bool) {
        return _getCrosschainEscrowVaultStorage().intentWhitelist[packetHash];
    }

    receive() external payable {
        // Contract can receive ETH for escrow purposes
    }
}

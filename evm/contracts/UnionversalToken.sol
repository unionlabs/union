pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";

import "solady/utils/LibBytes.sol";

import "./internal/Versioned.sol";
import "./apps/ucs/03-zkgm/ISolver.sol";
import "./core/04-channel/IBCPacket.sol";

contract UnionversalToken is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    ERC20Upgradeable,
    Versioned,
    ISolver
{
    using LibBytes for *;

    error UnionversalToken_OnlyZkgm();
    error UnionversalToken_IntentWhitelistedOnly();
    error UnionversalToken_CounterpartyIsNotFungible();
    error UnionversalToken_BaseAmountMustCoverQuoteAmount();
    error UnionversalToken_InvalidCounterpartyBeneficiary();
    error UnionversalToken_Fool();

    bytes32 internal constant UnionversalToken_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.zkgm.u")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct FungibleCounterparty {
        bytes beneficiary;
    }

    struct UStorage {
        address zkgm;
        uint8 decimals;
        bytes salt;
        // (path, channelId, baseToken) => FungibleCounterparty
        mapping(
            uint256 => mapping(uint32 => mapping(bytes => FungibleCounterparty))
        ) fungibleCounterparties;
        mapping(bytes32 => bool) intentWhitelist;
    }

    function _getUStorage() private pure returns (UStorage storage $) {
        bytes32 slot = UnionversalToken_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _authority,
        address _zkgm,
        string calldata _name,
        string calldata _symbol,
        uint8 _decimals,
        bytes calldata _salt
    ) external initializer {
        __UnionversalToken_init(
            _authority, _zkgm, _name, _symbol, _decimals, _salt
        );
    }

    function __UnionversalToken_init(
        address _authority,
        address _zkgm,
        string calldata _name,
        string calldata _symbol,
        uint8 _decimals,
        bytes calldata _salt
    ) internal onlyInitializing {
        __UUPSUpgradeable_init();
        __AccessManaged_init(_authority);
        __ERC20_init(_name, _symbol);
        UStorage storage $ = _getUStorage();
        $.zkgm = _zkgm;
        $.decimals = _decimals;
        $.salt = _salt;
    }

    function zkgm() public view returns (address) {
        return _getUStorage().zkgm;
    }

    function salt() public view returns (bytes memory) {
        return _getUStorage().salt;
    }

    function fungibleCounterparty(
        uint256 path,
        uint32 channelId,
        bytes calldata baseToken
    ) public view returns (FungibleCounterparty memory) {
        return _getUStorage().fungibleCounterparties[path][channelId][baseToken];
    }

    function intentWhitelist(
        bytes32 packetHash
    ) public view returns (bool) {
        return _getUStorage().intentWhitelist[packetHash];
    }

    function decimals()
        public
        view
        override(ERC20Upgradeable)
        returns (uint8)
    {
        return _getUStorage().decimals;
    }

    function mint(address to, uint256 amount) external onlyZkgm {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyZkgm {
        _burn(from, amount);
    }

    function setFungibleCounterparty(
        uint256 path,
        uint32 channelId,
        bytes calldata token,
        FungibleCounterparty calldata counterparty
    ) public restricted {
        _getUStorage().fungibleCounterparties[path][channelId][token] =
            counterparty;
    }

    function whitelistIntent(
        bytes32[] calldata packetHashes,
        bool whitelist
    ) public restricted {
        for (uint256 i = 0; i < packetHashes.length; i++) {
            _getUStorage().intentWhitelist[packetHashes[i]] = whitelist;
        }
    }

    function solve(
        IBCPacket calldata packet,
        TokenOrderV2 calldata order,
        uint256 path,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) external override onlyZkgm returns (bytes memory) {
        if (intent) {
            bytes32 packetHash = IBCPacketLib.commitPacket(packet);
            if (!_getUStorage().intentWhitelist[packetHash]) {
                revert UnionversalToken_IntentWhitelistedOnly();
            }
        }

        FungibleCounterparty memory counterparty = _getUStorage()
            .fungibleCounterparties[path][packet.destinationChannelId][order
            .baseToken];
        if (counterparty.beneficiary.length == 0) {
            revert UnionversalToken_CounterpartyIsNotFungible();
        }

        if (!order.quoteToken.eq(abi.encodePacked(address(this)))) {
            revert UnionversalToken_Fool();
        }

        if (order.quoteAmount > order.baseAmount) {
            revert UnionversalToken_BaseAmountMustCoverQuoteAmount();
        }

        // Incentive relayer.
        uint256 fee = order.baseAmount - order.quoteAmount;
        if (fee > 0) {
            _mint(relayer, fee);
        }

        if (order.quoteAmount > 0) {
            _mint(address(bytes20(order.receiver)), order.quoteAmount);
        }

        // The market maker address will be the preconfigured beneficiary.
        // Likely another U contract/vault.
        return counterparty.beneficiary;
    }

    modifier onlyZkgm() {
        if (msg.sender != _getUStorage().zkgm) {
            revert UnionversalToken_OnlyZkgm();
        }
        _;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}
}

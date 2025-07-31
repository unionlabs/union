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

contract U is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    ERC20Upgradeable,
    Versioned,
    ISolver
{
    using LibBytes for *;

    error U_OnlyZkgm();
    error U_IntentWhitelistedOnly();
    error U_CounterpartyIsNotFungible();
    error U_BaseAmountMustCoverQuoteAmount();
    error U_InvalidCounterpartyBeneficiary();

    bytes32 internal constant U_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.zkgm.u")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct FungibleCounterparty {
        bytes beneficiary;
    }

    struct UStorage {
        address zkgm;
        uint8 decimals;
        bytes salt;
        // (channelId, baseToken) => FungibleCounterparty
        mapping(uint32 => mapping(bytes => FungibleCounterparty))
            fungibleCounterparties;
        mapping(bytes32 => bool) intentWhitelist;
    }

    function _getUStorage() private pure returns (UStorage storage $) {
        bytes32 slot = U_STORAGE_SLOT;
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
        __U_init(_authority, _zkgm, _name, _symbol, _decimals, _salt);
    }

    function __U_init(
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

    function decimals()
        public
        view
        override(ERC20Upgradeable)
        returns (uint8)
    {
        return _getUStorage().decimals;
    }

    function transfer(
        address to,
        uint256 value
    ) public override returns (bool) {
        address from = _msgSender();
        // Allow zkgm transferring to the zero address (burning).
        if (from == _getUStorage().zkgm) {
            _update(from, to, value);
        } else {
            _transfer(from, to, value);
        }
        return true;
    }

    function mint(address to, uint256 amount) external onlyZkgm {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyZkgm {
        _burn(from, amount);
    }

    function setFungibleCounterparty(
        uint32 channelId,
        bytes calldata token,
        FungibleCounterparty calldata counterparty
    ) public restricted {
        _getUStorage().fungibleCounterparties[channelId][token] = counterparty;
    }

    function whitelistIntent(
        bytes32[] calldata packetHashes,
        bool whitelist
    ) public restricted {
        for (uint256 i = 0; i < packetHashes.length; i++) {
            _getUStorage().intentWhitelist[packetHashes[i]] = whitelist;
        }
    }

    function allowMarketMakers() external override returns (bool) {
        return true;
    }

    function solve(
        IBCPacket calldata packet,
        TokenOrderV2 calldata order,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) external override onlyZkgm {
        if (intent) {
            bytes32 packetHash = IBCPacketLib.commitPacket(packet);
            if (!_getUStorage().intentWhitelist[packetHash]) {
                revert U_IntentWhitelistedOnly();
            }
        }

        FungibleCounterparty memory counterparty = _getUStorage()
            .fungibleCounterparties[packet.destinationChannelId][order.baseToken];
        if (counterparty.beneficiary.length == 0) {
            revert U_CounterpartyIsNotFungible();
        }

        // Maker address is provided by the relayer in relayerMsg.
        // Ensure it's the configured address that will gets the funds on acknowledgement.
        if (!relayerMsg.eq(abi.encodePacked(counterparty.beneficiary))) {
            revert U_InvalidCounterpartyBeneficiary();
        }

        if (order.quoteAmount > order.baseAmount) {
            revert U_BaseAmountMustCoverQuoteAmount();
        }

        // Incentive relayer.
        uint256 fee = order.quoteAmount - order.baseAmount;
        if (fee > 0) {
            _mint(relayer, fee);
        }

        uint256 quoteAmountMinusFee = order.quoteAmount - fee;
        if (quoteAmountMinusFee > 0) {
            _mint(address(bytes20(order.receiver)), quoteAmountMinusFee);
        }
    }

    modifier onlyZkgm() {
        if (msg.sender != _getUStorage().zkgm) {
            revert U_OnlyZkgm();
        }
        _;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}
}

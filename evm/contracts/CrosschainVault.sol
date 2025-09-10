pragma solidity ^0.8.27;

import "@openzeppelin/contracts/utils/math/Math.sol";

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/token/ERC20/extensions/ERC4626Upgradeable.sol";

import "solady/utils/LibBytes.sol";

import "./internal/Versioned.sol";
import "./apps/ucs/03-zkgm/ISolver.sol";
import "./core/04-channel/IBCPacket.sol";

contract CrosschainVault is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    ERC4626Upgradeable,
    Versioned,
    ISolver
{
    using LibBytes for *;
    using Math for *;

    uint256 public constant BPS_SCALE = 1_000_000;

    error CrosschainVault_OnlyZkgm();
    error CrosschainVault_IntentWhitelistedOnly();
    error CrosschainVault_CounterpartyIsNotFungible();
    error CrosschainVault_BaseAmountMustCoverQuoteAmount();
    error CrosschainVault_InvalidCounterpartyBeneficiary();
    error CrosschainVault_Fool();
    error CrosschainVault_RespectMinFee();
    error CrosschainVault_RepayingTooMuch();

    bytes32 internal constant CrosschainVault_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.crosschainVault")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct FungibleCounterparty {
        uint256 bpsFee;
        uint256 bpsProtocolFee;
        address protocolFeeBeneficiary;
        uint256 debt;
        bytes debtAccount;
    }

    struct CrosschainVaultStorage {
        // (path, channelId, baseToken) => FungibleCounterparty
        mapping(
            uint256 => mapping(uint32 => mapping(bytes => FungibleCounterparty))
        ) fungibleCounterparties;
        mapping(bytes32 => bool) intentWhitelist;
        address zkgm;
        address quoteToken;
        uint256 deployedCapital;
    }

    function _getCrosschainVaultStorage()
        private
        pure
        returns (CrosschainVaultStorage storage $)
    {
        bytes32 slot = CrosschainVault_STORAGE_SLOT;
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
        address _quoteToken
    ) external initializer {
        __CrosschainVault_init(_authority, _zkgm, _quoteToken);
    }

    function __CrosschainVault_init(
        address _authority,
        address _zkgm,
        address _quoteToken
    ) private onlyInitializing {
        __UUPSUpgradeable_init();
        __AccessManaged_init(_authority);
        __ERC4626_init(IERC20(_quoteToken));
        CrosschainVaultStorage storage $ = _getCrosschainVaultStorage();
        $.zkgm = _zkgm;
        $.quoteToken = _quoteToken;
        $.deployedCapital = 0;
    }

    function zkgm() public view returns (address) {
        return _getCrosschainVaultStorage().zkgm;
    }

    function quoteToken() public view returns (address) {
        return _getCrosschainVaultStorage().quoteToken;
    }

    function deployedCapital() public view returns (uint256) {
        return _getCrosschainVaultStorage().deployedCapital;
    }

    function fungibleCounterparty(
        uint256 path,
        uint32 channelId,
        bytes calldata baseToken
    ) public view returns (FungibleCounterparty memory) {
        return _getCrosschainVaultStorage().fungibleCounterparties[path][channelId][baseToken];
    }

    function intentWhitelist(
        bytes32 packetHash
    ) public view returns (bool) {
        return _getCrosschainVaultStorage().intentWhitelist[packetHash];
    }

    function setFungibleCounterparty(
        uint256 path,
        uint32 channelId,
        bytes calldata token,
        FungibleCounterparty calldata counterparty
    ) public restricted {
        _getCrosschainVaultStorage().fungibleCounterparties[path][channelId][token]
        = counterparty;
    }

    function whitelistIntent(
        bytes32[] calldata packetHashes,
        bool whitelist
    ) public restricted {
        for (uint256 i = 0; i < packetHashes.length; i++) {
            _getCrosschainVaultStorage().intentWhitelist[packetHashes[i]] =
                whitelist;
        }
    }

    function _fee(
        uint256 amount,
        uint256 bps
    ) internal pure returns (uint256) {
        return amount.mulDiv(bps, BPS_SCALE);
    }

    function fee(
        uint256 path,
        uint32 channelId,
        bytes calldata baseToken,
        uint256 amount
    ) public view returns (uint256) {
        FungibleCounterparty memory counterparty = _getCrosschainVaultStorage()
            .fungibleCounterparties[path][channelId][baseToken];
        return _fee(amount, counterparty.bpsFee)
            + _fee(amount, counterparty.bpsProtocolFee);
    }

    function totalAssets() public view override returns (uint256) {
        return IERC20(asset()).balanceOf(address(this))
            + _getCrosschainVaultStorage().deployedCapital;
    }

    function _deployCapital(address beneficiary, uint256 amount) internal {
        if (amount > 0) {
            IERC20(asset()).transfer(beneficiary, amount);
            _getCrosschainVaultStorage().deployedCapital += amount;
        }
    }

    function _accountDebt(
        uint256 path,
        uint32 channelId,
        bytes calldata baseToken,
        uint256 amount
    ) internal {
        _getCrosschainVaultStorage().fungibleCounterparties[path][channelId][baseToken]
            .debt += amount;
    }

    function repay(
        uint256 path,
        uint32 channelId,
        bytes calldata baseToken,
        uint256 amount
    ) public {
        CrosschainVaultStorage storage $ = _getCrosschainVaultStorage();
        FungibleCounterparty storage counterparty =
            $.fungibleCounterparties[path][channelId][baseToken];
        if (counterparty.debtAccount.length == 0) {
            revert CrosschainVault_CounterpartyIsNotFungible();
        }
        if (counterparty.debt < amount) {
            revert CrosschainVault_RepayingTooMuch();
        }
        IERC20(asset()).transferFrom(msg.sender, address(this), amount);
        counterparty.debt -= amount;
        $.deployedCapital -= amount;
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
        CrosschainVaultStorage storage $ = _getCrosschainVaultStorage();
        if (intent) {
            bytes32 packetHash = IBCPacketLib.commitPacket(packet);
            if (!$.intentWhitelist[packetHash]) {
                revert CrosschainVault_IntentWhitelistedOnly();
            }
        }

        FungibleCounterparty memory counterparty = $.fungibleCounterparties[path][packet
            .destinationChannelId][order.baseToken];
        if (counterparty.debtAccount.length == 0) {
            revert CrosschainVault_CounterpartyIsNotFungible();
        }

        if (!order.quoteToken.eq(abi.encodePacked($.quoteToken))) {
            revert CrosschainVault_Fool();
        }

        if (order.quoteAmount > order.baseAmount) {
            revert CrosschainVault_BaseAmountMustCoverQuoteAmount();
        }

        uint256 expectedVaultFee = _fee(order.baseAmount, counterparty.bpsFee);
        uint256 expectedProtocolFee =
            _fee(order.baseAmount, counterparty.bpsProtocolFee);
        uint256 expectedMinimumFee = expectedVaultFee + expectedProtocolFee;

        uint256 actualFee = order.baseAmount - order.quoteAmount;
        if (actualFee < expectedMinimumFee) {
            revert CrosschainVault_RespectMinFee();
        }

        // Pay receiver.
        _deployCapital(address(bytes20(order.receiver)), order.quoteAmount);

        // Pay protocol.
        _deployCapital(counterparty.protocolFeeBeneficiary, expectedProtocolFee);

        // Pay vault by not deploying any capital for the expectedVaultFee.

        // Incentive relayer if anything is left.
        uint256 relayerFee = actualFee - expectedMinimumFee;
        _deployCapital(relayer, relayerFee);

        // The full base amount is a debt from the counterparty.
        _accountDebt(
            path, packet.destinationChannelId, order.baseToken, order.baseAmount
        );

        // Counterparty debtAccount is funded on acknowledgement and owe us the debt amount.
        return counterparty.debtAccount;
    }

    modifier onlyZkgm() {
        if (msg.sender != _getCrosschainVaultStorage().zkgm) {
            revert CrosschainVault_OnlyZkgm();
        }
        _;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}
}

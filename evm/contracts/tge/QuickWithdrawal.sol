pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

import "solady/tokens/ERC20.sol";

import "../internal/Versioned.sol";

contract QuickWithdrawal is
    Initializable,
    UUPSUpgradeable,
    PausableUpgradeable,
    AccessManagedUpgradeable,
    Versioned
{
    error QuickWithdrawal_ZeroAddress();
    error QuickWithdrawal_ZeroAmount();
    error QuickWithdrawal_RateTooHigh();
    error QuickWithdrawal_MustBeReplenished();
    error QuickWithdrawal_NotActive();
    error QuickWithdrawal_Active();

    uint256 public constant SCALE = 1_000_000_000;

    address public immutable BASE_TOKEN;
    address public immutable QUOTE_TOKEN;

    bool public active;
    uint256 public rate;

    constructor(address _baseToken, address _quoteToken) {
        _disableInitializers();
        if (_baseToken == address(0)) revert QuickWithdrawal_ZeroAddress();
        if (_quoteToken == address(0)) revert QuickWithdrawal_ZeroAddress();
        BASE_TOKEN = _baseToken;
        QUOTE_TOKEN = _quoteToken;
    }

    function initialize(
        address _authority,
        bool _active,
        uint256 _rate
    ) external initializer {
        __UUPSUpgradeable_init();
        __Pausable_init();
        __AccessManaged_init(_authority);
        active = _active;
        _setRate(_rate);
    }

    function swap(
        uint256 baseAmount
    ) public returns (uint256) {
        return Math.mulDiv(baseAmount, rate, SCALE);
    }

    function withdraw(
        uint256 baseAmount
    ) public whenNotPaused {
        if (!active) revert QuickWithdrawal_NotActive();
        if (baseAmount == 0) revert QuickWithdrawal_ZeroAmount();
        uint256 quoteAmount = swap(baseAmount);
        if (ERC20(QUOTE_TOKEN).balanceOf(address(this)) < quoteAmount) {
            revert QuickWithdrawal_MustBeReplenished();
        }
        ERC20(BASE_TOKEN).transferFrom(msg.sender, address(this), baseAmount);
        ERC20(QUOTE_TOKEN).transfer(msg.sender, quoteAmount);
    }

    function _setRate(
        uint256 _rate
    ) internal {
        if (_rate > SCALE) {
            revert QuickWithdrawal_RateTooHigh();
        }
        rate = _rate;
    }

    function setRate(
        uint256 _rate
    ) external restricted whenNotPaused {
        _setRate(_rate);
    }

    function setActive(
        bool _active
    ) external restricted whenNotPaused {
        active = _active;
    }

    function adminWithdraw(
        address destination
    ) external restricted whenNotPaused {
        if (active) revert QuickWithdrawal_Active();
        ERC20(BASE_TOKEN).transfer(
            destination, ERC20(BASE_TOKEN).balanceOf(address(this))
        );
        ERC20(QUOTE_TOKEN).transfer(
            destination, ERC20(QUOTE_TOKEN).balanceOf(address(this))
        );
    }

    function _authorizeUpgrade(
        address
    ) internal override restricted {}

    function pause() public restricted {
        _pause();
    }

    function unpause() public restricted {
        _unpause();
    }
}

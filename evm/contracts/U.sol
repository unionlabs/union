pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";

import "./internal/Versioned.sol";

contract U is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    ERC20Upgradeable,
    Versioned
{
    error UDare();

    bytes32 internal constant U_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.zkgm.u")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct UStorage {
        address minter;
        uint8 decimals;
        bytes salt;
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
        address _minter,
        string calldata _name,
        string calldata _symbol,
        uint8 _decimals,
        bytes calldata _salt
    ) external initializer {
        __U_init(_authority, _minter, _name, _symbol, _decimals, _salt);
    }

    function __U_init(
        address _authority,
        address _minter,
        string calldata _name,
        string calldata _symbol,
        uint8 _decimals,
        bytes calldata _salt
    ) internal onlyInitializing {
        __UUPSUpgradeable_init();
        __AccessManaged_init(_authority);
        __ERC20_init(_name, _symbol);
        UStorage storage $ = _getUStorage();
        $.minter = _minter;
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

    function transfer(address to, uint256 value) public override returns (bool) {
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

    function burn(address from, uint256 amount) external onlyMinter {
        _burn(from, amount);
    }

    modifier onlyMinter() {
        if (msg.sender != _getUStorage().minter) {
            revert UDare();
        }
        _;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}
}

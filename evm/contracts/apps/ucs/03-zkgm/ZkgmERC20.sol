pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/access/Ownable2StepUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";

import "../../../internal/Versioned.sol";

import "./IZkgmERC20.sol";

contract ZkgmERC20 is
    Initializable,
    UUPSUpgradeable,
    Ownable2StepUpgradeable,
    ERC20Upgradeable,
    Versioned,
    IZkgmERC20
{
    error ERC20Unauthorized();

    bytes32 internal constant ZKGM_ERC20_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.zkgm.erc20")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct ZkgmERC20Storage {
        address minter;
        uint8 decimals;
    }

    function _getZkgmERC20Storage()
        private
        pure
        returns (ZkgmERC20Storage storage $)
    {
        bytes32 slot = ZKGM_ERC20_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _admin,
        address _minter,
        string memory _name,
        string memory _symbol,
        uint8 _decimals
    ) external initializer {
        __ZkgmERC20_init(_admin, _minter, _name, _symbol, _decimals);
    }

    function __ZkgmERC20_init(
        address _admin,
        address _minter,
        string memory _name,
        string memory _symbol,
        uint8 _decimals
    ) internal onlyInitializing {
        __Ownable_init(_admin);
        __UUPSUpgradeable_init();
        __ERC20_init(_name, _symbol);
        ZkgmERC20Storage storage $ = _getZkgmERC20Storage();
        $.minter = _minter;
        $.decimals = _decimals;
    }

    function decimals()
        public
        view
        override(ERC20Upgradeable, IERC20Metadata)
        returns (uint8)
    {
        return _getZkgmERC20Storage().decimals;
    }

    function mint(address to, uint256 amount) external onlyMinter {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyMinter {
        _burn(from, amount);
    }

    modifier onlyMinter() {
        if (msg.sender != _getZkgmERC20Storage().minter) {
            revert ERC20Unauthorized();
        }
        _;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}
}

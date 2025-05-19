pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/token/ERC721/ERC721Upgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/token/ERC721/extensions/ERC721URIStorageUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";

import "../../../internal/Versioned.sol";

contract ZkgmERC721 is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    ERC721URIStorageUpgradeable,
    Versioned
{
    error ZkgmERC721_Unauthorized();

    bytes32 internal constant ZKGM_ERC721_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.zkgm.erc721")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct ZkgmERC721Storage {
        address minter;
    }

    function _getZkgmERC721Storage()
        private
        pure
        returns (ZkgmERC721Storage storage $)
    {
        bytes32 slot = ZKGM_ERC721_STORAGE_SLOT;
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
        string memory _name,
        string memory _symbol
    ) external initializer {
        __AccessManaged_init(_authority);
        __UUPSUpgradeable_init();
        __ERC721_init(_name, _symbol);
        ZkgmERC721Storage storage $ = _getZkgmERC721Storage();
        $.minter = _minter;
    }

    function mint(uint256 tokenId, address to) external onlyMinter {
        _mint(to, tokenId);
    }

    function burn(
        uint256 tokenId
    ) external onlyMinter {
        _burn(tokenId);
    }

    modifier onlyMinter() {
        if (msg.sender != _getZkgmERC721Storage().minter) {
            revert ZkgmERC721_Unauthorized();
        }
        _;
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}
}

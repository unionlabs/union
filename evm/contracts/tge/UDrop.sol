pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";

import "solady/tokens/ERC20.sol";
import "solady/utils/MerkleProofLib.sol";
import "solady/utils/EfficientHashLib.sol";

import "../internal/Versioned.sol";

contract UDrop is
    Initializable,
    UUPSUpgradeable,
    PausableUpgradeable,
    AccessManagedUpgradeable,
    Versioned
{
    error UDrop_ZeroAddress();
    error UDrop_AlreadyClaimed();
    error UDrop_InvalidProof();
    error UDrop_NotActive();
    error UDrop_ClaimIsActive();

    bytes32 public immutable ROOT;
    address public immutable TOKEN;

    bool public active;
    mapping(address => bool) public claimed;

    constructor(bytes32 _root, address _token) {
        _disableInitializers();
        ROOT = _root;
        TOKEN = _token;
    }

    function initialize(
        address _authority,
        bool _active
    ) external initializer {
        __UUPSUpgradeable_init();
        __Pausable_init();
        __AccessManaged_init(_authority);
        active = _active;
    }

    function claim(
        address beneficiary,
        uint256 amount,
        bytes32[] calldata proof
    ) external whenNotPaused {
        if (!active) revert UDrop_NotActive();
        if (claimed[beneficiary]) revert UDrop_AlreadyClaimed();
        bytes32 leaf =
            EfficientHashLib.hash(abi.encodePacked(beneficiary, amount));
        if (!MerkleProofLib.verifyCalldata(proof, ROOT, leaf)) {
            revert UDrop_InvalidProof();
        }
        claimed[beneficiary] = true;
        ERC20(TOKEN).transfer(beneficiary, amount);
    }

    function setActive(
        bool _active
    ) external restricted whenNotPaused {
        active = _active;
    }

    function withdraw(
        address destination
    ) external restricted whenNotPaused {
        if (active) revert UDrop_ClaimIsActive();
        ERC20(TOKEN).transfer(
            destination, ERC20(TOKEN).balanceOf(address(this))
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

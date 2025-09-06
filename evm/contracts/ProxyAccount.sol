pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import "solady/utils/Multicallable.sol";
import "solady/utils/LibCall.sol";

import "./apps/ucs/03-zkgm/IZkgmable.sol";

contract ProxyAccount is
    Initializable,
    UUPSUpgradeable,
    Multicallable,
    IZkgmable
{
    using LibCall for *;

    error ProxyAccount_IntentUnsupported();
    error ProxyAccount_ZeroZkgm();
    error ProxyAccount_OnlyZkgm();
    error ProxyAccount_OnlyAdmin();
    error ProxyAccount_ZeroAdmin();

    event LocalAdminAdded(address indexed admin);
    event LocalAdminRemoved(address indexed admin);
    event RemoteAdminAdded(
        uint256 indexed path, uint32 indexed channelId, bytes admin
    );
    event RemoteAdminRemoved(
        uint256 indexed path, uint32 indexed channelId, bytes admin
    );

    bytes32 internal constant _ACCOUNT_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.account")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct AccountStorage {
        address zkgm;
        mapping(uint256 => mapping(uint32 => mapping(bytes => bool)))
            remoteAdmins;
        mapping(address => bool) localAdmins;
    }

    constructor() {
        _disableInitializers();
    }

    function _getAccountStorage()
        private
        pure
        returns (AccountStorage storage $)
    {
        bytes32 slot = _ACCOUNT_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyAdmin {}

    function initializeLocal(
        address _admin
    ) public initializer {
        __UUPSUpgradeable_init();
        _addLocalAdmin(_admin);
    }

    function initializeRemote(
        address _zkgm,
        uint256 _path,
        uint32 _channelId,
        bytes calldata _sender
    ) public initializer {
        __UUPSUpgradeable_init();
        AccountStorage storage $ = _getAccountStorage();
        $.zkgm = _zkgm;
        _addRemoteAdmin(_path, _channelId, _sender);
    }

    function zkgm() public view returns (address) {
        return _getAccountStorage().zkgm;
    }

    function isLocalAdmin(
        address admin
    ) public view returns (bool) {
        return _getAccountStorage().localAdmins[admin];
    }

    function isRemoteAdmin(
        uint256 path,
        uint32 channelId,
        bytes calldata admin
    ) public view returns (bool) {
        return _getAccountStorage().remoteAdmins[path][channelId][admin];
    }

    function setZkgm(
        address zkgm
    ) public onlyAdmin {
        _getAccountStorage().zkgm = zkgm;
    }

    function addLocalAdmin(
        address admin
    ) public onlyAdmin {
        _addLocalAdmin(admin);
    }

    function removeLocalAdmin(
        address admin
    ) public onlyAdmin {
        _removeLocalAdmin(admin);
    }

    function addRemoteAdmin(
        uint256 path,
        uint32 channelId,
        bytes calldata admin
    ) public onlyAdmin {
        _addRemoteAdmin(path, channelId, admin);
    }

    function removeRemoteAdmin(
        uint256 path,
        uint32 channelId,
        bytes calldata admin
    ) public onlyAdmin {
        _removeRemoteAdmin(path, channelId, admin);
    }

    function _addLocalAdmin(
        address admin
    ) internal {
        if (admin == address(0)) revert ProxyAccount_ZeroAdmin();
        _getAccountStorage().localAdmins[admin] = true;
        emit LocalAdminAdded(admin);
    }

    function _removeLocalAdmin(
        address admin
    ) internal {
        _getAccountStorage().localAdmins[admin] = false;
        emit LocalAdminRemoved(admin);
    }

    function _addRemoteAdmin(
        uint256 path,
        uint32 channelId,
        bytes calldata admin
    ) internal {
        _getAccountStorage().remoteAdmins[path][channelId][admin] = true;
        emit RemoteAdminAdded(path, channelId, admin);
    }

    function _removeRemoteAdmin(
        uint256 path,
        uint32 channelId,
        bytes calldata admin
    ) internal {
        _getAccountStorage().remoteAdmins[path][channelId][admin] = false;
        emit RemoteAdminRemoved(path, channelId, admin);
    }

    function execute(
        address target,
        uint256 value,
        bytes memory payload
    ) public onlyAdmin {
        _execute(target, value, payload);
    }

    function _execute(
        address target,
        uint256 value,
        bytes memory payload
    ) internal {
        (bool success,, bytes memory result) =
            target.tryCall(value, gasleft(), type(uint16).max, payload);
        if (!success) {
            LibCall.bubbleUpRevert(result);
        }
    }

    function multicall(
        bytes[] calldata data
    ) public payable override onlyAdmin returns (bytes[] memory) {
        return super.multicall(data);
    }

    function onZkgm(
        address caller,
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata sender,
        bytes calldata message,
        address relayer,
        bytes calldata relayerMsg
    ) external override onlyZkgm {
        if (!isRemoteAdmin(path, destinationChannelId, sender)) {
            revert ProxyAccount_OnlyAdmin();
        }
        (address target, uint256 value, bytes memory payload) =
            abi.decode(message, (address, uint256, bytes));
        _execute(target, value, payload);
    }

    function onIntentZkgm(
        address caller,
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata sender,
        bytes calldata message,
        address marketMaker,
        bytes calldata marketMakerMsg
    ) external override onlyZkgm {
        revert ProxyAccount_IntentUnsupported();
    }

    modifier onlyZkgm() {
        address zkgm = _getAccountStorage().zkgm;
        if (zkgm == address(0)) revert ProxyAccount_ZeroZkgm();
        if (msg.sender != zkgm) revert ProxyAccount_OnlyZkgm();
        _;
    }

    modifier onlyAdmin() {
        bool notSelf = msg.sender != address(this);
        bool notAdmin = !isLocalAdmin(msg.sender);
        if (notSelf && notAdmin) revert ProxyAccount_OnlyAdmin();
        _;
    }
}

pragma solidity ^0.8.27;

import "../24-host/IBCStore.sol";
import "../02-client/IBCClient.sol";
import "../03-connection/IBCConnection.sol";
import "../04-channel/IBCChannel.sol";
import "../04-channel/IBCPacket.sol";

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/ContextUpgradeable.sol";
import "@openzeppelin/utils/Context.sol";

/**
 * @dev IBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
abstract contract IBCHandler is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    IBCStore,
    IBCClient,
    IBCConnectionImpl,
    IBCChannelImpl,
    IBCPacketImpl
{
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address admin
    ) public virtual initializer {
        __Ownable_init(admin);
        __UUPSUpgradeable_init();
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}
}

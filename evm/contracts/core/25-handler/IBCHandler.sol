pragma solidity ^0.8.27;

import "../24-host/IBCStore.sol";
import "../02-client/IBCClient.sol";
import "../03-connection/IBCConnection.sol";
import "../04-channel/IBCChannel.sol";
import "../04-channel/IBCPacket.sol";

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/ContextUpgradeable.sol";
import "@openzeppelin/contracts/utils/Context.sol";

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

        commitments[nextClientSequencePath] = bytes32(uint256(1));
        commitments[nextChannelSequencePath] = bytes32(uint256(1));
        commitments[nextConnectionSequencePath] = bytes32(uint256(1));
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}
}

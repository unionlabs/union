pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import "../24-host/IBCStore.sol";
import "../02-client/IBCClient.sol";
import "../03-connection/IBCConnection.sol";
import "../04-channel/IBCChannel.sol";
import "../04-channel/IBCPacket.sol";
import "../../internal/Versioned.sol";

/**
 * @dev IBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
contract IBCHandler is
    Initializable,
    UUPSUpgradeable,
    IBCStore,
    IBCClient,
    IBCConnectionImpl,
    IBCChannelImpl,
    IBCPacketImpl,
    Versioned
{
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address authority
    ) external initializer {
        __IBCHandler_init(authority);
    }

    function __IBCHandler_init(
        address authority
    ) internal onlyInitializing {
        __AccessManaged_init(authority);
        __UUPSUpgradeable_init();
        commitments[nextClientSequencePath] = bytes32(uint256(1));
        commitments[nextChannelSequencePath] = bytes32(uint256(1));
        commitments[nextConnectionSequencePath] = bytes32(uint256(1));
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}
}

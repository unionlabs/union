pragma solidity ^0.8.23;

import "solady/utils/LibString.sol";
import "./25-handler/IBCHandler.sol";

/**
 * @dev OwnableIBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
contract OwnableIBCHandler is IBCHandler {
    using LibString for *;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address ibcClient,
        address ibcConnection,
        address ibcChannel,
        address ibcPacket,
        address admin
    ) public override initializer {
        IBCHandler.initialize(
            ibcClient, ibcConnection, ibcChannel, ibcPacket, admin
        );
    }

    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(
        string calldata clientType,
        ILightClient client
    ) public override onlyOwner {
        super.registerClient(clientType, client);
    }
}

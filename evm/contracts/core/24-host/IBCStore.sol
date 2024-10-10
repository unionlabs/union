pragma solidity ^0.8.27;

import "../02-client/ILightClient.sol";
import "../05-port/IIBCModule.sol";
import "../Types.sol";

library IBCStoreLib {
    bytes32 public constant COMMITMENT_PREFIX = keccak256("ethibc");
}

abstract contract IBCStore {
    bytes32 public constant COMMITMENT_PREFIX = IBCStoreLib.COMMITMENT_PREFIX;

    // Commitments
    // keccak256(IBC-compatible-store-path) => keccak256(IBC-compatible-commitment)
    mapping(bytes32 => bytes32) public commitments;

    // ClientType -> Address
    mapping(string => address) public clientRegistry;
    // ClientId -> ClientType
    mapping(uint32 => string) public clientTypes;
    // ClientId -> Address
    mapping(uint32 => address) public clientImpls;
    // ConnectionId -> Connection
    mapping(uint32 => IBCConnection) public connections;
    // ChannelId -> Channel
    mapping(uint32 => IBCChannel) public channels;
    // ChannelId -> PortId
    mapping(uint32 => address) public channelOwner;

    // Sequences for identifier
    bytes32 public constant nextClientSequencePath =
        keccak256("nextClientSequence");
    bytes32 public constant nextConnectionSequencePath =
        keccak256("nextConnectionSequence");
    bytes32 public constant nextChannelSequencePath =
        keccak256("nextChannelSequence");

    function getClient(
        uint32 clientId
    ) public view returns (ILightClient) {
        return getClientInternal(clientId);
    }

    function getClientInternal(
        uint32 clientId
    ) internal view returns (ILightClient) {
        address clientImpl = clientImpls[clientId];
        if (clientImpl == address(0)) {
            revert IBCErrors.ErrClientNotFound();
        }
        return ILightClient(clientImpl);
    }

    function lookupModuleByChannel(
        uint32 channelId
    ) internal view virtual returns (IIBCModule) {
        address module = channelOwner[channelId];
        if (module == address(0)) {
            revert IBCErrors.ErrModuleNotFound();
        }
        return IIBCModule(module);
    }

    function claimChannel(address portId, uint32 channelId) internal {
        channelOwner[channelId] = portId;
    }

    function authenticateChannelOwner(
        uint32 channelId
    ) internal view returns (bool) {
        return msg.sender == channelOwner[channelId];
    }

    function ensureConnectionState(
        uint32 connectionId
    ) internal view returns (uint32) {
        IBCConnection storage connection = connections[connectionId];
        if (connection.state != IBCConnectionState.Open) {
            revert IBCErrors.ErrInvalidConnectionState();
        }
        return connection.clientId;
    }

    function ensureChannelState(
        uint32 channelId
    ) internal view returns (IBCChannel storage) {
        IBCChannel storage channel = channels[channelId];
        if (channel.state != IBCChannelState.Open) {
            revert IBCErrors.ErrInvalidChannelState();
        }
        return channel;
    }
}

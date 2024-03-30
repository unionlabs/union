pragma solidity ^0.8.23;

// import "@openzeppelin/utils/Strings.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";

type ChannelId is bytes32;

error InvalidChannelId();

// :) https://github.com/ethereum/solidity/issues/1256
function parseChannelIdCalldata(string calldata channelId)
    pure
    returns (ChannelId)
{
    if (bytes(channelId).length > 32) {
        revert InvalidChannelId();
    } else {
        return ChannelId.wrap(bytes32(bytes(channelId)));
    }
}

function parseChannelIdMemory(string memory channelId)
    pure
    returns (ChannelId)
{
    if (bytes(channelId).length > 32) {
        revert InvalidChannelId();
    } else {
        return ChannelId.wrap(bytes32(bytes(channelId)));
    }
}

function toString(ChannelId channelId) pure returns (string memory) {
    bytes memory bz = abi.encodePacked(channelId);

    uint256 ptr = 0;

    while (bz[ptr] != 0) {
        ptr++;
    }

    assembly {
        mstore(bz, ptr)
    }

    return string(bz);
}

function equals(ChannelId a, ChannelId b) pure returns (bool) {
    return ChannelId.unwrap(a) == ChannelId.unwrap(b);
}

function notEquals(ChannelId a, ChannelId b) pure returns (bool) {
    return ChannelId.unwrap(a) != ChannelId.unwrap(b);
}

using {toString, equals as ==, notEquals as !=} for ChannelId global;

// TODO: Remove this library and just expose the structs and functions directly?
library IBCChannelTypes {
    using {parseChannelIdCalldata} for string;

    struct Channel {
        IbcCoreChannelV1GlobalEnums.State state;
        IbcCoreChannelV1GlobalEnums.Order ordering;
        Counterparty counterparty;
        string[] connectionHops;
        string version;
    }

    function optimizedChannel(IbcCoreChannelV1Channel.Data calldata channel)
        internal
        pure
        returns (Channel memory)
    {
        return Channel({
            state: channel.state,
            ordering: channel.ordering,
            counterparty: optimizedCounterparty(channel.counterparty),
            connectionHops: channel.connection_hops,
            version: channel.version
        });
    }

    struct Counterparty {
        ChannelId channelId;
        string portId;
    }

    function optimizedCounterparty(
        IbcCoreChannelV1Counterparty.Data calldata channelCounterparty
    ) internal pure returns (Counterparty memory) {
        return Counterparty({
            channelId: channelCounterparty.channel_id.parseChannelIdCalldata(),
            portId: channelCounterparty.port_id
        });
    }
}

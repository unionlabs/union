pragma solidity ^0.8.23;

import "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import "../../../contracts/core/04-channel/IBCChannelTypes.sol";
import "forge-std/Test.sol";

contract IBCChannelTypesTest is Test {
    using {parseChannelIdMemory} for string;

    function test_channelId() public {
        assertEq(
            string("channel-id").parseChannelIdMemory().toString(),
            string("channel-id")
        );

        assertEq(
            string("thirty-one-character-channel-id").parseChannelIdMemory()
                .toString(),
            string("thirty-one-character-channel-id")
        );

        assertEq(
            string("thirty-two-character-channel-id-").parseChannelIdMemory()
                .toString(),
            string("thirty-two-character-channel-id-")
        );
    }

    function test_channelId(string memory channelId) public {
        vm.assume(bytes(channelId).length <= 32);
        assertEq(channelId.parseChannelIdMemory().toString(), channelId);
    }
}

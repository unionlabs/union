// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "../../ucs/03-zkgm/IZkgm.sol";
import "../../ucs/03-zkgm/Types.sol";
import "../../ucs/03-zkgm/Lib.sol";
import "../../../core/04-channel/IBCPacket.sol";

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

struct Order {
    uint32 destinationChainId;
    bytes receiver;
    address baseToken;
    uint256 baseAmount;
    bytes quoteToken;
    uint256 quoteAmount;
    bytes32 salt;
    uint64 timeoutTimestamp;
}

contract Nexus is Ownable {
    using ZkgmLib for *; // If it uses function extensions (optional)
    using SafeERC20 for *; // If it uses function extensions (optional)

    IZkgm public zkgm;

    // Constructor to set the zkgm contract and initialize Ownable
    constructor(address _zkgm) Ownable(msg.sender) {
        require(_zkgm != address(0), "zkgm address cannot be zero");
        zkgm = IZkgm(_zkgm);
    }


    mapping(uint32 => uint32) public destinationToChannel;

    function transfer(Order memory order) public {
        // 1. Get channel ID for destination chain
        uint32 channelId = destinationToChannel[order.destinationChainId];
        require(channelId != 0, "Invalid destination chain");

        // 2. Transfer tokens from user to contract
        IERC20(order.baseToken).safeTransferFrom(
            msg.sender,
            address(this),
            order.baseAmount
        );

        IERC20(order.baseToken).approve(
            address(zkgm),
            order.baseAmount
        );

        // 3. Create fungible asset order instruction
        Instruction memory instruction = zkgm.makeFungibleAssetOrder(
            0,
            channelId,
            msg.sender,
            order.receiver,
            order.baseToken,
            order.baseAmount,
            order.quoteToken,
            order.quoteAmount
        );

        // 4. Call zkgm contract
        zkgm.send(
            channelId,
            order.timeoutTimestamp, // Could be current time + some buffer
            0, // Optional block timeout
            order.salt,
            instruction
        );
    }

    function setChannelId(uint32 destinationChainId, uint32 channelId) external onlyOwner {
        destinationToChannel[destinationChainId] = channelId;
    }
}

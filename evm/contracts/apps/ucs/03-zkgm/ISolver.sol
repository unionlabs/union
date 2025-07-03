pragma solidity ^0.8.27;

import "../../../core/Types.sol";
import "./Types.sol";

interface ISolver {
    function solve(
        IBCPacket calldata packet,
        FungibleAssetOrderV2 calldata order,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) external;
}

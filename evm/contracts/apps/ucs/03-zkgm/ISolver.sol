pragma solidity ^0.8.27;

import "../../../core/Types.sol";
import "./Types.sol";

interface ISolver {
    // Try to fill the order. Reverts if impossible.
    function solve(
        IBCPacket calldata packet,
        TokenOrderV2 calldata order,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) external;

    // Specificy whether the local ISolver is implemented by a token (likely
    // ERC20) and enables third party marker makers to provide the funds if the
    // solver itself failed to fill.
    function allowMarketMakers() external returns (bool);
}

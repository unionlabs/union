pragma solidity ^0.8.27;

import "../../../core/Types.sol";
import "./Types.sol";

interface ISolver {
    // Try to fill the order. Reverts if impossible.
    // Returns the market maker address (can return the relayerMsg if overwritting is unecessary).
    function solve(
        IBCPacket calldata packet,
        TokenOrderV2 calldata order,
        uint256 path,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) external returns (bytes memory);

    // Specificy whether the local ISolver is implemented by a token (likely
    // ERC20) and enables third party marker makers to provide the funds if the
    // solver itself failed to fill.
    function allowMarketMakers() external returns (bool);
}

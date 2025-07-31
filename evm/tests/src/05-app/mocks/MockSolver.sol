pragma solidity ^0.8.27;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/introspection/ERC165.sol";
import "../../../../contracts/core/04-channel/IBCPacket.sol";
import "../../../../contracts/apps/ucs/03-zkgm/ISolver.sol";
import "../../../../contracts/apps/ucs/03-zkgm/Zkgm.sol";

contract MockSolver is ISolver, ERC165 {
    bool public shouldFail;
    uint256 public solveCallCount;

    struct SolveCall {
        IBCPacket packet;
        TokenOrderV2 order;
        address caller;
        address relayer;
        bytes relayerMsg;
        bool intent;
    }

    SolveCall public lastCall;

    constructor() {}

    function setShouldFail(
        bool _shouldFail
    ) external {
        shouldFail = _shouldFail;
    }

    function allowMarketMakers() external override returns (bool) {
        return true;
    }

    function solve(
        IBCPacket calldata packet,
        TokenOrderV2 calldata order,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) external override {
        solveCallCount++;
        // Store the call data - can't store calldata directly in storage
        lastCall.packet = packet;
        lastCall.order = order;
        lastCall.caller = caller;
        lastCall.relayer = relayer;
        lastCall.relayerMsg = relayerMsg;
        lastCall.intent = intent;

        require(!shouldFail, "MockSolver: Configured to fail");
    }

    function supportsInterface(
        bytes4 interfaceId
    ) public view override returns (bool) {
        return interfaceId == type(ISolver).interfaceId
            || super.supportsInterface(interfaceId);
    }
}

contract MockSolverWithU is ISolver, ERC165 {
    IERC20 public immutable uToken;

    constructor(
        address _uToken
    ) {
        uToken = IERC20(_uToken);
    }

    function allowMarketMakers() external override returns (bool) {
        return true;
    }

    function solve(
        IBCPacket calldata packet,
        TokenOrderV2 calldata order,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) external override {
        // Simulate solver using U token to fulfill order
        address receiver = address(uint160(bytes20(order.receiver)));

        // Transfer U tokens instead of quote tokens
        uToken.transfer(receiver, order.quoteAmount);
    }

    function supportsInterface(
        bytes4 interfaceId
    ) public view override returns (bool) {
        return interfaceId == type(ISolver).interfaceId
            || super.supportsInterface(interfaceId);
    }
}

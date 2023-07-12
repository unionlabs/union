pragma solidity ^0.8.18;

import "./TransferPacket.sol";
import "./ICS20Transfer.sol";
import "./IICS20Bank.sol";
import "../../core/25-handler/IBCHandler.sol";

contract ICS20TransferBank is ICS20Transfer {
    using strings for *;

    IBCHandler private ibcHandler;
    IICS20Bank private bank;

    constructor(IBCHandler _ibcHandler, IICS20Bank _bank) {
        ibcHandler = _ibcHandler;
        bank = _bank;
    }

    function sendTransfer(
        string calldata denom,
        uint64 amount,
        string calldata receiver,
        string calldata sourcePort,
        string calldata sourceChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight
    ) external {
        if (
            !denom.toSlice().startsWith(
                _makeDenomPrefix(sourcePort, sourceChannel)
            )
        ) {
            // sender is source chain
            require(
                _transferFrom(
                    _msgSender(),
                    _getEscrowAddress(sourceChannel),
                    denom,
                    amount
                ),
                "_transferFrom failed"
            );
        } else {
            require(_burn(_msgSender(), denom, amount), "_burn failed");
        }

        _sendPacket(
            TransferPacket({
                denom: denom,
                amount: amount,
                sender: string(abi.encodePacked(_msgSender())),
                receiver: receiver
                // receiver: string(abi.encodePacked(receiver))
            }),
            sourcePort,
            sourceChannel,
            timeoutRevisionNumber,
            timeoutRevisionHeight
        );
    }

    function _transferFrom(
        address sender,
        address receiver,
        string memory denom,
        uint256 amount
    ) internal override returns (bool) {
        bank.transferFrom(sender, receiver, denom, amount);
        return true;
    }

    function _mint(
        address account,
        string memory denom,
        uint256 amount
    ) internal override returns (bool) {
        bank.mint(account, denom, amount);
        return true;
    }

    function _burn(
        address account,
        string memory denom,
        uint256 amount
    ) internal override returns (bool) {
        bank.burn(account, denom, amount);
        return true;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }
}

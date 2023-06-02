pragma solidity ^0.8.18;

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
        address receiver,
        string calldata sourcePort,
        string calldata sourceChannel,
        uint64 timeoutHeight
    ) external {
        if (!denom.toSlice().startsWith(_makeDenomPrefix(sourcePort, sourceChannel))) {
            // sender is source chain
            require(_transferFrom(_msgSender(), _getEscrowAddress(sourceChannel), denom, amount));
        } else {
            require(_burn(_msgSender(), denom, amount));
        }

        _sendPacket(
            IbcApplicationsTransferV2FungibleTokenPacketData.Data({
                denom: denom,
                amount: Strings.toString(amount),
                sender: string(abi.encodePacked(_msgSender())),
                receiver: string(abi.encodePacked(receiver)),
                // TODO: allow for users to dispatch memo?
                memo: ""
            }),
            sourcePort,
            sourceChannel,
            timeoutHeight
        );
    }

    function _transferFrom(address sender, address receiver, string memory denom, uint256 amount)
        internal
        override
        returns (bool)
    {
        try bank.transferFrom(sender, receiver, denom, amount) {
            return true;
        } catch (bytes memory) {
            return false;
        }
    }

    function _mint(address account, string memory denom, uint256 amount) internal override returns (bool) {
        try bank.mint(account, denom, amount) {
            return true;
        } catch (bytes memory) {
            return false;
        }
    }

    function _burn(address account, string memory denom, uint256 amount) internal override returns (bool) {
        try bank.burn(account, denom, amount) {
            return true;
        } catch (bytes memory) {
            return false;
        }
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }
}

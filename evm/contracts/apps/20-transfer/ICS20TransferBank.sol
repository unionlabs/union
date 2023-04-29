// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.18;

import "./ICS20Transfer.sol";
import "./IICS20Bank.sol";
import "../../core/25-handler/IBCHandler.sol";

contract ICS20TransferBank is ICS20Transfer {
    IICS20Bank bank;

    constructor(IBCHandler ibcHandler_, IICS20Bank bank_) ICS20Transfer(ibcHandler_) {
        bank = bank_;
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
}

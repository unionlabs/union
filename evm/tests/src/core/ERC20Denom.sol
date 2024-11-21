// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "../../../contracts/apps/ucs/01-relay/ERC20Denom.sol";

contract TestERC20DenomHandler is ERC20Denom {
    constructor(string memory denomName) ERC20Denom(denomName) {}

    /**
     * @dev Utility function to set the admin address directly for testing purposes.
     *      This would not be available in production.
     */
    function setAdmin(address newAdmin) public {
        admin = newAdmin;
    }

    /**
     * @dev Assume a balance for a specific account.
     *      Used to simulate test scenarios without minting/burning directly.
     */
    function assumeBalance(address account, uint256 balance) public {
        _mint(account, balance);
    }

    
}

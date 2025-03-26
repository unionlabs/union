// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

// nexus: 0xb4C2E37a9705e0cb556EAE7ad8D56f6BD4Be95Cb
// erc20: 0xA588C09D2fE853714d93347F5138FFAA3F7Bdf06

contract UnionToken is ERC20 {
    constructor() ERC20("Union", "UNO") {
        _mint(msg.sender, 1000000 * (10 ** uint256(decimals())));
    }
}

pragma solidity ^0.8.23;

import "@openzeppelin/token/ERC20/IERC20.sol";

interface IERC20Denom is IERC20 {
    function mint(address to, uint256 amount) external;

    function burn(address from, uint256 amount) external;
}

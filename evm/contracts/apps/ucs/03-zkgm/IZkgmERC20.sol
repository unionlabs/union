pragma solidity ^0.8.27;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

interface IZkgmERC20 is IERC20, IERC20Metadata {
    function mint(address to, uint256 amount) external;
    function burn(address from, uint256 amount) external;
}

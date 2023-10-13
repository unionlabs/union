pragma solidity ^0.8.21;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "./IERC20Denom.sol";

contract ERC20Denom is ERC20, IERC20Denom {
    address public admin;

    constructor(string memory name) ERC20(name, name) {
        admin = msg.sender;
    }

    function mint(address to, uint256 amount) external {
        require(msg.sender == admin, "only admin");
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external {
        require(msg.sender == admin, "only admin");
        _burn(from, amount);
    }
}

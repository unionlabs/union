pragma solidity ^0.8.23;

import "@openzeppelin/token/ERC20/ERC20.sol";
import "./IERC20Denom.sol";

contract ERC20Denom is ERC20, IERC20Denom {
    error ERC20Unauthorized();

    address public admin;

    constructor(string memory name) ERC20(name, name) {
        admin = msg.sender;
    }

    function mint(address to, uint256 amount) external {
        if (msg.sender != admin) {
            revert ERC20Unauthorized();
        }
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external {
        if (msg.sender != admin) {
            revert ERC20Unauthorized();
        }
        _burn(from, amount);
    }
}

pragma solidity ^0.8.23;

import "@openzeppelin/token/ERC20/ERC20.sol";
import "./IERC20Denom.sol";

contract ERC20Denom is ERC20, IERC20Denom {
    error ERC20Unauthorized();

    address public admin;

    // Metadata updated via UCS01 governance
    string private _name;
    string private _symbol;
    uint8 private _decimals;

    constructor(string memory denomName) ERC20(denomName, denomName) {
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

    function update(
        string calldata newName,
        string calldata newSymbol,
        uint8 newDecimals
    ) external {
        if (msg.sender != admin) {
            revert ERC20Unauthorized();
        }
        _name = newName;
        _symbol = newSymbol;
        _decimals = newDecimals;
    }

    function name()
        public
        view
        override(ERC20, IERC20Metadata)
        returns (string memory)
    {
        return _name;
    }

    function symbol()
        public
        view
        override(ERC20, IERC20Metadata)
        returns (string memory)
    {
        return _symbol;
    }

    function decimals()
        public
        view
        override(ERC20, IERC20Metadata)
        returns (uint8)
    {
        return _decimals;
    }
}

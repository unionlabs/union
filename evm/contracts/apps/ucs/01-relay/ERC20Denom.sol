pragma solidity ^0.8.27;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "./IERC20Denom.sol";

contract ERC20Denom is ERC20, IERC20Denom {
    error ERC20Unauthorized();

    address public admin;

    string private _name;
    string private _symbol;
    uint8 private _decimals;

    constructor(
        string memory denomName
    ) ERC20(denomName, denomName) {
        admin = msg.sender;
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

    function mint(address to, uint256 amount) external onlyAdmin {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyAdmin {
        _burn(from, amount);
    }

    function update(
        string calldata newName,
        string calldata newSymbol,
        uint8 newDecimals
    ) external onlyAdmin {
        _name = newName;
        _symbol = newSymbol;
        _decimals = newDecimals;
    }

    modifier onlyAdmin() {
        _checkAdmin();
        _;
    }

    function _checkAdmin() internal view virtual {
        if (msg.sender != admin) {
            revert ERC20Unauthorized();
        }
    }
}

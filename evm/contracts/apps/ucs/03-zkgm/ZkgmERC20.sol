pragma solidity ^0.8.27;

import "@openzeppelin/token/ERC20/ERC20.sol";
import "./IZkgmERC20.sol";

contract ZkgmERC20 is ERC20, IZkgmERC20 {
    error ERC20Unauthorized();

    address public admin;
    uint8 private _decimals;

    constructor(string memory n, string memory s, address a) ERC20(n, s) {
        admin = a;
        _decimals = 18;
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

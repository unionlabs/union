pragma solidity ^0.8.27;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "./IZkgmERC20.sol";

contract ZkgmERC20 is ERC20, IZkgmERC20 {
    error ERC20Unauthorized();

    address public immutable ADMIN;
    uint8 public immutable DECIMALS;

    constructor(
        string memory _name,
        string memory _symbol,
        uint8 _decimals,
        address _admin
    ) ERC20(_name, _symbol) {
        ADMIN = _admin;
        DECIMALS = _decimals;
    }

    function decimals()
        public
        view
        override(ERC20, IERC20Metadata)
        returns (uint8)
    {
        return DECIMALS;
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
        if (msg.sender != ADMIN) {
            revert ERC20Unauthorized();
        }
    }
}

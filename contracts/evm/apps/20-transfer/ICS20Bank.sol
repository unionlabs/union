// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/utils/Context.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import "./IICS20Bank.sol";

contract ICS20Bank is Context, AccessControl, IICS20Bank {
    using Address for address;

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");

    // Mapping from token ID to account balances
    mapping(string => mapping(address => uint256)) private _balances;

    constructor() {
        _setupRole(ADMIN_ROLE, _msgSender());
    }

    function setOperator(address operator) public virtual {
        require(hasRole(ADMIN_ROLE, _msgSender()), "must have admin role to set new operator");
        _setupRole(OPERATOR_ROLE, operator);
    }

    function balanceOf(address account, string calldata id) external view virtual returns (uint256) {
        require(account != address(0), "ICS20Bank: balance query for the zero address");
        return _balances[id][account];
    }

    function transferFrom(address from, address to, string calldata id, uint256 amount) external virtual override {
        require(to != address(0), "ICS20Bank: transfer to the zero address");
        require(
            from == _msgSender() || hasRole(OPERATOR_ROLE, _msgSender()), "ICS20Bank: caller is not owner nor approved"
        );

        uint256 fromBalance = _balances[id][from];
        require(fromBalance >= amount, "ICS20Bank: insufficient balance for transfer");
        _balances[id][from] = fromBalance - amount;
        _balances[id][to] += amount;
    }

    function mint(address account, string calldata id, uint256 amount) external virtual override {
        require(hasRole(OPERATOR_ROLE, _msgSender()), "ICS20Bank: must have minter role to mint");
        _mint(account, id, amount);
    }

    function burn(address account, string calldata id, uint256 amount) external virtual override {
        require(hasRole(OPERATOR_ROLE, _msgSender()), "ICS20Bank: must have minter role to mint");
        _burn(account, id, amount);
    }

    function deposit(address tokenContract, uint256 amount, address receiver) external virtual {
        require(tokenContract.isContract());
        require(IERC20(tokenContract).transferFrom(_msgSender(), address(this), amount));
        _mint(receiver, _genDenom(tokenContract), amount);
    }

    function withdraw(address tokenContract, uint256 amount, address receiver) external virtual {
        require(tokenContract.isContract());
        _burn(_msgSender(), _genDenom(tokenContract), amount);
        require(IERC20(tokenContract).transfer(receiver, amount));
    }

    function _mint(address account, string memory id, uint256 amount) internal virtual {
        _balances[id][account] += amount;
    }

    function _burn(address account, string memory id, uint256 amount) internal virtual {
        uint256 accountBalance = _balances[id][account];
        require(accountBalance >= amount, "ICS20Bank: burn amount exceeds balance");
        _balances[id][account] = accountBalance - amount;
    }

    function _genDenom(address tokenContract) internal pure virtual returns (string memory) {
        return addressToString(tokenContract);
    }

    /// Helper functions ///
    // TODO move following functions into any library
    function addressToString(address _address) internal pure returns (string memory) {
        bytes memory alphabet = "0123456789abcdef";
        bytes20 data = bytes20(_address);

        bytes memory str = new bytes(42);
        str[0] = "0";
        str[1] = "x";
        for (uint256 i = 0; i < 20; i++) {
            str[2 + i * 2] = alphabet[uint256(uint8(data[i] >> 4))];
            str[2 + 1 + i * 2] = alphabet[uint256(uint8(data[i] & 0x0f))];
        }
        return string(str);
    }

    // a copy from https://github.com/provable-things/ethereum-api/blob/161552ebd4f77090d86482cff8c863cf903c6f5f/oraclizeAPI_0.6.sol
    function parseAddr(string memory _a) internal pure returns (address _parsedAddress) {
        bytes memory tmp = bytes(_a);
        uint160 iaddr = 0;
        uint160 b1;
        uint160 b2;
        for (uint256 i = 2; i < 2 + 2 * 20; i += 2) {
            iaddr *= 256;
            b1 = uint160(uint8(tmp[i]));
            b2 = uint160(uint8(tmp[i + 1]));
            if ((b1 >= 97) && (b1 <= 102)) {
                b1 -= 87;
            } else if ((b1 >= 65) && (b1 <= 70)) {
                b1 -= 55;
            } else if ((b1 >= 48) && (b1 <= 57)) {
                b1 -= 48;
            }
            if ((b2 >= 97) && (b2 <= 102)) {
                b2 -= 87;
            } else if ((b2 >= 65) && (b2 <= 70)) {
                b2 -= 55;
            } else if ((b2 >= 48) && (b2 <= 57)) {
                b2 -= 48;
            }
            iaddr += (b1 * 16 + b2);
        }
        return address(iaddr);
    }
}

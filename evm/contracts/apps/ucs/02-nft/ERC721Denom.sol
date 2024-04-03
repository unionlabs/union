pragma solidity ^0.8.23;

import "@openzeppelin/token/ERC721/ERC721.sol";
import "./IERC721Denom.sol";

contract ERC721Denom is ERC721, IERC721Denom {
    error ERC721Unauthorized();

    address public admin;

    constructor(string memory name) ERC721(name, name) {
        admin = msg.sender;
    }

    function mint(address to, uint256 tokenId) external {
        if (msg.sender != admin) {
            revert ERC721Unauthorized();
        }
        _mint(to, tokenId);
    }

    function burn(uint256 tokenId) external {
        if (msg.sender != admin) {
            revert ERC721Unauthorized();
        }
        _burn(tokenId);
    }
}

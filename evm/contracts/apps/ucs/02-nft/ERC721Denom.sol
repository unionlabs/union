pragma solidity ^0.8.27;

import "@openzeppelin/token/ERC721/extensions/ERC721URIStorage.sol";
import "./IERC721Denom.sol";

contract ERC721Denom is ERC721URIStorage, IERC721Denom {
    error ERC721Unauthorized();

    address public admin;

    constructor(
        string memory name,
        string memory symbol
    ) ERC721(name, symbol) {
        admin = msg.sender;
    }

    function mint(
        address to,
        uint256 tokenId,
        string calldata tokenURI
    ) external {
        if (msg.sender != admin) {
            revert ERC721Unauthorized();
        }
        _safeMint(to, tokenId);
        _setTokenURI(tokenId, tokenURI);
    }

    function burn(
        uint256 tokenId
    ) external {
        if (msg.sender != admin) {
            revert ERC721Unauthorized();
        }
        _burn(tokenId);
    }
}

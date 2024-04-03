pragma solidity ^0.8.23;

import "@openzeppelin/token/ERC721/IERC721.sol";

interface IERC721Denom is IERC721 {
    function mint(address to, uint256 tokenId) external;

    function burn(uint256 tokenId) external;
}

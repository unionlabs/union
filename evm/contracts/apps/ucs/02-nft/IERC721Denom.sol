pragma solidity ^0.8.27;

import "@openzeppelin/contracts/token/ERC721/IERC721.sol";

interface IERC721Denom is IERC721 {
    function mint(
        address to,
        uint256 tokenId,
        string calldata tokenURI
    ) external;

    function burn(
        uint256 tokenId
    ) external;
}

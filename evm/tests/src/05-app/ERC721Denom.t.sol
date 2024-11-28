// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/apps/ucs/02-nft/ERC721Denom.sol";

contract ERC721DenomTests is Test {
    ERC721Denom erc721Denom;
    address admin = address(0xABcD);
    address user = address(0x1234);
    address randomUser = address(0x5678);

    error ERC721NonexistentToken(uint256 token);

    function setUp() public {
        // Deploy the ERC721Denom contract
        vm.startPrank(admin); // Set admin as the sender for deployment
        erc721Denom = new ERC721Denom("TestNFT", "TNFT");
        vm.stopPrank();
    }

    // Test minting functionality
    function test_mint_success() public {
        uint256 tokenId = 1;
        string memory tokenURI = "ipfs://mockURI";

        vm.startPrank(admin); // Only admin can mint
        erc721Denom.mint(user, tokenId, tokenURI);
        vm.stopPrank();

        // Check ownership and token URI
        assertEq(erc721Denom.ownerOf(tokenId), user, "Owner mismatch");
        assertEq(erc721Denom.tokenURI(tokenId), tokenURI, "Token URI mismatch");
    }

    function test_mint_unauthorized_reverts() public {
        uint256 tokenId = 1;
        string memory tokenURI = "ipfs://mockURI";

        // Attempt to mint from a non-admin address
        vm.startPrank(randomUser);
        vm.expectRevert(
            abi.encodeWithSelector(ERC721Denom.ERC721Unauthorized.selector)
        );
        erc721Denom.mint(user, tokenId, tokenURI);
        vm.stopPrank();
    }

    // Test burning functionality
    function test_burn_success() public {
        uint256 tokenId = 1;
        string memory tokenURI = "ipfs://mockURI";

        // Mint a token first
        vm.startPrank(admin);
        erc721Denom.mint(user, tokenId, tokenURI);
        vm.stopPrank();

        // Burn the token
        vm.startPrank(admin);
        erc721Denom.burn(tokenId);
        vm.stopPrank();

        // Verify that the token no longer exists
        vm.expectRevert(
            abi.encodeWithSelector(ERC721NonexistentToken.selector, tokenId)
        );
        erc721Denom.ownerOf(tokenId);
    }

    function test_burn_unauthorized_reverts() public {
        uint256 tokenId = 1;
        string memory tokenURI = "ipfs://mockURI";

        // Mint a token first
        vm.startPrank(admin);
        erc721Denom.mint(user, tokenId, tokenURI);
        vm.stopPrank();

        // Attempt to burn from a non-admin address
        vm.startPrank(randomUser);
        vm.expectRevert(
            abi.encodeWithSelector(ERC721Denom.ERC721Unauthorized.selector)
        );
        erc721Denom.burn(tokenId);
        vm.stopPrank();
    }

    // Test unauthorized access to mint and burn functions
    function test_unauthorized_access_reverts() public {
        uint256 tokenId = 1;
        string memory tokenURI = "ipfs://mockURI";

        // Unauthorized user tries to mint
        vm.startPrank(user);
        vm.expectRevert(
            abi.encodeWithSelector(ERC721Denom.ERC721Unauthorized.selector)
        );
        erc721Denom.mint(user, tokenId, tokenURI);
        vm.stopPrank();

        // Mint with admin
        vm.startPrank(admin);
        erc721Denom.mint(user, tokenId, tokenURI);
        vm.stopPrank();

        // Unauthorized user tries to burn
        vm.startPrank(randomUser);
        vm.expectRevert(
            abi.encodeWithSelector(ERC721Denom.ERC721Unauthorized.selector)
        );
        erc721Denom.burn(tokenId);
        vm.stopPrank();
    }

    // Test token properties after deployment
    function test_token_properties() public {
        assertEq(erc721Denom.name(), "TestNFT", "Token name mismatch");
        assertEq(erc721Denom.symbol(), "TNFT", "Token symbol mismatch");
        assertEq(erc721Denom.admin(), admin, "Admin address mismatch");
    }
}

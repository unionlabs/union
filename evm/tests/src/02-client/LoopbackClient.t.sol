// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/clients/LoopbackClient.sol";

contract LoopbackClientLibWrapper {
    function toEvmHeader(
        bytes calldata encodedHeader
    ) external pure returns (EvmHeader memory) {
        return LoopbackClientLib.toEvmHeader(encodedHeader);
    }
}

contract LoopbackClientTest is Test {
    LoopbackClientLibWrapper wrapper;

    function setUp() public {
        wrapper = new LoopbackClientLibWrapper();
    }

    // Real Ethereum mainnet block header (block 0x16c3ee5 = 23920357)
    // Block hash: 0xda159f02094bdb8d4b3c79a986bd8c96d188d70173291bbac9c54afbc183f9c7
    function getEthMainnetHeader() internal pure returns (bytes memory) {
        return
        hex"f90283a0405ca7dc228d2448fe7a1e20d9ff09b7628f874f7d73a8d0799af07f4f0f33cda01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794dadb0d80178819f2319190d340ce9a924f783711a03350c8652844a5728f2a73d19c970d5f0a9f6690935eea22d455fb03f3efb277a00e1e90b4dd6c020c06389e4693ffe9db5624b97ec25b89ff0c0e932c84f2b136a0e71bbd25b59291affcae41e01a70360a386eb9a1cb13c5bfeb781e5dc0fda005b9010011a0006751404b9431a20069c6e000011381013a5446200c228081082428c0835c0421f8b8ba108026103b61320605210e21e0284c083d8303b58c0149245010b400946200418219a8060a08c051603400b00012805741220c22dc07812021d0101f044806620362c00e1b080a992c8802230c0205056e020ba411d4001b05b8c2203008d4a0041850211620052024161114890989009c2a242008c0603447080b0b08400280240d421808e0988020982402efc62020800e006a852d048c6340d25019265004eb117000083462709880b084024b470c00960944004640003ecf0832bd782207c0008044d419890010a055698000011908c0201860526a0570008084016c3ee58402bf53e08376f1df846924c5e7974275696c6465724e657420284e65746865726d696e6429a0513431c984f68e5df00a61962758a00c532286eab77576c020a47a6143339a34880000000000000000840de3e760a0e79468ae242f77ca1f646b3c44f47b8e0c9f7d81b0595d050da422131ef3a13383120000830a0000a0aed13d7db24d28699822ee17acee009b2de309d9d8a622183d5616854b7d37b3a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    }

    // Expected values from Ethereum mainnet block
    bytes32 constant ETH_STATE_ROOT =
        0x3350c8652844a5728f2a73d19c970d5f0a9f6690935eea22d455fb03f3efb277;
    uint256 constant ETH_TIMESTAMP = 0x6924c5e7; // 1764017639

    // Real Base mainnet block header (block 0x24d3485 = 38663301)
    // Block hash: 0xd847c6579aa4c0527e7f878c08a68a21e44f8a31203be45744cf61fb0b918419
    function getBaseMainnetHeader() internal pure returns (bytes memory) {
        return
        hex"f9026fa02121e17746ab5765552d7e7335f00a11baeb0920f50fc2da9fc77cbdbdee4f3da01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347944200000000000000000000000000000000000011a039d33a12c8dccd974deb1401fbb718250ee4b5249940975e0936f56ea88cd1a3a09bd48752db144d161effc27743848c2dc090bd72027bcea228c5bc3104362033a004248b8ab251f567686e099ff48587ba6590fec5b3a93db1b176474e61b8bb17b90100bf77e7f5fe417cded767ddfdba8b7f22ff2d867cd3dad6bd0facb0e34da03ff8ef3dcedff727fe283f273df7b7eabef32fd5fa459ffdf5ee2fffb7bf7dff7c927ebbff98d77de5e9b7d7e65f93de21765fff651fdfedaeffef2bfbdffd67d8ef7fed57f6bbdf85d0e9a3bdf97b9f7ff321555f4eeacdaed73ebedd57fcd99fcf9bf17bbef9cd7cacc12e16edab5defdbedebbfdb787f8df9f7efedd75ee609f89b5f5696d3fc3fd737dfbde61db69bf97ff58e9f6ff6f9ff2bffa3ff5f51e58dfe13fbfe73dbfcf37ff5df949e6775a60dfedcfff5ff0e7fecfcc9fbdfbbfbef7ffffa9b66fefd4e7dfbe74f337ffdf7fffeb5f477fd4fc55b4ccbcddaf96fba8084024d34858411e1a30084048e667e846924c5ed89000000003200000005a03c46536d7e016bf04291b8dbcff8b2606ad0ebb2a0466b6760d2d4e68d42642b880000000000000000830b8ec4a0ea08782b44671e0911fdff3e8574395cd5a008898a80ce1f8517b0a579480a048080a0eb1f0486677fa10656c168f404a09d738bdc7941acfb5f5c2b2f872111aeaa29a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    }

    // Expected values from Base mainnet block
    bytes32 constant BASE_STATE_ROOT =
        0x39d33a12c8dccd974deb1401fbb718250ee4b5249940975e0936f56ea88cd1a3;
    uint256 constant BASE_TIMESTAMP = 0x6924c5ed; // 1764017645

    function test_toEvmHeader_eth_parsesStateRoot() public view {
        EvmHeader memory header = wrapper.toEvmHeader(getEthMainnetHeader());
        assertEq(header.stateRoot, ETH_STATE_ROOT, "ETH state root mismatch");
    }

    function test_toEvmHeader_eth_parsesTimestamp() public view {
        EvmHeader memory header = wrapper.toEvmHeader(getEthMainnetHeader());
        assertEq(header.timestamp, ETH_TIMESTAMP, "ETH timestamp mismatch");
    }

    function test_toEvmHeader_base_parsesStateRoot() public view {
        EvmHeader memory header = wrapper.toEvmHeader(getBaseMainnetHeader());
        assertEq(header.stateRoot, BASE_STATE_ROOT, "Base state root mismatch");
    }

    function test_toEvmHeader_base_parsesTimestamp() public view {
        EvmHeader memory header = wrapper.toEvmHeader(getBaseMainnetHeader());
        assertEq(header.timestamp, BASE_TIMESTAMP, "Base timestamp mismatch");
    }

    function test_toEvmHeader_revertsOnShortHeader() public {
        // Header too short (less than 448 bytes)
        bytes memory shortHeader = new bytes(400);
        shortHeader[0] = 0xf9;

        vm.expectRevert(); // InvalidBlockHeaderEncoding()
        wrapper.toEvmHeader(shortHeader);
    }

    function test_toEvmHeader_revertsOnInvalidPrefix() public {
        // Header with wrong prefix (not 0xf9)
        bytes memory invalidPrefix = new bytes(500);
        invalidPrefix[0] = 0x00; // Invalid prefix

        vm.expectRevert(); // InvalidBlockHeaderEncoding()
        wrapper.toEvmHeader(invalidPrefix);
    }
}

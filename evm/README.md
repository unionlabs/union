# Contract addresses

All the deployed contracts are upgradeable proxies forwarding calls the the underlying implementation.

We use a a special contract called deployer in order to generate deterministic addresses that don't include the initcode in the derivation, see deploy https://github.com/Vectorized/solady/blob/e6ad61c844d6392910bdd21d39a33b3d668fc987/src/utils/CREATE3.sol#L63.

This deployer contract will be pre-deployed on all EVM networks where we deploy the IBC stack.

```solidity
import "solady/utils/CREATE3.sol";
import "solady/utils/LibString.sol";
contract Deployer {
    using LibString for *;
    function deploy(
        string memory salt,
        bytes calldata creationCode,
        uint256 value
    ) public returns (address) {
        return CREATE3.deploy(
            keccak256(abi.encodePacked(msg.sender.toHexString(), "/", salt)),
            creationCode,
            value
        );
    }
}
```

The following table maps salt to contracts:

| salt                    | contract       |
| ----------------------- | -------------- |
| "ibc-is-based"          | IBCHandler     |
| "lightclients/cometbls" | CometblsClient |
| "protocols/ucs01"       | UCS01          |
| "protocols/ucs02"       | UCS02          |
| "multicall"             | Multicall      |

The combination `(deployer_source, deployer_source_nonce, deployer, sender, salt)` fully determines the final addresses (no bytecode hash of any of the above contract involved).

## Devnet

This links are working if you run a local devnet on a x86 machine only (Blockscout is currently unsupported on arm64).

- IBCHandler: [0xed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5](http://localhost/address/0xed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5)
- CometblsClient: [0xc4f27a952faBa4174ce0Ee6D9d0c6F4c41524d49](http://localhost/address/0xc4f27a952faBa4174ce0Ee6D9d0c6F4c41524d49)
- UCS00: [0x21bd17aec8CEb789D3145a606968Dcc428c1e4F4](http://localhost/address/0x21bd17aec8CEb789D3145a606968Dcc428c1e4F4)
- UCS01: [0xa9d03ba6E27B43c69a64C87F845485b73A8e5d46](http://localhost/address/0xa9d03ba6E27B43c69a64C87F845485b73A8e5d46)
- UCS02: [0x524D4d28fc90dc5A257162abE37081f52681C7D6](http://localhost/address/0x524D4d28fc90dc5A257162abE37081f52681C7D6)
- Multicall: [0x9fd9D9528c8373D990a1380B9414bDE179007A35](http://localhost/address/0x9fd9D9528c8373D990a1380B9414bDE179007A35)

## Testnet 8

Note that the addresses are different because we often redeploy without upgrading when a storage breaking update is pushed.
Production contracts will get solely upgraded through the proxy and have the same addresses accross networks.

### Sepolia

- Deployer: [0x8473C8A647f7EEE80dE978251DAEfD6EFa63ED7C](https://sepolia.etherscan.io/address/0x8473C8A647f7EEE80dE978251DAEfD6EFa63ED7C)
- Sender: [0x153919669Edc8A5D0c8D1E4507c9CE60435A1177](https://sepolia.etherscan.io/address/0x153919669Edc8A5D0c8D1E4507c9CE60435A1177)
- IBCHandler: [0xE3e97964d842623D2Da087978Fbe343523D810B2](https://sepolia.etherscan.io/address/0xE3e97964d842623D2Da087978Fbe343523D810B2)
- CometblsClient: [0xbFFc83e32Bd9428f00bE562e5608DB945c7fF475](https://sepolia.etherscan.io/address/0xbFFc83e32Bd9428f00bE562e5608DB945c7fF475)
- UCS00: [0xf81158aC87f2cdc8543E40aBD8Fe8Ad1efe50e5e](https://sepolia.etherscan.io/address/0xf81158aC87f2cdc8543E40aBD8Fe8Ad1efe50e5e)
- UCS01: [0x3188A9D9a3AdfaFf95074dfBF14ed3ab6C3C0B6E](https://sepolia.etherscan.io/address/0x3188A9D9a3AdfaFf95074dfBF14ed3ab6C3C0B6E)
- UCS02: [0xcd83BFe45F4bc25Db2A153D6Aba40B99a2D90C61](https://sepolia.etherscan.io/address/0xcd83BFe45F4bc25Db2A153D6Aba40B99a2D90C61)
- Multicall: [0xa4Cb22b21fD3701797218324A3425d50e12E66Cd](https://sepolia.etherscan.io/address/0xa4Cb22b21fD3701797218324A3425d50e12E66Cd)

### Holesky

- Deployer: [0x8473C8A647f7EEE80dE978251DAEfD6EFa63ED7C](https://holesky.etherscan.io/address/0x8473C8A647f7EEE80dE978251DAEfD6EFa63ED7C)
- Sender: [0x153919669Edc8A5D0c8D1E4507c9CE60435A1177](https://holesky.etherscan.io/address/0x153919669Edc8A5D0c8D1E4507c9CE60435A1177)
- IBCHandler: [0xE3e97964d842623D2Da087978Fbe343523D810B2](https://holesky.etherscan.io/address/0xE3e97964d842623D2Da087978Fbe343523D810B2)
- CometblsClient: [0xbFFc83e32Bd9428f00bE562e5608DB945c7fF475](https://holesky.etherscan.io/address/0xbFFc83e32Bd9428f00bE562e5608DB945c7fF475)
- UCS00: [0xf81158aC87f2cdc8543E40aBD8Fe8Ad1efe50e5e](https://holesky.etherscan.io/address/0xf81158aC87f2cdc8543E40aBD8Fe8Ad1efe50e5e)
- UCS01: [0x3188A9D9a3AdfaFf95074dfBF14ed3ab6C3C0B6E](https://holesky.etherscan.io/address/0x3188A9D9a3AdfaFf95074dfBF14ed3ab6C3C0B6E)
- UCS02: [0xcd83BFe45F4bc25Db2A153D6Aba40B99a2D90C61](https://holesky.etherscan.io/address/0xcd83BFe45F4bc25Db2A153D6Aba40B99a2D90C61)
- Multicall: [0xa4Cb22b21fD3701797218324A3425d50e12E66Cd](https://holesky.etherscan.io/address/0xa4Cb22b21fD3701797218324A3425d50e12E66Cd)

### Berachain

- Deployer: [0x17425b7d2d97E613dE2ADa01Dc472F76879E08Fe](https://bartio.beratrail.io/address/0x17425b7d2d97E613dE2ADa01Dc472F76879E08Fe)
- Sender: [0x27156Eb671984304ae75Da49aD60C4479B490A06](https://bartio.beratrail.io/address/0x27156Eb671984304ae75Da49aD60C4479B490A06)
- IBCHandler: [0x851c0EB711fe5C7c8fe6dD85d9A0254C8dd11aFD](https://bartio.beratrail.io/address/0x851c0EB711fe5C7c8fe6dD85d9A0254C8dd11aFD)
- CometblsClient: [0x702F0C9e4E0F5EB125866C6E2F57eC3751B4da1A](https://bartio.beratrail.io/address/0x702F0C9e4E0F5EB125866C6E2F57eC3751B4da1A)
- UCS01: [0x6F270608fB562133777AF0f71F6386ffc1737C30](https://bartio.beratrail.io/address/0x6F270608fB562133777AF0f71F6386ffc1737C30)
- UCS02: [0xD05751B3F4d8dCf8487DB33b57C523dD7DB11C25](https://bartio.beratrail.io/address/0xD05751B3F4d8dCf8487DB33b57C523dD7DB11C25)
- Multicall: [0x3147CA8f531070DDAC1b93700ef18E4Dd05b86ec](https://bartio.beratrail.io/address/0x3147CA8f531070DDAC1b93700ef18E4Dd05b86ec)

### Arbitrum

- Deployer: [0x7d00b15A53B8b14a482BF761653532F07b7DcBdE](https://sepolia.arbiscan.io/address/0x7d00b15A53B8b14a482BF761653532F07b7DcBdE)
- Sender: [0x50C9C35e0197e781e9aD7a3F6baDD8d01E45c377](https://sepolia.arbiscan.io/address/0x50C9C35e0197e781e9aD7a3F6baDD8d01E45c377)
- IBCHandler: [0xb599bfcfb9D4fCaE9f8aB5D45d9A6F145E6b7573](https://sepolia.arbiscan.io/address/0xb599bfcfb9D4fCaE9f8aB5D45d9A6F145E6b7573)
- CometblsClient: [0x2c84Dd2515e906a04C57c8604535CEAd6B2F5F73](https://sepolia.arbiscan.io/address/0x2c84Dd2515e906a04C57c8604535CEAd6B2F5F73)
- UCS01: [0xBd346331b31f8C43CC378286Bfe49f2f7F128c39](https://sepolia.arbiscan.io/address/0xBd346331b31f8C43CC378286Bfe49f2f7F128c39)
- UCS02: [0x4505EB10bc6E8DfB38C2AB65B3017fd0Ae223827](https://sepolia.arbiscan.io/address/0x4505EB10bc6E8DfB38C2AB65B3017fd0Ae223827)
- Multicall: [0xd867c233ee0908FC7BC21095dA47F775F6479F2A](https://sepolia.arbiscan.io/address/0xd867c233ee0908FC7BC21095dA47F775F6479F2A)

### Scroll

[Commit: 5530a7d3f88e602b9a8da445f54826699768c3dd](https://github.com/unionlabs/union/commit/5530a7d3f88e602b9a8da445f54826699768c3dd)

- Deployer: [0x8E6cbf264706486E533eA07399474d9e1616313d](https://sepolia.scrollscan.com/address/0x8E6cbf264706486E533eA07399474d9e1616313d)
- Sender: [0x6BFD43FE5cb241b360EC4a307700c6a42EE9F6cb](https://sepolia.scrollscan.com/address/0x6BFD43FE5cb241b360EC4a307700c6a42EE9F6cb)
- IBCHandler: [0x03792798d62F082a2748e686745a5Cd7Ab06Ee6D](https://sepolia.scrollscan.com/address/0x03792798d62F082a2748e686745a5Cd7Ab06Ee6D)
- CometblsClient: [0xf303226A9FF0a920a79BcC2d9012871735C0f611](https://sepolia.scrollscan.com/address/0xf303226A9FF0a920a79BcC2d9012871735C0f611)
- UCS01: [0xA61Bdce84F44CA842D0EE9c1706A3C9fDD311DC2](https://sepolia.scrollscan.com/address/0xA61Bdce84F44CA842D0EE9c1706A3C9fDD311DC2)
- UCS02: [0x8eEC1B331B46cDb1021718cf2422100eadD1e13e](https://sepolia.scrollscan.com/address/0x8eEC1B331B46cDb1021718cf2422100eadD1e13e)
- Multicall: [0x58FC3fB2d19A41bdbD5B5f4B12cf9C69172601C7](https://sepolia.scrollscan.com/address/0x58FC3fB2d19A41bdbD5B5f4B12cf9C69172601C7)

## Other networks

Assuming you create the deployer from a fresh account `<SOURCE>` (0 nonce), the `<DEPLOYER>` address can be precomputed with `cast compute-address --nonce 0 <SOURCE>`

Given the `<DEPLOYER>` contract and a `<SENDER>`, you can compute the IBC stack addresses using:

`nix run .\#evm-contracts-addresses -- <DEPLOYER> <SENDER>`

Example result using the devnet private key `0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77`:

```sh
~/github/union (main*) » cast compute-address --nonce 0 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD

Computed Address: 0x86D9aC0Bab011917f57B9E9607833b4340F9D4F8
```

```sh
~/github/union (main*) » nix run .\#evm-contracts-addresses -- 0x86D9aC0Bab011917f57B9E9607833b4340F9D4F8 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD

Script ran successfully.
Gas used: 52087

== Logs ==
  IBCHandler: 0xed2af2ad7fe0d92011b26a2e5d1b4dc7d12a47c5
  CometblsClient: 0xc4f27a952faba4174ce0ee6d9d0c6f4c41524d49
  UCS01: 0xa9d03ba6e27b43c69a64c87f845485b73a8e5d46
  UCS02: 0x524d4d28fc90dc5a257162abe37081f52681c7d6
```

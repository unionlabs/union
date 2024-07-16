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

- IBCHandler: [0xed2af2ad7fe0d92011b26a2e5d1b4dc7d12a47c5](http://localhost/address/0x524D4d28fc90dc5A257162abE37081f52681C7D6)
- CometblsClient: [0xc4f27a952faba4174ce0ee6d9d0c6f4c41524d49](http://localhost/address/0xc4f27a952faba4174ce0ee6d9d0c6f4c41524d49)
- UCS01: [0xa9d03ba6e27b43c69a64c87f845485b73a8e5d46](http://localhost/address/0xa9d03ba6e27b43c69a64c87f845485b73a8e5d46)
- UCS02: [0x524d4d28fc90dc5a257162abe37081f52681c7d6](http://localhost/address/0x524d4d28fc90dc5a257162abe37081f52681c7d6)
- Multicall: [0x9fd9D9528c8373D990a1380B9414bDE179007A35](http://localhost/address/0x9fd9D9528c8373D990a1380B9414bDE179007A35?tab=contract)

## Testnet 8

Note that the addresses are different because we often redeploy without upgrading when a storage breaking update is pushed.
Production contracts will get solely upgraded through the proxy.

### Sepolia
- Deployer: [0x12cffF5aAd6Fc340BBE6F1fe674C5Aa78f0d1E0F](https://sepolia.etherscan.io/address/0x12cffF5aAd6Fc340BBE6F1fe674C5Aa78f0d1E0F)
- Sender: [0x2c077908e1173ff1a6097ca9e2af547c1e5130c4](https://sepolia.etherscan.io/address/0x2c077908e1173ff1a6097ca9e2af547c1e5130c4)
- IBCHandler: [0xa390514f803a3b318b93bf6cd4beeb9f8299a0eb](https://sepolia.etherscan.io/address/0xa390514f803a3b318b93bf6cd4beeb9f8299a0eb)
- CometblsClient: [0x96979ed96ae00d724109b5ad859568e1239c0837](https://sepolia.etherscan.io/address/0x96979ed96ae00d724109b5ad859568e1239c0837)
- UCS01: [0xd0081080ae8493cf7340458eaf4412030df5feeb](https://sepolia.etherscan.io/address/0xd0081080ae8493cf7340458eaf4412030df5feeb)
- UCS02: [0x9153952f174a1bcd7a9b3818ff21ecf918d4dca9](https://sepolia.etherscan.io/address/0x9153952f174a1bcd7a9b3818ff21ecf918d4dca9)
- Multicall: [0x70BEDecc56C7104e410c1e4c25FcA0bcd29A0bA9](https://sepolia.etherscan.io/address/0x70bedecc56c7104e410c1e4c25fca0bcd29a0ba9)

### Berachain
- Deployer: [0x17425b7d2d97E613dE2ADa01Dc472F76879E08Fe](https://bartio.beratrail.io/address/0x1d95c54b436A73A43bC879E1C6A4b282fB88E784)
- Sender: [0x27156Eb671984304ae75Da49aD60C4479B490A06](https://bartio.beratrail.io/address/0x2c077908e1173ff1A6097ca9E2aF547c1E5130C4)
- IBCHandler: [0x851c0EB711fe5C7c8fe6dD85d9A0254C8dd11aFD](https://bartio.beratrail.io/address/0x663955E2eebf9b398CD9D32cA72359fdA17F2482)
- CometblsClient: [0x702F0C9e4E0F5EB125866C6E2F57eC3751B4da1A](https://bartio.beratrail.io/address/0xf9C6120Ef88E27937304622ccfc8Dcdb92C88b56)
- UCS01: [0x6F270608fB562133777AF0f71F6386ffc1737C30](https://bartio.beratrail.io/address/0x7f8e0965ffdAFa1c5621B14C51bDc631165fc788)
- UCS02: [0xD05751B3F4d8dCf8487DB33b57C523dD7DB11C25](https://bartio.beratrail.io/address/0x749937900103DF548625C47D3de5d928b61b8409)
- Multicall: [0x3147CA8f531070DDAC1b93700ef18E4Dd05b86ec](https://bartio.beratrail.io/address/0x248717213d7f79Ac37Fb0C01bA4f370be5c2Ab18)

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

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

The combination `(deployer_source, deployer_source_nonce, deployer, sender, salt)` fully determines the final addresses (no bytecode hash of any of the above contract involved).

## Devnet

This links are working if you run a local devnet on a x86 machine only (Blockscout is currently unsupported on arm64).

- IBCHandler: [0xed2af2ad7fe0d92011b26a2e5d1b4dc7d12a47c5](http://localhost/address/0x524D4d28fc90dc5A257162abE37081f52681C7D6)
- CometblsClient: [0xc4f27a952faba4174ce0ee6d9d0c6f4c41524d49](http://localhost/address/0xc4f27a952faba4174ce0ee6d9d0c6f4c41524d49)
- UCS01: [0xa9d03ba6e27b43c69a64c87f845485b73a8e5d46](http://localhost/address/0xa9d03ba6e27b43c69a64c87f845485b73a8e5d46)
- UCS02: [0x524d4d28fc90dc5a257162abe37081f52681c7d6](http://localhost/address/0x524d4d28fc90dc5a257162abe37081f52681c7d6)

## Testnet 8

Note that the addresses are different because we often redeploy without upgrading when a storage breaking update is pushed.
Production contracts will get solely upgraded through the proxy.

### Sepolia
- IBCHandler: [0xa390514f803a3b318b93bf6cd4beeb9f8299a0eb](https://sepolia.etherscan.io/address/0xa390514f803a3b318b93bf6cd4beeb9f8299a0eb)
- CometblsClient: [0x96979ed96ae00d724109b5ad859568e1239c0837](https://sepolia.etherscan.io/address/0x96979ed96ae00d724109b5ad859568e1239c0837)
- UCS01: [0xd0081080ae8493cf7340458eaf4412030df5feeb](https://sepolia.etherscan.io/address/0xd0081080ae8493cf7340458eaf4412030df5feeb)
- UCS02: [0x9153952f174a1bcd7a9b3818ff21ecf918d4dca9](https://sepolia.etherscan.io/address/0x9153952f174a1bcd7a9b3818ff21ecf918d4dca9)

### Berachain
- IBCHandler: [0x4e86d3eb0f4d8ddccec2b8fa5ccfc8170e8ac3dc](https://bartio.beratrail.io/address/0x4e86d3eb0f4d8ddccec2b8fa5ccfc8170e8ac3dc)
- CometblsClient: [0x3b089e62ed1f9257f7c66e79dde1463f063d6a35](https://bartio.beratrail.io/address/0x3b089e62ed1f9257f7c66e79dde1463f063d6a35)
- UCS01: [0x0e7aee8a4109b1c1916281d25f43b937f103a409](https://bartio.beratrail.io//address/0x0e7aee8a4109b1c1916281d25f43b937f103a409)
- UCS02: [0x275ff682294a96b88de06d5ced0bfaf4724ff2d8](https://bartio.beratrail.io/address/0x275ff682294a96b88de06d5ced0bfaf4724ff2d8)

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

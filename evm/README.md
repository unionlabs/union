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

### V1 (deprecated)
- IBCHandler: [0x6b6b60a68b8dcbb170f25045974d10098917f816](https://sepolia.etherscan.io/address/0x6b6b60a68b8dcbb170f25045974d10098917f816)
- CometblsClient: [0xf906a05a25bf5b61a5e4ff24be9122e2cea5f1e3](https://sepolia.etherscan.io/address/0xf906a05a25bf5b61a5e4ff24be9122e2cea5f1e3)
- UCS01: [0x3d0eb16ad2619666dbde1921282cd885b58eeefe](https://sepolia.etherscan.io/address/0x3d0eb16ad2619666dbde1921282cd885b58eeefe)
- UCS02: [0xb455b205106c9b72e967399e15efd8a025fd4a90](https://sepolia.etherscan.io/address/0xb455b205106c9b72e967399e15efd8a025fd4a90)

### V2 (live)
- IBCHandler: [0x2881e1c5863e358c9f56b64ec4583ab79e450116](https://sepolia.etherscan.io/address/0x2881e1c5863e358c9f56b64ec4583ab79e450116)
- CometblsClient: [0x2d5c9af388059658595f5c761d14a5b0300cb6e9](https://sepolia.etherscan.io/address/0x2d5c9af388059658595f5c761d14a5b0300cb6e9)
- UCS01: [0xddd103e667e10fa4037d0eaea1531a0211d3a67c](https://sepolia.etherscan.io/address/0xddd103e667e10fa4037d0eaea1531a0211d3a67c)
- UCS02: [0xcf43211ebb12fe3e25cde58f34d83f2ebf6e5690](https://sepolia.etherscan.io/address/0xcf43211ebb12fe3e25cde58f34d83f2ebf6e5690)

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

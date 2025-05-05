## Deployments

Deployment addresses have been moved [here](../docs/src/content/docs/protocol/deployments.mdx)

## Devnet explorer addresses

### Devnet

:::caution

These links are only valid if you run a local ethereum devnet on an x86 machine. (Blockscout is currently unsupported on arm64).

:::

| Category           | Name              | Address                                                                                                             |
|--------------------|-------------------|---------------------------------------------------------------------------------------------------------------------|
| core               | IBCHandler        | [`0xed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5`](http://localhost/address/0xed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5) |
| light-clients      | CometblsClient    | [`0xc4f27a952faBa4174ce0Ee6D9d0c6F4c41524d49`](http://localhost/address/0xc4f27a952faBa4174ce0Ee6D9d0c6F4c41524d49) |
| apps               | UCS00             | [`0x21bd17aec8CEb789D3145a606968Dcc428c1e4F4`](http://localhost/address/0x21bd17aec8CEb789D3145a606968Dcc428c1e4F4) |
|                    | UCS01             | [`0xa9d03ba6E27B43c69a64C87F845485b73A8e5d46`](http://localhost/address/0xa9d03ba6E27B43c69a64C87F845485b73A8e5d46) |
|                    | UCS02             | [`0x524D4d28fc90dc5A257162abE37081f52681C7D6`](http://localhost/address/0x524D4d28fc90dc5A257162abE37081f52681C7D6) |
|  support           | Multicall         | [`0x9fd9D9528c8373D990a1380B9414bDE179007A35`](http://localhost/address/0x9fd9D9528c8373D990a1380B9414bDE179007A35) |

## Deployment Process

> \[!NOTE\]
> The addresses are different because we often redeploy without upgrading when a storage breaking update is pushed.
> Production contracts will get solely upgraded through the proxy and have the same addresses across networks.

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

### Computing deployment addresses on other networks

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

## Upgrading contracts

To see all generated contract upgrade scripts

```bash
nix run .#eth-upgrade- <TAB>
```

To execute an upgrade (example values)

```bash
nix run .\#eth-upgrade-holesky-ucs03 -- --deployer_pk 0xa3cd41bff71ad19fddfd901a9773c975a0404d97 --sender_pk 0x153919669Edc8A5D0c8D1E4507c9CE60435A1177 --private_key omitted
```

To re-verify contracts after the upgrade:

```bash
nix run .\#eth-verify-holesky DEPLOYER_ADDR SENDER_ADDR ETHERSCAN_API_KEY
```

For example:

```bash
nix run .\#eth-verify-holesky 0xa3cd41bff71ad19fddfd901a9773c975a0404d97 0x153919669Edc8A5D0c8D1E4507c9CE60435A1177 omitted
```

## Verifying contracts

By default, we verify all contracts on [tenderly](https://tenderly.co/) (for chains that are supported). Additional verifications can be done by threading through the `FOUNDRY_ETHERSCAN` and `VERIFIER` environment variables.

TODO: Explain more here

## Other supported explorers

Other explorers that we verify on. This list is non-exhaustive.

### `ethereum.1`

<https://etherscan.io/>

```sh
# replace $KEY with your etherscan api key
FOUNDRY_ETHERSCAN='{ chain = { key = "$KEY", chain = "1", url = "https://api.etherscan.io/api" } }' nix run .#evm-scripts.ethereum
```

<https://eth.blockscout.com/>

```sh
# key is empty for blockscout, but still required by the foundry config schema
VERIFIER=blockscout FOUNDRY_ETHERSCAN='{ chain = { key = "", chain = "1", url = "https://eth.blockscout.com/api" } }' nix run .#evm-scripts.ethereum
```

<https://1.routescan.io/>

```sh
FOUNDRY_ETHERSCAN='{ chain = { key = "verifyContract", chain = "1", url = "https://api.routescan.io/v2/network/mainnet/evm/1/etherscan" } }' nix run .#evm-scripts.ethereum
```

### `bob.60808`

<https://explorer.gobob.xyz/>

```sh
VERIFIER=blockscout FOUNDRY_ETHERSCAN='{ chain = { key = "", chain = "60808", url = "https://explorer.gobob.xyz/api" } }' nix run .#evm-scripts.bob
```

### `bob.808813`

<https://bob-sepolia.explorer.gobob.xyz/>

```sh
VERIFIER=blockscout FOUNDRY_ETHERSCAN='{ chain = { key = "", chain = "808813", url = "https://bob-sepolia.explorer.gobob.xyz/api" } }' nix run .#evm-scripts.bob-sepolia
```

### `corn.21000000`

<https://cornscan.io/>

```sh
FOUNDRY_ETHERSCAN='{ chain = { key = "verifyContract", chain = "21000000", url = "https://api.routescan.io/v2/network/mainnet/evm/21000000/etherscan" } }' nix run .#evm-scripts.corn
```

### `corn.21000001`

<https://testnet.cornscan.io/>

```sh
FOUNDRY_ETHERSCAN='{ chain = { key = "verifyContract", chain = "21000001", url = "https://api.routescan.io/v2/network/testnet/evm/21000001/etherscan" } }' nix run .#evm-scripts.corn-testnet
```

### `sei.1328`

<https://seitrace.com/?chain=atlantic-2>

```sh
# key is an arbitrary non-empty string
FOUNDRY_ETHERSCAN='{ chain = { key = "asdf", chain = "1328", url = "https://seitrace.com/atlantic-2/api" } }' nix run .#evm-scripts.sei-atlantic
```

### `berachain.80069`

<https://bepolia.beratrail.io/>

```sh
FOUNDRY_ETHERSCAN='{ chain = { key = "verifyContract", chain = "80069", url = "https://api.routescan.io/v2/network/testnet/evm/80069/etherscan" } }' nix run .#evm-scripts.bepolia
```

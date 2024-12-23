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

## Testnet 9

Note that the addresses are different because we often redeploy without upgrading when a storage breaking update is pushed.
Production contracts will get solely upgraded through the proxy and have the same addresses accross networks.

### Holesky

- Deployer: [0xa3cd41bfF71AD19fDDfd901A9773C975A0404D97](https://eth-holesky.blockscout.com/address/0xa3cd41bfF71AD19fDDfd901A9773C975A0404D97)
- Sender: [0x153919669Edc8A5D0c8D1E4507c9CE60435A1177](https://eth-holesky.blockscout.com/address/0x153919669Edc8A5D0c8D1E4507c9CE60435A1177)
- IBCHandler: [0xfa4E502A3bf5f4Bc3EF0e17960b3Cd868d70E809](https://eth-holesky.blockscout.com/address/0xfa4E502A3bf5f4Bc3EF0e17960b3Cd868d70E809)
- CometblsClient: [0x6431abdc60313ec8780F5CCE1535dFe2DD891081](https://eth-holesky.blockscout.com/address/0x6431abdc60313ec8780F5CCE1535dFe2DD891081)
- UCS00: [0x92735254407859361265B51cDb76583ED7E3359b](https://eth-holesky.blockscout.com/address/0x92735254407859361265B51cDb76583ED7E3359b)
- UCS01: [0xdF48f737cc7eE649FC119B312932a9b99C40f417](https://eth-holesky.blockscout.com/address/0xdF48f737cc7eE649FC119B312932a9b99C40f417)
- UCS02: [0x8c5BB6EE0C679D605Fda89341148b9921C0d119c](https://eth-holesky.blockscout.com/address/address/0x8c5BB6EE0C679D605Fda89341148b9921C0d119c)
- Multicall: [0x64A764A734648fA636525C7e4b3cE38Ca256b647](https://eth-holesky.blockscout.com/address/address/0x64A764A734648fA636525C7e4b3cE38Ca256b647)

### Sepolia

- Deployer: [0xac6dBD360ABCfe0578e998D359d4F43a5A117219](https://eth-sepolia.blockscout.com/address/0xac6dBD360ABCfe0578e998D359d4F43a5A117219)
- Sender: [0x153919669Edc8A5D0c8D1E4507c9CE60435A1177](https://eth-sepolia.blockscout.com/address/0x153919669Edc8A5D0c8D1E4507c9CE60435A1177)
- IBCHandler: [0xbad69711Da45A0FF61e2c50b8c9B1F3314742d2b](https://eth-sepolia.blockscout.com/address/0xbad69711Da45A0FF61e2c50b8c9B1F3314742d2b)
- CometblsClient: [0x0A343260a06576a1f938C18F70FaA7eF2a3a7d4F](https://eth-sepolia.blockscout.com/address/0x0A343260a06576a1f938C18F70FaA7eF2a3a7d4F)
- UCS00: [0x271126f4F9B36CE16d9e2eF75691485ddCE11dB6](https://eth-sepolia.blockscout.com/address/0x271126f4F9B36CE16d9e2eF75691485ddCE11dB6)
- UCS01: [0xCFb741465F8e0AE9C62A548Fa85D312E6E5615Ba](https://eth-sepolia.blockscout.com/address/0xCFb741465F8e0AE9C62A548Fa85D312E6E5615Ba)
- UCS02: [0x12650fCccE6dB9E99CEE482490A5fAF248A62B22](https://eth-sepolia.blockscout.com/address/0x12650fCccE6dB9E99CEE482490A5fAF248A62B22)
- Multicall: [0x6FD4bf9438fAC8C535218E79191594A879E47E96](https://eth-sepolia.blockscout.com/address/0x6FD4bf9438fAC8C535218E79191594A879E47E96)

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

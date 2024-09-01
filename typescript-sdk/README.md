> [!NOTE] Work in progress

<h1 align="center" style="font-size: 2.75rem; font-weight: 900; color: white;">Union Labs TypeScript SDK</h1>

[![JSR](https://jsr.io/badges/@union/client)](https://jsr.io/@union/client)

Union Labs TypeScript SDK providing utilities for cross-chain transfers and more.

```sh
npx jsr add @union/client
```

## Usage

### Initiate a client

```ts
import { createUnionClient } from "@union/client"
import { privateKeyToAccount } from "viem/accounts"

const client = createUnionClient({
  chainId: "11155111",
  transport: http("https://rpc.sepolia.org"),
  account: privateKeyToAccount(`0x${PRIVATE_KEY}`) // or from wagmi configuration
})
```

### Examples

Transfer `UNO` from Union Testnet (`union-testnet-8`) chain to Sepolia on EVM (`1111551111`) chain.

```ts
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createUnionClient, hexStringToUint8Array } from "@union/client"

const PRIVATE_KEY = process.env.PRIVATE_KEY

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

const client = createUnionClient({
  account: cosmosAccount,
  chainId: "union-testnet-8",
  transport: http("https://rpc.testnet-8.union.build"),
})

const transfer = await client.transferAsset({
  amount: 1n,
  denomAddress: "muno",
  destinationChainId: "11155111",
  recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
})

if (transfer.isOk()) {
  return console.info("Transfer successful", transfer.value)
}

return console.error("Transfer failed", transfer.error)
```

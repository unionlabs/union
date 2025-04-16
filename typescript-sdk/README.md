# DEPRECATED: Union Labs TypeScript SDK

_See `../ts-sdk` for our sdk rewrite._

![JSR](https://jsr.io/badges/@union/client) ![NPM Version](https://img.shields.io/npm/v/@unionlabs/client)

Union Labs TypeScript SDK providing utilities for cross-chain transfers and more.

```sh
npm install @unionlabs/client
```

## Development

### Fetch latest schema

```sh
nix run .#ts-sdk-fetch-schema -L
```

### Publishing

Publish to npm registry:

```sh
npm run build # important!
npm publish --access='public' --no-git-tags
```

Publish to JSR:

```sh
bun ./scripts/publish.ts
```

## Usage

### Initiate a client

```ts
import { privateKeyToAccount } from "viem/accounts"
import { createUnionClient, http } from "@unionlabs/client"

const client = createUnionClient({
  chainId: "80084",
  transport: http("https://bartio.rpc.berachain.com"),
  account: privateKeyToAccount(`0x${process.env.PRIVATE_KEY}`),
})
```

### Examples

Transfer `strd` from Stride Testnet on Cosmos (`stride-internal-1`) chain to Sepolia on EVM (`1111551111`) chain.

```ts
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createUnionClient, hexToBytes, http } from "@unionlabs/client"

const PRIVATE_KEY = process.env["PRIVATE_KEY"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexToBytes(PRIVATE_KEY)),
  "stride"
)

const client = createUnionClient({
  account: cosmosAccount,
  chainId: "stride-internal-1",
  transport: http("stride.testnet-1.stridenet.co")
})

const transfer = await client.transferAsset({
  amount: 1n,
  autoApprove: true,
  denomAddress: "strd",
  destinationChainId: "11155111",
  receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd"
})

if (transfer.isErr()) {
  console.error(transfer.error)
  process.exit(1)
}

console.info(transfer.value)
```

## How to patch dependencies

```sh
npm install
npm install --package-lock-only
# edit node_modules/foo
./node_modules/patch-package/index.js foo
# a patch will be generated for foo in the patches/ dir
```

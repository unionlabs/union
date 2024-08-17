> [!NOTE] Work in progress

<br />

<p align="center">
  <img width="675" src="https://i.imgur.com/0h6fScq_d.webp?maxwidth=760&fidelity=grand" alt="Union logo" />
</p>
<br />
<p align="center">
  <!-- <a href="https://npmjs.com/package/@unionlabs/client"><img src="https://img.shields.io/npm/v/@unionlabs/client.svg" alt="npm package"></a> -->
</p>

<h1 align="center" style="font-size: 2.75rem; font-weight: 900; color: white;">Union Labs TypeScript SDK</h1>

Union Labs TypeScript SDK providing utilities for cross-chain transfers and more.

```sh
npx jsr add @union/client
```

### Quick Start

```ts
import { http } from "viem"
import { sepolia } from "viem/chains"
import { createCosmosSdkClient, cosmosHttp } from "@union/client"


const unionClient = createCosmosSdkClient({
  evm: {
    chain: sepolia,
    // browser wallet or `privateKeyToAccount` from `viem/accounts`
    account: evmAccount,
    transport: http("https://rpc2.sepolia.org")
  },
  cosmos: {
    // browser wallet or `DirectSecp256k1Wallet.fromKey`
    account: cosmosAccount,
    gasPrice: { amount: "0.0025", denom: "muno" },
    transport: cosmosHttp("https://rpc.testnet.bonlulu.uno")
  }
})

const gasCostResponse = await unionClient.simulateTransaction({
  amount: 1n,
  network: "evm",
  sourceChannel: "channel-69",
  path: ["11155111", "union-testnet-8"],
  recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
  denomAddress: "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
  relayContractAddress: "0x3C148Ec863404e48d88757E88e456963A14238ef"
})

if(!gasCostResponse.success) {
  throw new Error("Failed to simulate transaction")
}

console.info("Gas cost", gasCostResponse.data)

const transfer = await unionClient.transferAsset({
  amount: 1n,
  network: "evm",
  sourceChannel: "channel-69",
  path: ["11155111", "union-testnet-8"],
  recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
  denomAddress: "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
  relayContractAddress: "0x3C148Ec863404e48d88757E88e456963A14238ef"
})
```

See [`./playground/berachain-to-union.ts`](./playground/berachain-to-union.ts) and [`./playground`](./playground) in general for more examples.

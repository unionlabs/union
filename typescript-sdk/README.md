> [!NOTE] Work in progress

<br />

<p align="center">
  <img width="175" src="https://union.build/logo.svg" alt="Union logo" />
</p>
<br />
<p align="center">
  <!-- <a href="https://npmjs.com/package/@unionlabs/client"><img src="https://img.shields.io/npm/v/@unionlabs/client.svg" alt="npm package"></a> -->
</p>

<h1 align="center" style="font-size: 2.75rem; font-weight: 900; color: white;">Union Labs TypeScript SDK</h1>

Union Labs TypeScript SDK providing utilities for cross-chain transfers and more.

```sh
yarn add @unionlabs/client
```

### Patched dependencies:

- `@cosmjs/tendermint-rpc`, `@cosmjs/amino`, and `@cosmjs/stargate`. See [./patches](./patches) for details.

```ts
import { unionActions, chain, UCS01_EVM_ADDRESS } from '@unionlabs/client'

import { privateKeyToAccount } from 'viem/accounts'
import { http, publicActions, createWalletClient } from 'viem'
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing'

const demoEthereumAccount = privateKeyToAccount(
  '0x1075394148aee9ccae14500c37cfdfca7bea4a4984fd5882a9ecf1be610d84ee'
)
const demoEthereumAddress = demoEthereumAccount.address

const demoUnionAccount = await DirectSecp256k1HdWallet.fromMnemonic(
  'enlist hip relief stomach skate base shallow young switch frequent cry park',
  { prefix: 'union' }
)

const [demoUnionAccountData] = await demoUnionAccount.getAccounts()
if (!demoUnionAccountData?.address) throw new Error('demoUnionAccountData is undefined')
const demoUnionAddress = demoUnionAccountData.address

const { sepolia } = chain.ethereum
const { testnet: unionTestnet } = chain.union

export const client = createWalletClient({
  chain: sepolia,
  account: demoEthereumAccount,
  // transport can be an RPC URL or `custom(window.ethereum)` for browser
  transport: http(process.env.SEPOLIA_RPC_URL),
})
  .extend(publicActions)
  .extend(unionActions)

// approve the spending of a handful of uno. Returns transaction hash
const approvalHash = await client.approveAsset({
  chainId: '11155111',
  signer: demoEthereumAccount,
  amount: 500n,
  spender: UCS01_EVM_ADDRESS,
  assetId: sepolia.token.address,
})

console.log(JSON.stringify({ approvalHash }, undefined, 2))

// Send 1 muno from Sepolia to Union chain. Returns transaction hash
const ethereumToUnionTransfer = await client.sendAsset({
  chainId: '11155111',
  signer: demoEthereumAccount,
  assetId: sepolia.token.address,
  amount: 1n,
  receiver: demoUnionAddress,
})

console.log(JSON.stringify({ ethereumToUnionTransfer }, undefined, 2))

// Send 500 muno from Union chain to Sepolia. Returns transaction hash
const unionToEthereumTransfer = await client.sendAsset({
  chainId: '6',
  signer: demoUnionAccount,
  assetId: unionTestnet.token.address,
  amount: '100',
  denom: 'muno',
  receiver: demoEthereumAddress,
  gasPrice: '0.001muno',
})

console.log(JSON.stringify({ unionToEthereumTransfer }, undefined, 2))
```

Two balance retrieval actions are available:

```ts
// Returns balance of muno on Ethereum
const balanceOnEthereum = await client.getBalance({
  chainId: '11155111',
  address: demoEthereumAddress,
  assetId: sepolia.token.address,
})

console.log(JSON.stringify({ balanceOnEthereum }, undefined, 2))

// Returns balance of muno on Union chain as string
const balanceOnUnion = await client.getBalance({
  chainId: '6',
  address: demoUnionAddress,
  assetId: unionTestnet.token.denom,
})

console.log(JSON.stringify({ balanceOnUnion }, undefined, 2))
```

## Development

### Prerequisites

- [Node.js](https://nodejs.org/en/) LTS (v20.x)
- Environment Variables

  Copy `.env.example` to `.env` and fill in the values.

  ```sh
  NODE_ENV="development"
  # browser or node. Use node for testing locally (e.g., against anvil account)
  CLIENT_MODE="browser"
  # optional for local testing
  ANVIL_ACCOUNT_PRIVATE_KEY=""
  # https://docs.union.build/joining_the_testnet/public_endpoints#rpc
  UNION_RPC_URL=""
  UNION_GRAPHQL_API=""
  UNION_CHAIN_ID=""
  MUNO_ERC20_ADDRESS=""
  UCS01_EVM_ADDRESS=""
  UCS01_UNION_SOURCE_CHANNEL=""
  UCS01_SEPOLIA_SOURCE_CHANNEL=""
  UCS01_SEPOLIA_PORT_ID=""
  UCS01_UNION_ADDRESS=""
  ```

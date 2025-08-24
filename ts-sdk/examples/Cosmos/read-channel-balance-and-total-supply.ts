/**
 * @title Read Balance & Total Supply
 * @badge WIP:caution
 */
/// <reference types="effect" />
/// <reference types="@cosmjs/stargate" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
// ---cut---
import { Cosmos } from "@unionlabs/sdk"
import { Effect } from "effect"
// import { createPublicClient, http } from "viem"
// import { mainnet } from "viem/chains"

const xionClient = Cosmos.Client.Live("https://rpc.xion-testnet-2.xion.chain.kitchen/")

const xionBalance = Effect.gen(function*() {
  const address = "xion122ny3mep2l7nhtafpwav2y9e5jrslhekkyv629"
  const denom = "ibc/6490A7EAB61059BFC1CDDEB05917DD70BDF3A611654162A1A47DB930D40D8AF4" // USDC on XION

  return yield* Cosmos.getBalanceNow(address, denom)
}).pipe(
  Effect.provide(xionClient),
)

Effect.runPromise(Effect.all({
  xionBalance,
}))
  .then(console.log)
  .catch(console.error)

// Read Ethereum Channel Balance
// Effect.runPromiseExit(
//   Effect.gen(function*() {
//     // Create a public client for Sepolia
//     const client = createPublicClient({
//       chain: mainnet,
//       transport: http("https://rpc.1.ethereum.chain.kitchen"),
//     })

//     const tokenAddress = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
//     const ucs03Address = "0x5fbe74a283f7954f10aa04c2edf55578811aeb03"
//     const source_channel_id = 1
//     const path = 0n

//     const balance = yield* EthereumChannelBalance(path, tokenAddress).pipe(
//       Effect.provideService(ViemPublicClientDestination, { client }),
//       Effect.provideService(EvmChannelDestination, {
//         ucs03address: ucs03Address,
//         channelId: source_channel_id,
//       }),
//     )

//     yield* Effect.log(`Channel Balance of token address: ${tokenAddress} on Ethereum: ${balance}`)
//   }),
// ).then(exit => console.log(JSON.stringify(exit, null, 2)))

// Read Ethereum TotalSupply
// Effect.runPromiseExit(
//   Effect.gen(function*() {
//     // Create a public client for Sepolia
//     const client = createPublicClient({
//       chain: mainnet,
//       transport: http("https://rpc.1.ethereum.chain.kitchen"),
//     })

//     // USDC on Sepolia
//     const tokenAddress = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"

//     // Read ERC20 metadata
//     const totalSupply = yield* readErc20TotalSupply(tokenAddress).pipe(
//       Effect.provideService(ViemPublicClient, { client }),
//     )

//     yield* Effect.log(`Total Supply of token: ${tokenAddress} on Ethereum: ${totalSupply}`)
//   }),
// ).then(exit => console.log(JSON.stringify(exit, null, 2)))

// Read Babylon TotalSupply
// Effect.runPromiseExit(
//   Effect.gen(function*() {
//     // Create a CosmWasm client
//     const client = yield* createCosmWasmClient("https://rpc.bbn-1.babylon.chain.kitchen")

//     const tokenAddress = "bbn1300se0vwue77hn6s8wph64ey6d55zaf48jrveg9wafsquncn3e4scssgvd"

//     const withClient = Effect.provideService(CosmWasmClientContext, { client })

//     const totalSupply = yield* readCw20TotalSupply(tokenAddress).pipe(withClient)
//     yield* Effect.log(`Total Supply of token: ${tokenAddress} on Babylon: ${totalSupply}`)
//   }),
// ).then(exit => console.log(JSON.stringify(exit, null, 2)))

// Read Babylon Channel Balance
// Effect.runPromiseExit(
//   Effect.gen(function*() {
//     // Create a CosmWasm client
//     const client = yield* createCosmWasmClient("https://rpc.bbn-1.babylon.chain.kitchen")

//     const tokenAddress = "bbn1300se0vwue77hn6s8wph64ey6d55zaf48jrveg9wafsquncn3e4scssgvd"
//     const ucs03Address = "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h" // WETH on stargaze
//     const source_channel_id = 1
//     const path = 0n

//     const totalSupply = yield* CosmosChannelBalance(path, tokenAddress).pipe(
//       Effect.provideService(CosmWasmClientDestination, { client }),
//       Effect.provideService(CosmosChannelDestination, {
//         ucs03address: ucs03Address,
//         channelId: source_channel_id,
//       }),
//     )
//     yield* Effect.log(
//       `Channel Balance of token address: ${tokenAddress} on Babylon: ${totalSupply}`,
//     )
//   }),
// ).then(exit => console.log(JSON.stringify(exit, null, 2)))

// Read Babylon Total Balance
// Effect.runPromiseExit(
//   Effect.gen(function*() {
//     // Create a CosmWasm client
//     const client = yield* createCosmWasmClient("https://rpc.bbn-1.babylon.chain.kitchen")

//     const minterAddress = "bbn1c723xf74f0r9g4uyn0cv2t7pkgcq7x0gaw5h773j78rk35w0j0usslxen6"
//     const denom = "ibc/65D0BEC6DAD96C7F5043D1E54E54B6BB5D5B3AEC3FF6CEBB75B9E059F3580EA3"

//     const balance = yield* Effect.tryPromise({
//       try: () => client.getBalance(minterAddress, denom),
//       catch: cause => new Error(`bank query failed: ${cause}`),
//     })

//     yield* Effect.log(
//       `Total Balance of zkgm: ${balance.amount} on Babylon, for denom: ${denom} and minter: ${minterAddress}`,
//     )
//   }),
// ).then(exit => console.log(JSON.stringify(exit, null, 2)))

// Read Ethereum Total Balance
// Effect.runPromiseExit(
//   Effect.gen(function*() {
//     // Create a CosmWasm client
//     const client = createPublicClient({
//       chain: mainnet,
//       transport: http("https://rpc.1.ethereum.chain.kitchen"),
//     })

//     const minterAddress = "0x5fbe74a283f7954f10aa04c2edf55578811aeb03"
//     const denom = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"

//     // Read ERC20 metadata
//     const balance = yield* readErc20Balance(denom, minterAddress).pipe(
//       Effect.provideService(ViemPublicClient, { client }),
//     )
//     yield* Effect.log(
//       `Total Balance of zkgm: ${balance} on Ethereum for denom: ${denom} and minter: ${minterAddress}`,
//     )
//   }),
// ).then(exit => console.log(JSON.stringify(exit, null, 2)))

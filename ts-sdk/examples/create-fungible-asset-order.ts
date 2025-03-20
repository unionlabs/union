import { Effect } from "effect"
import { ViemPublicClientDestination, ViemPublicClientSource } from "../src/evm/client"
import { createPublicClient, http, parseEther, toHex } from "viem"
import { sepolia, holesky } from "viem/chains"
import { DestinationConfig } from "../src/evm/quote-token"
import { CosmosDestinationConfig } from "../src/cosmos/quote-token"
import { 
  createEvmToEvmFungibleAssetOrder, 
  createEvmToCosmosFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createCosmosToCosmosFungibleAssetOrder
} from "../src/evm/ucs03/fungible-asset-order"
import { CosmWasmClientContext, createCosmWasmClient } from "../src/cosmos/client"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

// Example 1: EVM to EVM transfer
console.log("EVM to EVM transfer:")
Effect.runPromiseExit(
  createEvmToEvmFungibleAssetOrder({
    sender: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    receiver: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    baseToken: "0x94373a4919B3240D86eA41593D5eBa789FEF3848", // WETH on holesky
    baseAmount: parseEther("100"), // 100 tokens
    quoteAmount: parseEther("0.05") // 0.05 quote tokens
  }).pipe(
    Effect.provideService(ViemPublicClientSource, {
      client: createPublicClient({
        chain: holesky,
        transport: http()
      })
    }),
    Effect.provideService(ViemPublicClientDestination, {
      client: createPublicClient({
        chain: sepolia,
        transport: http()
      })
    }),
    Effect.provideService(DestinationConfig, {
      ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
      channelId: 8
    })
  )
).then(exit => console.log(JSON.stringify(exit, null, 2)))

// Example 2: Cosmos to EVM transfer
console.log("\nCosmos to EVM transfer:")
Effect.runPromiseExit(
  Effect.gen(function* () {
    const client = yield* createCosmWasmClient("https://rpc.elgafar-1.stargaze-apis.com")
    
    return yield* createCosmosToEvmFungibleAssetOrder({
      sender: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
      receiver: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      baseToken: "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr", // WETH on stargaze
      baseAmount: BigInt(1000000), // 1 token
      quoteAmount: parseEther("0.05") // 0.05 quote tokens
    }).pipe(
      Effect.provideService(CosmWasmClientContext, { client }),
      Effect.provideService(ViemPublicClientDestination, {
        client: createPublicClient({
          chain: sepolia,
          transport: http()
        })
      }),
      Effect.provideService(DestinationConfig, {
        ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
        channelId: 8
      })
    )
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))

// Example 3: EVM to Cosmos transfer
console.log("\nEVM to Cosmos transfer:")
Effect.runPromiseExit(
  Effect.gen(function* () {
    const client = yield* createCosmWasmClient("https://rpc.elgafar-1.stargaze-apis.com")
    
    return yield* createEvmToCosmosFungibleAssetOrder({
      sender: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      receiver: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
      baseToken: "0x94373a4919B3240D86eA41593D5eBa789FEF3848", // WETH on holesky
      baseAmount: parseEther("100"), // 100 tokens
      quoteAmount: BigInt(1000000) // 1 token
    }).pipe(
      Effect.provideService(ViemPublicClientSource, {
        client: createPublicClient({
          chain: holesky,
          transport: http()
        })
      }),
      Effect.provideService(CosmWasmClientContext, { client }),
      Effect.provideService(CosmosDestinationConfig, {
        ucs03address: "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr",
        channelId: 8
      })
    )
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))

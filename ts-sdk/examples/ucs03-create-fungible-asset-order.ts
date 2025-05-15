import { Effect, Logger, LogLevel, ManagedRuntime } from "effect"
import { createPublicClient, http, parseEther, toHex } from "viem"
import { holesky, sepolia } from "viem/chains"
import { ViemPublicClientDestination, ViemPublicClientSource } from "../src/evm/client.js"
import {
  createCosmosToCosmosFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createEvmToCosmosFungibleAssetOrder,
  createEvmToEvmFungibleAssetOrder,
} from "../src/ucs03/fungible-asset-order.js"

import { CosmosChannelDestination, CosmosChannelSource } from "@unionlabs/sdk/cosmos/channel"
import { EvmChannelDestination } from "@unionlabs/sdk/evm/channel"
import { ChannelId, TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"
import { AddressCosmosZkgm, AddressEvmZkgm } from "@unionlabs/sdk/schema/address"
import {
  CosmWasmClientDestination,
  CosmWasmClientSource,
  createCosmWasmClient,
} from "../src/cosmos/client.js"

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

const { runPromiseExit } = ManagedRuntime.make(
  Logger.minimumLogLevel(LogLevel.Error),
)

// Example 1: EVM to EVM transfer
runPromiseExit(
  createEvmToEvmFungibleAssetOrder({
    sender: AddressEvmZkgm.make("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
    receiver: AddressEvmZkgm.make("0x70997970C51812dc3A010C7d01b50e0d17dc79C8"),
    baseToken: TokenRawDenom.make("0x94373a4919B3240D86eA41593D5eBa789FEF3848"), // WETH on holesky
    baseAmount: parseEther("100"), // 100 tokens
    quoteAmount: parseEther("0.05"), // 0.05 quote tokens
    sourceChainId: UniversalChainId.make("ethereum.17000"),
    sourceChannelId: ChannelId.make(2),
  }).pipe(
    Effect.provideService(ViemPublicClientSource, {
      client: createPublicClient({ chain: holesky, transport: http() }),
    }),
    Effect.provideService(ViemPublicClientDestination, {
      client: createPublicClient({ chain: sepolia, transport: http() }),
    }),
    Effect.provideService(EvmChannelDestination, {
      ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
      channelId: 8,
    }),
  ),
).then(exit => console.log("EVM to EVM", JSON.stringify(exit, null, 2)))

// Example 2: Cosmos to EVM transfer
// console.log("\nCosmos to EVM transfer:")
runPromiseExit(
  Effect.gen(function*() {
    const client = yield* createCosmWasmClient("https://rpc.bbn-test-5.babylon.chain.kitchen")

    return yield* createCosmosToEvmFungibleAssetOrder({
      sender: AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh")),
      receiver: AddressEvmZkgm.make("0x70997970c51812dc3a010c7d01b50e0d17dc79c8"),
      baseToken: "ubbn" as unknown as any,
      baseAmount: 1000000n, // 1 token
      quoteAmount: parseEther("0.05"), // 0.05 quote tokens
      sourceChainId: UniversalChainId.make("babylon.bbn-test-5"),
      sourceChannelId: ChannelId.make(2),
    }).pipe(
      Effect.provideService(CosmWasmClientSource, { client }),
      Effect.provideService(ViemPublicClientDestination, {
        client: createPublicClient({ chain: sepolia, transport: http() }),
      }),
      Effect.provideService(EvmChannelDestination, {
        ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
        channelId: 8,
      }),
    )
  }),
).then(exit => console.log("Cosmos to EVM", JSON.stringify(exit, null, 2)))

// Example 3: EVM to Cosmos transfer
runPromiseExit(
  Effect.gen(function*() {
    const client = yield* createCosmWasmClient("https://rpc.bbn-test-5.babylon.chain.kitchen")

    return yield* createEvmToCosmosFungibleAssetOrder({
      sender: AddressEvmZkgm.make("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"),
      receiver: AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh")),
      baseToken: TokenRawDenom.make("0xb476983cc7853797fc5adc4bcad39b277bc79656"),
      baseAmount: parseEther("100"), // 100 tokens
      quoteAmount: 1000000n, // 1 token
      sourceChainId: UniversalChainId.make("ethereum.17000"),
      sourceChannelId: ChannelId.make(2),
    }).pipe(
      Effect.provideService(ViemPublicClientSource, {
        client: createPublicClient({ chain: holesky, transport: http() }),
      }),
      Effect.provideService(CosmWasmClientDestination, { client }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h",
        channelId: 3,
      }),
    )
  }),
).then(exit => console.log("EVM to Cosmos", JSON.stringify(exit, null, 2)))

// Example 4: Cosmos to Cosmos transfer
runPromiseExit(
  Effect.gen(function*() {
    // Create CosmWasm clients for source and destination chains
    const sourceClient = yield* createCosmWasmClient("https://rpc.bbn-test-5.babylon.chain.kitchen")
    const destClient = yield* createCosmWasmClient("https://rpc.osmo-test-5.osmosis.chain.kitchen")

    return yield* createCosmosToCosmosFungibleAssetOrder({
      sender: AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh")),
      receiver: AddressCosmosZkgm.make(toHex("osmo122ny3mep2l7nhtafpwav2y9e5jrslhekuk9g2u")), // Example cosmos address
      baseToken: "ubbn" as unknown as any,
      baseAmount: 1000000n, // 1 token
      quoteAmount: 1000000n, // Expected amount on destination chain
      sourceChainId: UniversalChainId.make("babylon.bbn-test-5"),
      sourceChannelId: ChannelId.make(2),
    }).pipe(
      Effect.provideService(CosmWasmClientSource, { client: sourceClient }),
      Effect.provideService(CosmWasmClientDestination, { client: destClient }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "osmo1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qs2uecc",
        channelId: 1,
      }),
      Effect.provideService(CosmosChannelSource, {
        ucs03address: "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h",
        channelId: 5,
      }),
    )
  }),
).then(exit => console.log("Cosmos to Cosmos", JSON.stringify(exit, null, 2)))

import { assert, describe, it } from "@effect/vitest"
import { ensureHex } from "@unionlabs/sdk/utils/index"
import { Arbitrary, type Context, Effect, Exit, Layer, pipe } from "effect"
import * as fc from "effect/FastCheck"
import { vi } from "vitest"
import { CosmosChannelDestination } from "../../src/cosmos/channel.js"
import { CosmWasmClientDestination, CosmWasmClientSource } from "../../src/cosmos/client.js"
import { EvmChannelDestination } from "../../src/evm/channel.js"
import { ViemPublicClientDestination, ViemPublicClientSource } from "../../src/evm/client.js"
import {
  CosmosToCosmosIntent,
  CosmosToEvmIntent,
  createCosmosToCosmosFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createEvmToCosmosFungibleAssetOrder,
  createEvmToEvmFungibleAssetOrder,
  EvmToCosmosIntent,
  EvmToEvmIntent,
} from "../../src/ucs03/fungible-asset-order.js"

vi.mock("../../src/graphql/unwrapped-quote-token.js", async importOriginal => {
  return {
    ...(await importOriginal<typeof import("../../src/graphql/unwrapped-quote-token.js")>()),
    graphqlQuoteTokenUnwrapQuery: () => Effect.succeed("0x12345"),
  }
})

// Mock data for tests
const mockErc20Meta = {
  name: "Mock ERC20",
  symbol: "MOCK",
  decimals: 18,
}

const mockCw20TokenInfo = {
  name: "Mock CW20",
  symbol: "MCKT",
  decimals: 6,
  total_supply: "1000000000",
}

const mockEvmQuoteToken = "0x123" as const
const mockCosmosQuoteToken = "0x123" as const

// Mock clients
const mockViemPublicClientSource = {
  client: {
    // biome-ignore lint/suspicious/useAwait: reason
    readContract: async (params: any) => {
      // This simulates reading ERC20 metadata based on the function name
      if (params.functionName === "name") {
        return mockErc20Meta.name
      }
      if (params.functionName === "symbol") {
        return mockErc20Meta.symbol
      }
      if (params.functionName === "decimals") {
        return mockErc20Meta.decimals
      }
      return null
    },
  },
}

const mockViemPublicClientDestination = {
  client: {
    // biome-ignore lint/suspicious/useAwait: reason
    readContract: async () => {
      // This simulates predicting a quote token
      return [mockEvmQuoteToken]
    },
  },
}

const mockCosmWasmClientSource = {
  client: {
    // biome-ignore lint/suspicious/useAwait: reason
    queryContractSmart: async (_contractAddress: string, query: any) => {
      // This simulates reading CW20 token info
      if (query.token_info) {
        return mockCw20TokenInfo
      }
      if (query.balance) {
        return { balance: "1000000" }
      }
      return null
    },
  },
}

const mockCosmWasmClientDestination = {
  client: {
    // biome-ignore lint/suspicious/useAwait: reason
    queryContractSmart: async (_contractAddress: string, query: any) => {
      // This simulates predicting a quote token
      if (query.predict_wrapped_token) {
        return { wrapped_token: mockCosmosQuoteToken }
      }
      return null
    },
  },
}

const EvmToEvm = Layer.mergeAll(
  // @ts-expect-error
  Layer.succeed(ViemPublicClientSource, mockViemPublicClientSource),
  // @ts-expect-error
  Layer.succeed(ViemPublicClientDestination, mockViemPublicClientDestination),
  Layer.succeed(EvmChannelDestination, {
    ucs03address: "0xUCS03Address",
    channelId: 1,
  }),
)

const EvmToCosmos = Layer.mergeAll(
  // @ts-expect-error
  Layer.succeed(ViemPublicClientSource, mockViemPublicClientSource),
  // @ts-expect-error
  Layer.succeed(CosmWasmClientDestination, mockCosmWasmClientDestination),
  Layer.succeed(CosmosChannelDestination, {
    ucs03address: "cosmos1ucs03address",
    channelId: 1,
  }),
)

const CosmosToEvm = Layer.mergeAll(
  // @ts-expect-error
  Layer.succeed(CosmWasmClientSource, mockCosmWasmClientSource),
  // @ts-expect-error
  Layer.succeed(ViemPublicClientDestination, mockViemPublicClientDestination),
  Layer.succeed(EvmChannelDestination, {
    ucs03address: "0xUCS03Address",
    channelId: 1,
  }),
)

const CosmosToCosmos = Layer.mergeAll(
  // @ts-expect-error
  Layer.succeed(CosmWasmClientSource, mockCosmWasmClientSource),
  // @ts-expect-error
  Layer.succeed(CosmWasmClientDestination, mockCosmWasmClientDestination),
  Layer.succeed(CosmosChannelDestination, {
    ucs03address: "cosmos1ucs03address",
    channelId: 1,
  }),
)

const EvmToEvmError = Layer.mergeAll(
  EvmToEvm,
  Layer.succeed(ViemPublicClientSource, {
    client: {
      // biome-ignore lint/suspicious/useAwait: reason
      readContract: async () => {
        throw new Error("Mock error")
      },
    },
  } as unknown as Context.Tag.Service<ViemPublicClientSource>),
)

const CosmosToCosmosError = Layer.mergeAll(
  CosmosToCosmos,
  Layer.succeed(CosmWasmClientSource, {
    client: {
      queryContractSmart: () => Promise.reject({}),
    },
  } as unknown as Context.Tag.Service<CosmWasmClientSource>),
)

describe("Fungible Asset Order Tests", () => {
  it.layer(EvmToEvm)("EVM to EVM", it => {
    it.effect.skip("should create a fungible asset order from EVM to EVM", () =>
      Effect.gen(function*() {
        const evmIntent = pipe(
          EvmToEvmIntent,
          Arbitrary.make,
          (arb) => fc.sample(arb, 1)[0],
        )
        const result = yield* createEvmToEvmFungibleAssetOrder(evmIntent)
        assert.deepStrictEqual(result, {
          _tag: "FungibleAssetOrder",
          opcode: 3,
          version: 1,
          operand: [
            evmIntent.sender,
            evmIntent.receiver,
            evmIntent.baseToken,
            evmIntent.baseAmount,
            mockErc20Meta.symbol,
            mockErc20Meta.name,
            mockErc20Meta.decimals,
            0n,
            mockEvmQuoteToken,
            evmIntent.quoteAmount,
          ],
        })
      }))
  })

  it.layer(EvmToCosmos)("EVM to Cosmos", it => {
    it.effect("should create a fungible asset order from EVM to Cosmos", () =>
      Effect.gen(function*() {
        const evmIntent = pipe(
          EvmToCosmosIntent,
          Arbitrary.make,
          (arb) => fc.sample(arb, 1)[0],
        )
        const result = yield* createEvmToCosmosFungibleAssetOrder(evmIntent)
        assert.deepStrictEqual(result, {
          _tag: "FungibleAssetOrder",
          opcode: 3,
          version: 1,
          operand: [
            evmIntent.sender,
            evmIntent.receiver,
            evmIntent.baseToken,
            evmIntent.baseAmount,
            mockErc20Meta.symbol,
            mockErc20Meta.name,
            mockErc20Meta.decimals,
            0n,
            mockCosmosQuoteToken,
            evmIntent.quoteAmount,
          ],
        })
      }))
  })

  it.layer(CosmosToEvm)("Cosmos to EVM", it => {
    it.effect("should create a fungible asset order from Cosmos to EVM", () =>
      Effect.gen(function*() {
        const cosmosIntent = pipe(
          CosmosToEvmIntent,
          Arbitrary.make,
          (arb) => fc.sample(arb, 1)[0],
        )
        const result = yield* createCosmosToEvmFungibleAssetOrder(cosmosIntent)
        assert.deepStrictEqual(result, {
          _tag: "FungibleAssetOrder",
          opcode: 3,
          version: 1,
          operand: [
            cosmosIntent.sender,
            cosmosIntent.receiver,
            ensureHex(cosmosIntent.baseToken),
            cosmosIntent.baseAmount,
            mockCw20TokenInfo.symbol,
            mockCw20TokenInfo.name,
            mockCw20TokenInfo.decimals,
            0n,
            mockEvmQuoteToken,
            cosmosIntent.quoteAmount,
          ],
        })
      }))
  })

  it.layer(CosmosToCosmos)("Cosmos to Cosmos", it => {
    it.effect("should create a fungible asset order from Cosmos to Cosmos", () =>
      Effect.gen(function*() {
        const cosmosIntent = pipe(
          CosmosToCosmosIntent,
          Arbitrary.make,
          (arb) => fc.sample(arb, 1)[0],
        )
        const result = yield* createCosmosToCosmosFungibleAssetOrder(cosmosIntent)
        assert.deepStrictEqual(result, {
          _tag: "FungibleAssetOrder",
          opcode: 3,
          version: 1,
          operand: [
            cosmosIntent.sender,
            cosmosIntent.receiver,
            ensureHex(cosmosIntent.baseToken),
            cosmosIntent.baseAmount,
            mockCw20TokenInfo.symbol,
            mockCw20TokenInfo.name,
            mockCw20TokenInfo.decimals,
            0n,
            mockCosmosQuoteToken,
            cosmosIntent.quoteAmount,
          ],
        })
      }))
  })

  describe("Error handling", () => {
    it.layer(EvmToEvmError)(it => {
      it.effect(
        "should handle errors when creating EVM to EVM fungible asset order with invalid input",
        () =>
          Effect.gen(function*() {
            const evmIntent = pipe(
              EvmToEvmIntent,
              Arbitrary.make,
              (arb) => fc.sample(arb, 1)[0],
            )
            const result = yield* Effect.exit(
              createEvmToEvmFungibleAssetOrder({
                ...evmIntent,
                sender: "nonHexSender",
              } as unknown as any),
            )
            assert.isTrue(Exit.isFailure(result))
          }),
      )
    })

    it.layer(CosmosToCosmosError)(it => {
      /** This is no longer applicable. There is a default handling applied on contract read error. */
      // it.effect("should handle errors when creating Cosmos to Cosmos fungible asset order when contract query fails", () =>
      //   Effect.gen(function* () {
      //     const result = yield* Effect.exit(createCosmosToCosmosFungibleAssetOrder(cosmosIntent))
      //     expect(Exit.isFailure(result)).toBe(true)
      //   })
      // )
    })
  })
})

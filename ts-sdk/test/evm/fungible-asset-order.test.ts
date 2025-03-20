import { describe, it, expect } from "vitest"
import { Effect } from "effect"
import { ViemPublicClientSource, ViemPublicClientDestination } from "../../src/evm/client.js"
import { CosmWasmClientSource, CosmWasmClientDestination } from "../../src/cosmos/client.js"
import { DestinationConfig } from "../../src/evm/quote-token.js"
import { CosmosDestinationConfig } from "../../src/cosmos/quote-token.js"
import {
  createEvmToEvmFungibleAssetOrder,
  createEvmToCosmosFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createCosmosToCosmosFungibleAssetOrder
} from "../../src/ucs03/fungible-asset-order.js"
import { toHex } from "viem"

// Mock data for tests
const mockErc20Meta = {
  name: "Mock ERC20",
  symbol: "MOCK",
  decimals: 18
}

const mockCw20TokenInfo = {
  name: "Mock CW20",
  symbol: "MCKT",
  decimals: 6,
  total_supply: "1000000000"
}

const mockEvmQuoteToken = "0xMockQuoteToken" as const
const mockCosmosQuoteToken = "cosmos1mockquotetoken"

// Mock clients
const mockViemPublicClientSource = {
  client: {
    readContract: async (params: any) => {
      // This simulates reading ERC20 metadata based on the function name
      if (params.functionName === "name") return mockErc20Meta.name
      if (params.functionName === "symbol") return mockErc20Meta.symbol
      if (params.functionName === "decimals") return mockErc20Meta.decimals
      return null
    }
  }
}

const mockViemPublicClientDestination = {
  client: {
    readContract: async () => {
      // This simulates predicting a quote token
      return [mockEvmQuoteToken]
    }
  }
}

const mockCosmWasmClientSource = {
  client: {
    queryContractSmart: async (_contractAddress: string, query: any) => {
      // This simulates reading CW20 token info
      if (query.token_info) {
        return mockCw20TokenInfo
      }
      if (query.balance) {
        return { balance: "1000000" }
      }
      return null
    }
  }
}

const mockCosmWasmClientDestination = {
  client: {
    queryContractSmart: async (_contractAddress: string, query: any) => {
      // This simulates predicting a quote token
      if (query.predict_wrapped_token) {
        return { wrapped_token: mockCosmosQuoteToken }
      }
      return null
    }
  }
}

// Test data
const evmIntent = {
  sender: "0xSender" as const,
  receiver: "0xReceiver" as const,
  baseToken: "0xBaseToken" as const,
  baseAmount: BigInt(1000000000000000000n), // 1 token with 18 decimals
  quoteAmount: BigInt(500000000000000000n) // 0.5 token with 18 decimals
}

const cosmosIntent = {
  sender: "cosmos1sender",
  receiver: "cosmos1receiver",
  baseToken: "cosmos1basetoken",
  baseAmount: BigInt(1000000), // 1 token with 6 decimals
  quoteAmount: BigInt(500000) // 0.5 token with 6 decimals
}

describe("Fungible Asset Order Tests", () => {
  describe("EVM to EVM", () => {
    it("should create a fungible asset order from EVM to EVM", async () => {
      const result = await Effect.runPromise(
        createEvmToEvmFungibleAssetOrder(evmIntent).pipe(
          //@ts-ignore: its a mock
          Effect.provideService(ViemPublicClientSource, mockViemPublicClientSource),
          //@ts-ignore: its a mock
          Effect.provideService(ViemPublicClientDestination, mockViemPublicClientDestination),
          Effect.provideService(DestinationConfig, {
            ucs03address: "0xUCS03Address",
            channelId: 1
          })
        )
      )

      expect(result).toEqual([
        evmIntent.sender,
        evmIntent.receiver,
        evmIntent.baseToken,
        evmIntent.baseAmount,
        mockErc20Meta.symbol,
        mockErc20Meta.name,
        mockErc20Meta.decimals,
        0,
        mockEvmQuoteToken,
        evmIntent.quoteAmount
      ])
    })
  })

  describe("EVM to Cosmos", () => {
    it("should create a fungible asset order from EVM to Cosmos", async () => {
      const result = await Effect.runPromise(
        createEvmToCosmosFungibleAssetOrder(evmIntent).pipe(
          //@ts-ignore: its a mock
          Effect.provideService(ViemPublicClientSource, mockViemPublicClientSource),
          //@ts-ignore: its a mock
          Effect.provideService(CosmWasmClientDestination, mockCosmWasmClientDestination),
          Effect.provideService(CosmosDestinationConfig, {
            ucs03address: "cosmos1ucs03address",
            channelId: 1
          })
        )
      )

      expect(result).toEqual([
        evmIntent.sender,
        evmIntent.receiver,
        evmIntent.baseToken,
        evmIntent.baseAmount,
        mockErc20Meta.symbol,
        mockErc20Meta.name,
        mockErc20Meta.decimals,
        0,
        mockCosmosQuoteToken,
        evmIntent.quoteAmount
      ])
    })
  })

  describe("Cosmos to EVM", () => {
    it("should create a fungible asset order from Cosmos to EVM", async () => {
      const result = await Effect.runPromise(
        // @ts-ignore
        createCosmosToEvmFungibleAssetOrder(cosmosIntent).pipe(
          //@ts-ignore: its a mock
          Effect.provideService(CosmWasmClientSource, mockCosmWasmClientSource),
          //@ts-ignore: its a mock
          Effect.provideService(ViemPublicClientDestination, mockViemPublicClientDestination),
          Effect.provideService(DestinationConfig, {
            ucs03address: "0xUCS03Address",
            channelId: 1
          })
        )
      )

      expect(result).toEqual([
        cosmosIntent.sender,
        cosmosIntent.receiver,
        toHex(cosmosIntent.baseToken),
        cosmosIntent.baseAmount,
        mockCw20TokenInfo.symbol,
        mockCw20TokenInfo.name,
        mockCw20TokenInfo.decimals,
        0,
        mockEvmQuoteToken,
        cosmosIntent.quoteAmount
      ])
    })
  })

  describe("Cosmos to Cosmos", () => {
    it("should create a fungible asset order from Cosmos to Cosmos", async () => {
      const result = await Effect.runPromise(
        // @ts-ignore
        createCosmosToCosmosFungibleAssetOrder(cosmosIntent).pipe(
          //@ts-ignore: its a mock
          Effect.provideService(CosmWasmClientSource, mockCosmWasmClientSource),
          //@ts-ignore: its a mock
          Effect.provideService(CosmWasmClientDestination, mockCosmWasmClientDestination),
          Effect.provideService(CosmosDestinationConfig, {
            ucs03address: "cosmos1ucs03address",
            channelId: 1
          })
        )
      )

      expect(result).toEqual([
        cosmosIntent.sender,
        cosmosIntent.receiver,
        cosmosIntent.baseToken,
        cosmosIntent.baseAmount,
        mockCw20TokenInfo.symbol,
        mockCw20TokenInfo.name,
        mockCw20TokenInfo.decimals,
        0,
        mockCosmosQuoteToken,
        cosmosIntent.quoteAmount
      ])
    })
  })

  describe("Error handling", () => {
    it("should handle errors when creating EVM to EVM fungible asset order", async () => {
      const errorClient = {
        client: {
          readContract: async () => {
            throw new Error("Mock error")
          }
        }
      }

      const result = await Effect.runPromiseExit(
        createEvmToEvmFungibleAssetOrder(evmIntent).pipe(
          //@ts-ignore: its a mock
          Effect.provideService(ViemPublicClientSource, errorClient),
          //@ts-ignore: its a mock
          Effect.provideService(ViemPublicClientDestination, mockViemPublicClientDestination),
          Effect.provideService(DestinationConfig, {
            ucs03address: "0xUCS03Address",
            channelId: 1
          })
        )
      )

      expect(result._tag).toBe("Failure")
    })

    it("should handle errors when creating Cosmos to Cosmos fungible asset order", async () => {
      const errorClient = {
        client: {
          queryContractSmart: async () => {
            throw new Error("Mock error")
          }
        }
      }

      const result = await Effect.runPromiseExit(
        //@ts-ignore: its a mock
        createCosmosToCosmosFungibleAssetOrder(cosmosIntent).pipe(
          //@ts-ignore: its a mock
          Effect.provideService(CosmWasmClientSource, errorClient),
          //@ts-ignore: its a mock
          Effect.provideService(CosmWasmClientDestination, mockCosmWasmClientDestination),
          Effect.provideService(CosmosDestinationConfig, {
            ucs03address: "cosmos1ucs03address",
            channelId: 1
          })
        )
      )

      expect(result._tag).toBe("Failure")
    })
  })
})

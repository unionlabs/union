import { describe, it, expect, vi, beforeEach } from "vitest"
import { Effect } from "effect"
import {
  readErc20Meta,
  readErc20Name,
  readErc20Symbol,
  readErc20Decimals
} from "../../src/evm/erc20.js"
import { ViemPublicClient } from "../../src/evm/client.js"
import { erc20Abi, type PublicClient } from "viem"

// Mock client for testing
const mockClient = {
  readContract: vi.fn()
}

// Mock service layer
const mockViemPublicClient = {
  // @ts-ignore we do a partial mock
  client: mockClient as PublicClient
}

describe("ERC20 Module", () => {
  const testTokenAddress = "0x1234567890123456789012345678901234567890"

  beforeEach(() => {
    vi.resetAllMocks()
  })

  describe("readErc20Name", () => {
    it("should return the token name", async () => {
      // Setup mock
      mockClient.readContract.mockResolvedValueOnce("Test Token")

      // Execute
      const result = await Effect.runPromise(
        readErc20Name(testTokenAddress).pipe(
          Effect.provideService(ViemPublicClient, mockViemPublicClient)
        )
      )

      // Verify
      expect(result).toBe("Test Token")
      expect(mockClient.readContract).toHaveBeenCalledWith({
        address: testTokenAddress,
        abi: erc20Abi,
        functionName: "name"
      })
    })

    it("should handle errors", async () => {
      // Setup mock to throw
      const testError = new Error("Contract error")
      mockClient.readContract.mockRejectedValueOnce(testError)

      // Execute and verify
      await expect(
        Effect.runPromise(
          readErc20Name(testTokenAddress).pipe(
            Effect.provideService(ViemPublicClient, mockViemPublicClient)
          )
        )
      ).rejects.toThrow()
    })
  })

  describe("readErc20Symbol", () => {
    it("should return the token symbol", async () => {
      // Setup mock
      mockClient.readContract.mockResolvedValueOnce("TKN")

      // Execute
      const result = await Effect.runPromise(
        readErc20Symbol(testTokenAddress).pipe(
          Effect.provideService(ViemPublicClient, mockViemPublicClient)
        )
      )

      // Verify
      expect(result).toBe("TKN")
      expect(mockClient.readContract).toHaveBeenCalledWith({
        address: testTokenAddress,
        abi: erc20Abi,
        functionName: "symbol"
      })
    })
  })

  describe("readErc20Decimals", () => {
    it("should return the token decimals", async () => {
      // Setup mock
      mockClient.readContract.mockResolvedValueOnce(18)

      // Execute
      const result = await Effect.runPromise(
        readErc20Decimals(testTokenAddress).pipe(
          Effect.provideService(ViemPublicClient, mockViemPublicClient)
        )
      )

      // Verify
      expect(result).toBe(18)
      expect(mockClient.readContract).toHaveBeenCalledWith({
        address: testTokenAddress,
        abi: erc20Abi,
        functionName: "decimals"
      })
    })
  })

  describe("readErc20Meta", () => {
    it.fails("should return all token metadata", async () => {
      // Setup mocks
      mockClient.readContract.mockResolvedValueOnce("Test Token") // name
      mockClient.readContract.mockResolvedValueOnce("TKN") // symbol
      mockClient.readContract.mockResolvedValueOnce(18) // decimals

      // Execute
      const result = await Effect.runPromise(
        readErc20Meta(testTokenAddress).pipe(
          Effect.provideService(ViemPublicClient, mockViemPublicClient)
        )
      )

      // Verify
      expect(result).toEqual({
        name: "Test Token",
        symbol: "TKN",
        decimals: 18
      })
      expect(mockClient.readContract).toHaveBeenCalledTimes(3)
    })
  })
})

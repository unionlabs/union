import { describe, it, expect, vi, beforeEach } from "vitest"
import { Effect } from "effect"
import { readContract, writeContract } from "../../src/evm/contract.js"
import { PublicViemClient, ViemWalletClient } from "../../src/evm/client.js"
import { type PublicClient, type WalletClient } from "viem"

// Mock clients for testing
const mockPublicClient = {
  readContract: vi.fn()
}

const mockWalletClient = {
  writeContract: vi.fn()
}

// Mock service layers
const mockPublicViemClient = {
  // @ts-ignore we do a partial mock
  client: mockPublicClient as PublicClient
}

const mockViemWalletClient = {
  // @ts-ignore we do a partial mock
  client: mockWalletClient as WalletClient
}

describe("Contract Module", () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  describe("readContract", () => {
    it("should call client.readContract with the provided parameters", async () => {
      // Setup mock
      const mockResult = "Test Result"
      mockPublicClient.readContract.mockResolvedValueOnce(mockResult)

      // Test parameters
      const testParams = {
        address: "0x1234567890123456789012345678901234567890",
        abi: [{ name: "test", type: "function", inputs: [], outputs: [{ type: "string" }] }],
        functionName: "test"
      }

      // Execute
      const result = await Effect.runPromise(
        readContract(mockPublicClient, testParams)
      )

      // Verify
      expect(result).toBe(mockResult)
      expect(mockPublicClient.readContract).toHaveBeenCalledWith(testParams)
    })

    it("should handle errors", async () => {
      // Setup mock to throw
      const testError = new Error("Contract read error")
      mockPublicClient.readContract.mockRejectedValueOnce(testError)

      // Test parameters
      const testParams = {
        address: "0x1234567890123456789012345678901234567890",
        abi: [{ name: "test", type: "function", inputs: [], outputs: [{ type: "string" }] }],
        functionName: "test"
      }

      // Execute and verify
      await expect(
        Effect.runPromise(readContract(mockPublicClient, testParams))
      ).rejects.toThrow()
    })
  })

  describe("writeContract", () => {
    it("should call client.writeContract with the provided parameters", async () => {
      // Setup mock
      const mockTxHash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
      mockWalletClient.writeContract.mockResolvedValueOnce(mockTxHash)

      // Test parameters
      const testParams = {
        address: "0x1234567890123456789012345678901234567890",
        abi: [{ name: "test", type: "function", inputs: [], outputs: [] }],
        functionName: "test",
        account: "0x1234567890123456789012345678901234567890"
      }

      // Execute
      const result = await Effect.runPromise(
        writeContract(mockWalletClient, testParams)
      )

      // Verify
      expect(result).toBe(mockTxHash)
      expect(mockWalletClient.writeContract).toHaveBeenCalledWith(testParams)
    })

    it("should handle errors", async () => {
      // Setup mock to throw
      const testError = new Error("Contract write error")
      mockWalletClient.writeContract.mockRejectedValueOnce(testError)

      // Test parameters
      const testParams = {
        address: "0x1234567890123456789012345678901234567890",
        abi: [{ name: "test", type: "function", inputs: [], outputs: [] }],
        functionName: "test",
        account: "0x1234567890123456789012345678901234567890"
      }

      // Execute and verify
      await expect(
        Effect.runPromise(writeContract(mockWalletClient, testParams))
      ).rejects.toThrow()
    })
  })
})

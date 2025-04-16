import { describe, it, expect, vi, beforeEach } from "vitest"
import { Effect } from "effect"
import { readContract } from "../../src/evm/contract.js"
import type { WalletClient } from "viem"

// Mock clients for testing
// @ts-ignore we do a partial mock
const mockPublicClient = {
  readContract: vi.fn()
}

// @ts-ignore we do a partial mock
const mockWalletClient = {
  writeContract: vi.fn()
} as WalletClient

// Mock service layers
// @ts-expect-error
const mockPublicViemClient = {
  client: mockPublicClient
}

// @ts-expect-error
const mockViemWalletClient = {
  // @ts-ignore we do a partial mock
  client: mockWalletClient
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
      // @ts-ignore it's a mock client
      const result = await Effect.runPromise(readContract(mockPublicClient, testParams))

      // Verify
      expect(result).toBe(mockResult)
      expect(mockPublicClient.readContract).toHaveBeenCalledWith(testParams)
    })
  })
})

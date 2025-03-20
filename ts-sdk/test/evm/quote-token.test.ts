import { describe, it, expect, vi, beforeEach } from "vitest"
import { Effect } from "effect"
import { DestinationConfig, predictQuoteToken } from "../../src/evm/quote-token.js"
import { ViemPublicClientDestination } from "../../src/evm/client.js"
import { ucs03abi } from "../../src/evm/abi/ucs03.js"
import { toHex } from "viem"

// Mock client for testing
const mockClient = {
  readContract: vi.fn()
}

// Mock service layer
const mockViemPublicClientDestination = {
  // @ts-ignore we do a partial mock
  client: mockClient as PublicClient
}

describe("Quote Token Module", () => {
  const testParams = {
    baseToken: toHex("muno"),
    ucs03address: "0x05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5",
    destinationChannelId: 1
  } as const

  beforeEach(() => {
    vi.resetAllMocks()
  })

  it("should return the predicted wrapped token address", async () => {
    // Setup mock to return a tuple with the address
    const predictedAddress = "0xabcdef1234567890abcdef1234567890abcdef12"
    mockClient.readContract.mockResolvedValueOnce([predictedAddress])

    // Execute
    const result = await Effect.runPromise(
      predictQuoteToken(testParams.baseToken).pipe(
        Effect.provideService(ViemPublicClientDestination, mockViemPublicClientDestination),
        Effect.provideService(DestinationConfig, {
          ucs03address: testParams.ucs03address,
          channelId: testParams.destinationChannelId
        })
      )
    )

    // Verify
    expect(result).toBe(predictedAddress)
    expect(mockClient.readContract).toHaveBeenCalledWith({
      address: testParams.ucs03address,
      abi: ucs03abi,
      functionName: "predictWrappedToken",
      args: [0n, testParams.destinationChannelId, testParams.baseToken]
    })
  })
})

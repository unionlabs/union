import { Effect } from "effect"
import type { Address, Hex, ReadContractErrorType } from "viem"
import { PublicDestinationViemClient, ReadContractError } from "./client.js"
import { ucs03abi } from "./abi/ucs03.js"

function extractErrorDetails<T extends Error>(
  error: T
): {
  [K in keyof T]: T[K]
} & {
  message: string
  name: string
  stack?: string
  cause?: unknown
} {
  const extractedError = {} as {
    [K in keyof T]: T[K]
  } & {
    message: string
    name: string
    stack?: string
    cause?: unknown
  }

  // Extract all own properties, including non-enumerable ones
  Object.getOwnPropertyNames(error).forEach(key => {
    extractedError[key as keyof T] = error[key as keyof T]
  })

  // Explicitly copy inherited properties
  extractedError.message = error.message
  extractedError.name = error.name
  if (error.stack) extractedError.stack = error.stack
  if ("cause" in error) extractedError.cause = error.cause

  return extractedError
}

export const quoteToken = (baseToken: Hex, ucs03address: Address, destinationChannelId: number) =>
  Effect.gen(function* () {
    let client = (yield* PublicDestinationViemClient).client

    const predictedQuoteToken = yield* Effect.tryPromise({
      try: () =>
        client.readContract({
          address: ucs03address,
          abi: ucs03abi,
          functionName: "predictWrappedToken",
          args: [0n, destinationChannelId, baseToken]
        }),
      catch: error =>
        new ReadContractError({ cause: extractErrorDetails(error as ReadContractErrorType) })
    }).pipe(Effect.map(([address]) => address))

    return predictedQuoteToken
  })

import { Effect } from "effect"
import { Data } from "effect"
import type { Hash, WaitForTransactionReceiptTimeoutErrorType } from "viem"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { ViemPublicClient } from "./client.js"

export class WaitForTransactionReceiptError extends Data.TaggedError(
  "WaitForTransactionReceiptError",
)<{
  cause: WaitForTransactionReceiptTimeoutErrorType
}> {}

/**
 * Wait for a transaction receipt
 * @param hash The transaction hash to wait for
 * @returns An Effect that resolves to the transaction receipt
 */
export const waitForTransactionReceipt = (hash: Hash) =>
  Effect.gen(function*() {
    const client = (yield* ViemPublicClient).client

    const receipt = yield* Effect.tryPromise({
      try: () => client.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({
          cause: extractErrorDetails(err as WaitForTransactionReceiptTimeoutErrorType),
        }),
    })

    return receipt
  })

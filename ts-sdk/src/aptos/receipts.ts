import { Effect } from "effect"
import { Data } from "effect"
import { AptosPublicClient } from "./client.js"
import { extractErrorDetails } from "../utils/extract-error-details.js"

export type Hash = `0x${string}`

export class WaitForTransactionReceiptError extends Data.TaggedError(
  "WaitForTransactionReceiptError"
)<{
  cause: WaitForTransactionReceiptError
}> {}

/**
 * Wait for a transaction receipt
 * @param hash The transaction hash to wait for
 * @returns An Effect that resolves to the transaction receipt
 */
export const waitForTransactionReceipt = (hash: Hash) =>
  Effect.gen(function* () {
    const client = (yield* AptosPublicClient).client

    const receipt = yield* Effect.tryPromise({
      try: () => client.waitForTransaction({ hash, options: { checkSuccess: false } }),
      catch: err =>
        new WaitForTransactionReceiptError({
          cause: extractErrorDetails(err as WaitForTransactionReceiptError)
        })
    })

    return receipt
  })

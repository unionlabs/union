import { Effect, Data } from "effect"
import type { SuiClient } from "@mysten/sui/client"
import { Transaction } from "@mysten/sui/transactions"
import { extractErrorDetails } from "../utils/extract-error-details.js"

/**
 * Error type for Aptos contract query failures
 */
export class readContractError extends Data.TaggedError("readContractError")<{
  cause: unknown
}> {}

/**
 * Error type for Aptos contract execution failures
 */
export class ExecuteContractError extends Data.TaggedError("ExecuteContractError")<{
  cause: unknown
}> {}

// TODO: add comments
export const readContract = <T>(
  client: SuiClient,
  sender: string,
  packageId: string,
  module: string,
  fn: string,
  typeArgs: string[],
  args: any[],
  tx: Transaction
) =>
  Effect.tryPromise({
    try: async () => {
      // build a dummy Tx that does the desired Move call
      tx.moveCall({
        target: `${packageId}::${module}::${fn}`,
        typeArguments: typeArgs,
        arguments: args,
      })
      // dev-inspect it
      const result = await client.devInspectTransactionBlock({
        transactionBlock: tx,
        sender,
      })
      // unwrap your return value however you like
      return result.results // result as unknown as T
    },
    catch: e => new readContractError({cause: extractErrorDetails(e as Error)}),
  }).pipe(
    // optional: e.g. timeout & retry like your Aptos wrapper
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
  )

export const writeContract = (
  client: SuiClient,
  signer: { address: string; signAndExecute: any },
  packageId: string,
  module: string,
  fn: string,
  typeArgs: string[],
  args: any[],
) =>
  Effect.tryPromise({
    try: async () => {
      const tx = new Transaction()
      tx.moveCall({
        target: `${packageId}::${module}::${fn}`,
        typeArguments: typeArgs,
        arguments: args,
      })
      // sign & execute
      const res = await client.signAndExecuteTransaction({
        signer,
        transaction: tx,
      })
      return res
    },
    catch: e => new ExecuteContractError({cause: extractErrorDetails(e as Error)}),  })

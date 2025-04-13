import { Effect, Data } from "effect"
import type { SigningCosmWasmClient, CosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { extractErrorDetails } from "../utils/extract-error-details.js"

/**
 * Error type for CosmWasm contract query failures
 */
export class QueryContractError extends Data.TaggedError("QueryContractError")<{
  cause: unknown
}> {}

/**
 * Error type for CosmWasm contract execution failures
 */
export class ExecuteContractError extends Data.TaggedError("ExecuteContractError")<{
  cause: unknown
  message: string
}> {}

/**
 * A type-safe wrapper around CosmWasm's queryContract that handles error cases
 * and returns an Effect with proper type inference.
 *
 * @param client - The CosmWasmClient to use for the contract query
 * @param contractAddress - The address of the contract to query
 * @param queryMsg - The query message to send to the contract
 * @returns An Effect that resolves to the properly typed return value
 */
export const queryContract = <T = unknown>(
  client: CosmWasmClient,
  contractAddress: string,
  queryMsg: Record<string, unknown>
) =>
  Effect.tryPromise({
    try: async () => {
      const result = await client.queryContractSmart(contractAddress, queryMsg)
      return result as T
    },
    catch: error => new QueryContractError({ cause: extractErrorDetails(error as Error) })
  }).pipe(Effect.timeout("10 seconds"), Effect.retry({ times: 5 }))

/**
 * A type-safe wrapper around CosmWasm's executeContract that handles error cases
 * and returns an Effect with proper type inference.
 *
 * @param client - The SigningCosmWasmClient to use for the contract execution
 * @param senderAddress - The address of the sender executing the contract
 * @param contractAddress - The address of the contract to execute
 * @param msg - The execute message to send to the contract
 * @param funds - Optional funds to send with the transaction
 * @returns An Effect that resolves to the execution result
 */
export const executeContract = (
  client: SigningCosmWasmClient,
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
  funds?: ReadonlyArray<{ denom: string; amount: string }>
) =>
  Effect.tryPromise({
    try: () => client.execute(senderAddress, contractAddress, msg, "auto", undefined, funds),
    catch: error =>
      new ExecuteContractError({
        cause: extractErrorDetails(error as Error),
        message: (error as Error).message
      })
  })

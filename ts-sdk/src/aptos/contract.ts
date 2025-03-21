import { Effect, Data } from "effect"
import { Aptos, AptosConfig, Network, AptosApiError, type Account as AptosAccount } from "@aptos-labs/ts-sdk"
  import { extractErrorDetails } from "../utils/extract-error-details.js"
  import type { AptosBrowserWallet } from "./wallet.js"
  import { waitForTransactionReceipt } from "./receipts.js"

/**
 * Error type for Aptos contract query failures
 */
export class QueryContractError extends Data.TaggedError("QueryContractError")<{
  cause: unknown
}> {}

/**
 * Error type for Aptos contract execution failures
 */
export class ExecuteContractError extends Data.TaggedError("ExecuteContractError")<{
  cause: unknown
}> {}


// TODO: add comments
export const queryContract = <T = unknown>(
  client: Aptos,
  contractAddress: string,
  function_name: string, // `ibc_app::predict_wrapped_token` as an example.
  typeArguments: Array<any>,
  functionArguments: Array<any>
) =>
  Effect.tryPromise({
    try: async () => {
      const result = await client.view({
        payload: {
          function: `${contractAddress}::${function_name}`,
          typeArguments: typeArguments,
          functionArguments: functionArguments
        }
      })
      return result as T
    },
    catch: error => new QueryContractError({ cause: extractErrorDetails(error as Error) })
  }).pipe(Effect.timeout("10 seconds"), Effect.retry({ times: 5 }))

// TODO: add comments
export const executeContractWithWallet = (
  client: AptosBrowserWallet,
  contractAddress: string,
  function_name: string, // `ibc_app::predict_wrapped_token` as an example.
  typeArguments: Array<any>,
  functionArguments: Array<any>
) =>
  Effect.tryPromise({
    try: async () => {
      const walletPayload = {
        function: `${contractAddress}::${function_name}`,
        typeArguments: typeArguments,
        functionArguments: functionArguments
      }
      const result = await client.signAndSubmitTransaction({ payload: walletPayload })
      return result 
    },
    catch: error => new ExecuteContractError({ cause: extractErrorDetails(error as Error) })
  })


export const executeContractWithKey = (
    client: Aptos,
    signer: AptosAccount,
    contractAddress: string,
    function_name: string, // `ibc_app::predict_wrapped_token` as an example.
    typeArguments: Array<any>,
    functionArguments: Array<any>
  ) =>
    Effect.tryPromise({
      try: async () => {
      const payload = await client.transaction.build.simple({
        sender: signer.accountAddress,
        data: {
          function: `${contractAddress}::${function_name}`,
          typeArguments: typeArguments,
          functionArguments: functionArguments
        }
        })

        const txn = await client.signAndSubmitTransaction({ signer: signer, transaction: payload})
        return txn
        
      },
      catch: error => new ExecuteContractError({ cause: extractErrorDetails(error as Error) })
    })
  
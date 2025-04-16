import { Effect, Data } from "effect"
import type { Aptos, Account as AptosAccount } from "@aptos-labs/ts-sdk"
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
export const readContract = <T = unknown>(
  client: Aptos,
  contractAddress: string,
  module_name: string,
  function_name: string,
  typeArguments: Array<any>,
  functionArguments: Array<any>
) =>
  Effect.tryPromise({
    try: async () => {
      const result = await client.view({
        payload: {
          function: `${contractAddress}::${module_name}::${function_name}`,
          typeArguments: typeArguments,
          functionArguments: functionArguments
        }
      })
      return result as [T]
    },
    catch: error => new readContractError({ cause: extractErrorDetails(error as Error) })
  }).pipe(Effect.timeout("10 seconds"), Effect.retry({ times: 5 }))

// TODO: add comments - fix it when integrating with app2
// export const executeContractWithWallet = (
//   client: AptosBrowserWallet,
//   contractAddress: string,
//   module_name: string,
//   function_name: string,
//   typeArguments: Array<any>,
//   functionArguments: Array<any>
// ) =>
//   Effect.tryPromise({
//     try: async () => {
//       const walletPayload = {
//         function: `${contractAddress}::${module_name}::${function_name}`,
//         typeArguments: typeArguments,
//         functionArguments: functionArguments
//       }
//       const result = await client.signAndSubmitTransaction({ payload: walletPayload })
//       return result
//     },
//     catch: error => new ExecuteContractError({ cause: extractErrorDetails(error as Error) })
//   })

export const writeContract = (
  client: Aptos,
  signer: AptosAccount,
  contractAddress: string,
  module_name: string,
  function_name: string, // `ibc_app::predict_wrapped_token` as an example.
  typeArguments: Array<any>,
  functionArguments: Array<any>
) =>
  Effect.tryPromise({
    try: async () => {
      const payload = await client.transaction.build.simple({
        sender: signer.accountAddress,
        data: {
          function: `${contractAddress}::${module_name}::${function_name}`,
          typeArguments: typeArguments,
          functionArguments: functionArguments
        }
      })

      const txn = await client.signAndSubmitTransaction({ signer: signer, transaction: payload })
      return txn
    },
    catch: error => new ExecuteContractError({ cause: extractErrorDetails(error as Error) })
  })

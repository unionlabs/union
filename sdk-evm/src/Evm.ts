import * as Utils from "@unionlabs/sdk/Utils"
import { Data, Effect, pipe } from "effect"
import type {
  Abi,
  ContractFunctionArgs,
  ContractFunctionName,
  WriteContractErrorType,
  WriteContractParameters,
} from "viem"
import * as Wallet from "./Wallet.js"

/**
 * @category errors
 * @since 2.0.0
 */
export class WriteContractError extends Data.TaggedError("@unionlabs/sdk/Evm/WriteContractError")<{
  cause: WriteContractErrorType
}> {}

/**
 * A type-safe wrapper around viem's writeContract that handles error cases
 * and returns an Effect with proper type inference. Extracts all error info
 *
 * @param client - The viem WalletClient to use for the contract transaction
 * @param params - The parameters for the contract transaction
 * @returns An Effect that resolves to the transaction hash
 *
 * @category utils
 * @since 2.0.0
 */
export const writeContract = <
  TAbi extends Abi,
  TFunctionName extends ContractFunctionName<TAbi, "nonpayable" | "payable"> = ContractFunctionName<
    TAbi,
    "nonpayable" | "payable"
  >,
  TArgs extends ContractFunctionArgs<
    TAbi,
    "nonpayable" | "payable",
    TFunctionName
  > = ContractFunctionArgs<TAbi, "nonpayable" | "payable", TFunctionName>,
>(
  params: WriteContractParameters<TAbi, TFunctionName, TArgs>,
) =>
  pipe(
    Wallet.Wallet,
    Effect.andThen(({ client }) =>
      Effect.tryPromise({
        try: () => client.writeContract(params),
        catch: error =>
          new WriteContractError({
            cause: Utils.extractErrorDetails(error as WriteContractErrorType),
          }),
      })
    ),
  )

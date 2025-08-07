import { Ucs03, Utils } from "@unionlabs/sdk"
import { Hex } from "@unionlabs/sdk/schema/hex"
import { Context, Data, Effect, pipe } from "effect"
import {
  Abi,
  Account as ViemAccount,
  Chain as ViemChain,
  ContractFunctionArgs,
  ContractFunctionName,
  CreateWalletClientErrorType,
  WalletClient as ViemWalletClient,
  WriteContractErrorType,
  WriteContractParameters,
} from "viem"
import * as Evm from "./Evm.js"
import * as internal from "./internal/wallet.js"

export interface EvmWallet {
  readonly client: ViemWalletClient
  readonly account: ViemAccount
  readonly chain: ViemChain
}

/**
 * @category errors
 * @since 2.0.0
 */
export class CreateWalletError
  extends Data.TaggedError("@unionlabs/sdk-evm/EvmWallet/CreateWalletError")<{
    cause: CreateWalletClientErrorType
  }>
{}

/**
 * A wallet client that can be used for signing transactions
 *
 * @category context
 * @since 2.0.0
 */
export class EvmWallet extends Context.Tag("@unionlabs/sdk-evm/EvmWallet/EvmWallet")<
  EvmWallet,
  EvmWallet
>() {
  static Live = internal.walletClientLayer(this)
}

/**
 * @category errors
 * @since 2.0.0
 */
export class WriteContractError
  extends Data.TaggedError("@unionlabs/sdk-evm/EvmWallet/WriteContractError")<{
    cause: WriteContractErrorType
  }>
{}

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
    EvmWallet,
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

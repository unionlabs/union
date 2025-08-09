/**
 * TODO
 *
 * @since 0.0.0
 */
import { Chain } from "@unionlabs/sdk/schema/chain"
import { Data, Effect, pipe } from "effect"
import type {
  Abi,
  ContractFunctionArgs,
  ContractFunctionName,
  Hash,
  WriteContractParameters,
} from "viem"
import * as Evm from "./Evm.js"

/** @since 0.0.0 */
export type TransactionState = Data.TaggedEnum<{
  WriteContractInProgress: {}
  WriteContractComplete: { exit: Effect.Effect.Success<ReturnType<typeof Evm.writeContract>> }
  TransactionReceiptInProgress: { readonly hash: Hash } // on chain hash
  TransactionReceiptComplete: {
    exit: Effect.Effect.Success<ReturnType<typeof Evm.waitForTransactionReceipt>>
  }
}>

/** @since 0.0.0 */
export const TransactionState = Data.taggedEnum<TransactionState>()
/** @since 0.0.0 */
export const {
  WriteContractInProgress,
  WriteContractComplete,
  TransactionReceiptInProgress,
  TransactionReceiptComplete,
  $is: is,
} = TransactionState

/**
 * @since 0.0.0
 */
export const nextState = <
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
  ts: TransactionState,
  chain: Chain,
  params: WriteContractParameters<TAbi, TFunctionName, TArgs>,
): Effect.Effect<
  TransactionState,
  Evm.WaitForTransactionReceiptError | Evm.WriteContractError,
  Evm.WalletClient | Evm.PublicClient
> =>
  TransactionState.$match(ts, {
    WriteContractInProgress: () =>
      pipe(
        Evm.writeContract(params),
        Effect.map((exit) =>
          WriteContractComplete({
            exit,
          })
        ),
      ),

    WriteContractComplete: ({ exit: hash }) =>
      Effect.succeed(TransactionReceiptInProgress({ hash })),

    TransactionReceiptInProgress: ({ hash }) =>
      pipe(
        Evm.waitForTransactionReceipt(hash),
        Effect.map((exit) => TransactionReceiptComplete({ exit })),
      ),

    TransactionReceiptComplete: () => Effect.succeed(ts),
  })

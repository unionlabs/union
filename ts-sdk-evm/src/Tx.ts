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
import type * as EvmWallet from "./EvmWallet.js"

export type TransactionState = Data.TaggedEnum<{
  WriteContractInProgress: {}
  WriteContractComplete: { exit: Effect.Effect.Success<ReturnType<typeof EvmWallet.writeContract>> }
  TransactionReceiptInProgress: { readonly hash: Hash } // on chain hash
  TransactionReceiptComplete: {
    exit: Effect.Effect.Success<ReturnType<typeof Evm.waitForTransactionReceipt>>
  }
}>

export const TransactionState = Data.taggedEnum<TransactionState>()
export const {
  WriteContractInProgress,
  WriteContractComplete,
  TransactionReceiptInProgress,
  TransactionReceiptComplete,
  $is: is,
} = TransactionState

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

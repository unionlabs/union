import {
  EvmSwitchChainError,
  switchChain,
  WaitForTransactionReceiptError,
} from "$lib/services/transfer-ucs03-evm"
import { resolveSafeTx } from "$lib/transfer/shared/services/handlers/safe-hash"
import type { HasKey } from "$lib/types"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"
import {
  ViemPublicClient,
  waitForTransactionReceipt,
  writeContract,
  WriteContractError,
} from "@unionlabs/sdk/evm"
import { Cause, Data, Effect, Match, pipe } from "effect"
import type {
  Abi,
  Chain,
  ContractFunctionArgs,
  ContractFunctionName,
  Hash,
  PublicClient,
  WalletClient,
  WriteContractParameters,
} from "viem"

export type TransactionState = Data.TaggedEnum<{
  Filling: {}
  SwitchChainInProgress: {}
  SwitchChainComplete: { exit: Effect.Effect.Success<ReturnType<typeof switchChain>> }
  WriteContractInProgress: {}
  WriteContractComplete: { exit: Effect.Effect.Success<ReturnType<typeof writeContract>> }
  WaitForSafeWalletHash: { readonly hash: Hash } // the safeTxHash
  TransactionReceiptInProgress: { readonly hash: Hash } // on chain hash
  TransactionReceiptComplete: {
    exit: Effect.Effect.Success<ReturnType<typeof waitForTransactionReceipt>>
  }
}>
type ExitStates = HasKey<TransactionState, "exit">

export const TransactionState = Data.taggedEnum<TransactionState>()
export const {
  SwitchChainInProgress,
  SwitchChainComplete,
  WriteContractInProgress,
  WriteContractComplete,
  WaitForSafeWalletHash,
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
  publicClient: PublicClient,
  walletClient: WalletClient,
  params: WriteContractParameters<TAbi, TFunctionName, TArgs>,
): Effect.Effect<
  TransactionState,
  EvmSwitchChainError | WaitForTransactionReceiptError | WriteContractError,
  never
> =>
  TransactionState.$match(ts, {
    Filling: () => Effect.succeed(SwitchChainInProgress()),

    SwitchChainInProgress: () =>
      Effect.gen(function*() {
        // safe wagmi connector does not support wagmiSwitchChain
        const isSafeWallet = getLastConnectedWalletId() === "safe"

        if (isSafeWallet) {
          return WriteContractInProgress()
        }

        return yield* pipe(
          switchChain(chain),
          Effect.map((exit) => SwitchChainComplete({ exit })),
        )
      }),

    SwitchChainComplete: ({ exit }) => Effect.succeed(WriteContractInProgress()),

    WriteContractInProgress: () =>
      pipe(
        writeContract(walletClient, params),
        Effect.map((exit) =>
          WriteContractComplete({
            exit,
          })
        ),
      ),

    WriteContractComplete: ({ exit: hash }) =>
      Effect.gen(function*() {
        const wallet = getLastConnectedWalletId()

        // needed due to safe wagmi connector returns safeTx hash and not the onchain one
        return wallet === "safe"
          ? WaitForSafeWalletHash({ hash })
          : TransactionReceiptInProgress({ hash })
      }),

    WaitForSafeWalletHash: ({ hash }) =>
      Effect.gen(function*() {
        const resolvedHash = yield* resolveSafeTx(hash) // TODO ???

        return TransactionReceiptInProgress({ hash: resolvedHash })
      }),

    TransactionReceiptInProgress: ({ hash }) =>
      pipe(
        waitForTransactionReceipt(hash),
        Effect.provideService(ViemPublicClient, { client: publicClient }),
        Effect.map((exit) => TransactionReceiptComplete({ exit })),
      ),

    TransactionReceiptComplete: () => Effect.succeed(ts),
  }).pipe(
    Effect.tap((to) =>
      pipe(
        Effect.log("fsm.transition"),
        Effect.annotateLogs({
          from: ts._tag,
          to: to._tag,
          chain: "evm",
        }),
      )
    ),
    Effect.tapErrorCause((cause) =>
      pipe(
        Effect.logError("fsm.transition", cause),
        Effect.annotateLogs({
          from: ts._tag,
          chain: "evm",
        }),
      )
    ),
  )

export const toCtaText = (orElse: string) =>
  pipe(
    Match.type<TransactionState>(),
    Match.tags({
      WriteContractInProgress: () => "Confirming Transaction..." as const,
      SwitchChainInProgress: () => "Switching Chain..." as const,
      TransactionReceiptInProgress: () => "Waiting for Receipt..." as const,
      WaitForSafeWalletHash: () => "Confirming Safe Wallet..." as const,
    }),
    Match.orElse(() => orElse),
  )

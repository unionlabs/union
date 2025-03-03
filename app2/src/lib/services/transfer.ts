import { Data, Effect } from "effect"
import type {
  CreatePublicClientErrorType,
  CreateWalletClientErrorType,
  SendTransactionErrorType,
  SendTransactionParameters,
  SwitchChainErrorType,
  WaitForTransactionReceiptErrorType
} from "viem"
import { createPublicClient, createWalletClient, http } from "viem"
import { sepolia } from "viem/chains"
import { custom, getConnectorClient, switchChain, waitForTransactionReceipt } from "@wagmi/core"
import { wagmiConfig, type ConfiguredChainId } from "$lib/wallet/evm/wagmi-config"

export class CreateWalletClientError extends Data.TaggedError("CreateWalletClientError")<{
  cause: CreateWalletClientErrorType
}> {}

export class WaitForTransactionReceiptError extends Data.TaggedError(
  "WaitForTransactionReceiptError"
)<{
  cause: WaitForTransactionReceiptErrorType
}> {}

export class CreatePublicClientError extends Data.TaggedError("CreatePublicClientError")<{
  cause: CreatePublicClientErrorType
}> {}

export class SendTransactionError extends Data.TaggedError("SendTransactionError")<{
  cause: SendTransactionErrorType
}> {}

export class SwitchChainError extends Data.TaggedError("SwitchChainError")<{
  cause: SwitchChainErrorType
}> {}

export type SubmitTransferError = SendTransactionError | CreateWalletClientError

export const switchToChain = (chainId: ConfiguredChainId) =>
  Effect.tryPromise({
    try: () => switchChain(wagmiConfig, { chainId }),
    catch: err => new SwitchChainError({ cause: err as SwitchChainErrorType })
  })

export const submitTransfer = (transactionArgs: SendTransactionParameters) =>
  Effect.gen(function* () {
    const publicClient = yield* Effect.try({
      try: () =>
        createPublicClient({
          chain: sepolia,
          transport: http()
        }),
      catch: err => new CreatePublicClientError({ cause: err as CreatePublicClientErrorType })
    })

    const walletClient = yield* Effect.try({
      try: () =>
        createWalletClient({
          chain: sepolia,
          transport: () => window.ethereum
        }),
      catch: err => new CreateWalletClientError({ cause: err as CreateWalletClientErrorType })
    })
    const hash = yield* Effect.tryPromise({
      try: () => walletClient.sendTransaction(transactionArgs),
      catch: err => new SendTransactionError({ cause: err as SendTransactionErrorType })
    })

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })

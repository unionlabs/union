import { Data, Effect } from "effect"
import type {
  CreatePublicClientErrorType,
  CreateWalletClientErrorType,
  Hash,
  SendTransactionErrorType,
  SendTransactionParameters,
  SwitchChainErrorType,
  WaitForTransactionReceiptErrorType
} from "viem"
import { createPublicClient, createWalletClient, http } from "viem"
import { sepolia } from "viem/chains"
import { custom, getConnectorClient, switchChain as wagmiSwitchChain } from "@wagmi/core"
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
export class ConnectorClientError extends Data.TaggedError("ConnectorClientError")<{
  cause: Error
}> {}

export type SubmitTransferError = SendTransactionError | CreateWalletClientError

export const switchChain = (chainId: ConfiguredChainId) =>
  Effect.tryPromise({
    try: () => wagmiSwitchChain(wagmiConfig, { chainId }),
    catch: err => new SwitchChainError({ cause: err as SwitchChainErrorType })
  })

export const submitTransfer = (transactionArgs: SendTransactionParameters) =>
  Effect.gen(function* () {
    const connectorClient = yield* Effect.tryPromise({
      try: () => getConnectorClient(wagmiConfig),
      catch: err => new ConnectorClientError({ cause: err as Error })
    })

    const walletClient = yield* Effect.try({
      try: () =>
        createWalletClient({
          chain: sepolia,
          transport: custom(connectorClient.transport)
        }),
      catch: err => new CreateWalletClientError({ cause: err as CreateWalletClientErrorType })
    })

    const hash = yield* Effect.tryPromise({
      try: () => walletClient.sendTransaction(transactionArgs),
      catch: err => new SendTransactionError({ cause: err as SendTransactionErrorType })
    })

    return hash
  })

export const waitForReceipt = (hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* Effect.try({
      try: () =>
        createPublicClient({
          chain: sepolia,
          transport: http()
        }),
      catch: err => new CreatePublicClientError({ cause: err as CreatePublicClientErrorType })
    })

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })

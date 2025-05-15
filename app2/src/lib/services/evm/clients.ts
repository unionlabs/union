import { getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils/extract-error-details.ts"
import { getConnectorClient, type GetConnectorClientErrorType } from "@wagmi/core"
import { Context, Data, Effect, Option } from "effect"
import {
  createPublicClient,
  type CreatePublicClientErrorType,
  createWalletClient,
  type CreateWalletClientErrorType,
  custom,
  http,
  type PublicClient,
} from "viem"
import {
  ConnectorClientError,
  CreatePublicClientError,
  CreateWalletClientError,
} from "../transfer/errors.ts"

export class PublicSourceViemClient extends Context.Tag("PublicSourceViemClient")<
  PublicSourceViemClient,
  { readonly client: PublicClient }
>() {}

// XXX: change tag to NoViemChainError
export class NoViemChainError extends Data.TaggedError("NoViemChain")<{
  chain: Chain
}> {}

export const getWagmiConnectorClient = Effect.tryPromise({
  try: () => getConnectorClient(getWagmiConfig()),
  catch: err =>
    new ConnectorClientError({
      wagmiConfig: getWagmiConfig(),
      cause: extractErrorDetails(err as Error) as GetConnectorClientErrorType,
    }),
})

/**
 * @deprecated use the one from ts-sdk instead
 */
export const getPublicClient = (chain: Chain) =>
  Effect.gen(function*() {
    const viemChain = chain.toViemChain()

    if (Option.isNone(viemChain)) {
      return yield* new NoViemChainError({ chain })
    }

    const client = yield* Effect.try({
      try: () =>
        createPublicClient({
          chain: viemChain.value,
          transport: http(),
        }),
      catch: err =>
        new CreatePublicClientError({
          cause: extractErrorDetails(err as Error) as CreatePublicClientErrorType,
        }),
    })
    return client
  })

/**
 * @deprecated use the one from ts-sdk instead
 */
export const getWalletClient = (chain: Chain) =>
  Effect.gen(function*() {
    const viemChain = chain.toViemChain()

    if (Option.isNone(viemChain)) {
      return yield* new NoViemChainError({ chain })
    }

    const connectorClient = yield* Effect.tryPromise({
      try: () => getConnectorClient(getWagmiConfig()),
      catch: err =>
        new ConnectorClientError({
          wagmiConfig: getWagmiConfig(),
          cause: extractErrorDetails(err as Error) as GetConnectorClientErrorType,
        }),
    })

    return yield* Effect.try({
      try: () =>
        createWalletClient({
          account: connectorClient.account,
          chain: viemChain.value,
          transport: custom(connectorClient.transport),
        }),
      catch: err =>
        new CreateWalletClientError({
          cause: extractErrorDetails(err as Error) as CreateWalletClientErrorType,
        }),
    })
  })

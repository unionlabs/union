import { Context, Data, Effect, Option } from "effect"
import {
  createPublicClient,
  type CreatePublicClientErrorType,
  createWalletClient,
  type CreateWalletClientErrorType,
  custom,
  http,
  type PublicClient
} from "viem"
import { getConnectorClient, type GetConnectorClientErrorType } from "@wagmi/core"
import { wagmiConfig } from "$lib/wallet/evm/wagmi-config"
import {
  ConnectorClientError,
  CreatePublicClientError,
  CreateWalletClientError
} from "../transfer/errors.ts"
import type { Chain } from "@unionlabs/sdk/schema"

export class PublicSourceViemClient extends Context.Tag("PublicSourceViemClient")<
  PublicSourceViemClient,
  { readonly client: PublicClient }
>() {}

export class NoViemChainError extends Data.TaggedError("NoViemChain")<{
  chain: Chain
}> {}

export const getPublicClient = (chain: Chain) =>
  Effect.gen(function* () {
    const viemChain = chain.toViemChain()

    if (Option.isNone(viemChain)) {
      return yield* new NoViemChainError({ chain })
    }

    const client = yield* Effect.try({
      try: () =>
        createPublicClient({
          chain: viemChain.value,
          transport: http()
        }),
      catch: err => new CreatePublicClientError({ cause: err as CreatePublicClientErrorType })
    })
    return client
  })

export const getWalletClient = (chain: Chain) =>
  Effect.gen(function* () {
    const viemChain = chain.toViemChain()

    if (Option.isNone(viemChain)) {
      return yield* new NoViemChainError({ chain })
    }

    const connectorClient = yield* Effect.tryPromise({
      try: () => getConnectorClient(wagmiConfig),
      catch: err => new ConnectorClientError({ cause: err as GetConnectorClientErrorType })
    })

    return yield* Effect.try({
      try: () =>
        createWalletClient({
          account: connectorClient.account,
          chain: viemChain.value,
          transport: custom(connectorClient.transport)
        }),
      catch: err => new CreateWalletClientError({ cause: err as CreateWalletClientErrorType })
    })
  })

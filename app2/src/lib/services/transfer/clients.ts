import { Effect } from "effect"
import {
  createPublicClient,
  createWalletClient,
  http,
  custom,
  type CreatePublicClientErrorType,
  type CreateWalletClientErrorType
} from "viem"
import { sepolia } from "viem/chains"
import { getConnectorClient, type GetConnectorClientErrorType } from "@wagmi/core"
import { wagmiConfig } from "$lib/wallet/evm/wagmi-config"
import { CreatePublicClientError, CreateWalletClientError, ConnectorClientError } from "./errors"

export const getPublicClient = Effect.try({
  try: () =>
    createPublicClient({
      chain: sepolia,
      transport: http()
    }),
  catch: err => new CreatePublicClientError({ cause: err as CreatePublicClientErrorType })
})

export const getWalletClient = Effect.gen(function* () {
  const connectorClient = yield* Effect.tryPromise({
    try: () => getConnectorClient(wagmiConfig),
    catch: err => new ConnectorClientError({ cause: err as GetConnectorClientErrorType })
  })

  return yield* Effect.try({
    try: () =>
      createWalletClient({
        chain: sepolia,
        transport: custom(connectorClient.transport)
      }),
    catch: err => new CreateWalletClientError({ cause: err as CreateWalletClientErrorType })
  })
})

import { getGasPriceForChain } from "$lib/services/cosmos/chain-info"
import type { CosmosWallet } from "$lib/services/cosmos/types.ts"
import {
  CosmosWalletNotConnectedError,
  CosmosWalletNotOnWindowError,
  CosmWasmError,
} from "$lib/services/transfer-ucs03-cosmos"
import { getCosmosOfflineSigner } from "$lib/services/transfer-ucs03-cosmos/offline-signer.ts"
import { cosmosStore, type CosmosWalletId } from "$lib/wallet/cosmos"
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { GasPrice } from "@cosmjs/stargate"
import type { Chain } from "@unionlabs/sdk/schema"
import { Effect, Option } from "effect"

export const getCosmWasmClient = (
  chain: Chain,
  connectedWallet: CosmosWalletId,
) =>
  Effect.gen(function*() {
    if (!chain.rpcs) {
      return yield* new CosmWasmError({ cause: "No RPCs available for chain" })
    }

    const offlineSigner = yield* getCosmosOfflineSigner(chain)
    const gasPriceInfo = yield* getGasPriceForChain(chain, connectedWallet)
    const gasPrice = GasPrice.fromString(
      `${gasPriceInfo.amount}${gasPriceInfo.denom}`,
    )

    const maybeRpcUrl = chain.getRpcUrl("rpc")
    if (Option.isNone(maybeRpcUrl)) {
      return yield* new CosmWasmError({
        cause: "No RPC URL of type 'rpc' available for chain",
      })
    }

    const rpcUrl = maybeRpcUrl.value.toString()

    return yield* Effect.tryPromise({
      try: () =>
        SigningCosmWasmClient.connectWithSigner(rpcUrl, offlineSigner, {
          gasPrice,
        }),
      catch: (err) => new CosmWasmError({ cause: String(err) }),
    })
  })

export const getCosmosPublicClient = (
  rpc: URL | string,
): Effect.Effect<CosmWasmClient, CosmWasmError, never> =>
  Effect.tryPromise({
    try: () => {
      const rpcString = typeof rpc === "string" ? rpc : rpc.toString()
      return CosmWasmClient.connect(rpcString)
    },
    catch: (err) =>
      new CosmWasmError({
        cause: `Failed to create CosmWasm client with RPC ${rpc}: ${
          String(
            err,
          )
        }`,
      }),
  })

export const getCosmosWalletClient = Effect.gen(function*() {
  const { connectedWallet, connectionStatus } = cosmosStore
  if (connectionStatus !== "connected" || !connectedWallet) {
    return yield* new CosmosWalletNotConnectedError({
      cause: "wallet not connected according to cosmosStore",
    })
  }

  const wallet = window[connectedWallet as keyof Window] as CosmosWallet
  if (!wallet) {
    return yield* new CosmosWalletNotOnWindowError({ kind: connectedWallet })
  }

  return wallet
})

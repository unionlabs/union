import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { Effect, Option } from "effect"
import { cosmosStore, type CosmosWalletId } from "$lib/wallet/cosmos"
import type { CosmosWallet } from "$lib/services/cosmos/types.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { getCosmosOfflineSigner } from "$lib/services/transfer-ucs03-cosmos/offline-signer.ts"
import { GasPrice } from "@cosmjs/stargate"
import { getGasPriceForChain } from "$lib/services/cosmos/chain-info"
import { CosmWasmError } from "$lib/services/transfer-ucs03-cosmos"

export const getCosmWasmClient = (chain: Chain, connectedWallet: CosmosWalletId) =>
  Effect.gen(function* () {
    if (!chain.rpcs) {
      return yield* Effect.fail(
        new CosmWasmError({
          cause: "No RPCs available for chain"
        })
      )
    }

    const offlineSigner = yield* Effect.mapError(
      getCosmosOfflineSigner(chain, connectedWallet),
      error =>
        new CosmWasmError({
          cause: String(error.cause) || "Failed to get offline signer"
        })
    )

    if (!offlineSigner) {
      return yield* Effect.fail(new CosmWasmError({ cause: "Offline signer is undefined" }))
    }

    const gasPriceInfo = yield* Effect.mapError(
      getGasPriceForChain(chain, connectedWallet),
      error =>
        new CosmWasmError({
          cause: `Failed to get gas price: ${error.cause}`
        })
    )

    const gasPrice = GasPrice.fromString(`${gasPriceInfo.amount}${gasPriceInfo.denom}`)

    const rpcUrl = chain.getRpcUrl("rpc")
    if (Option.isNone(rpcUrl)) {
      return yield* Effect.fail(
        new CosmWasmError({
          cause: "No RPC URL of type 'rpc' available for chain"
        })
      )
    }

    return yield* Effect.tryPromise({
      try: () =>
        SigningCosmWasmClient.connectWithSigner(
          rpcUrl.value.toString(),
          offlineSigner,
          {
            gasPrice
          }
        ),
      catch: err =>
        new CosmWasmError({
          cause: String(err)
        })
    })
  })

export const getCosmosPublicClient = (rpc: URL | string) =>
  Effect.tryPromise({
    try: () => {
      const rpcString = typeof rpc === "string" ? rpc : rpc.toString()
      return CosmWasmClient.connect(rpcString)
    },
    catch: err =>
      new CosmWasmError({
        cause: `Failed to create CosmWasm client with RPC ${rpc}: ${String(err)}`
      })
  })

export const getCosmosWalletClient = (): Effect.Effect<CosmosWallet, CosmWasmError, never> =>
  Effect.gen(function* () {
    const { connectedWallet, connectionStatus } = cosmosStore
    if (connectionStatus !== "connected" || !connectedWallet) {
      return yield* Effect.fail(new CosmWasmError({ cause: "Wallet not connected" }))
    }

    const wallet = window[connectedWallet as keyof Window] as CosmosWallet
    if (!wallet) {
      return yield* Effect.fail(new CosmWasmError({ cause: `Wallet ${connectedWallet} not found` }))
    }

    return wallet
  })

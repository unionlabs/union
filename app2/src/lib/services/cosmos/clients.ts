import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { Effect, Option } from "effect"
import { cosmosStore, type CosmosWalletId } from "$lib/wallet/cosmos"
import type { CosmosWallet } from "$lib/services/cosmos/types.ts"
import type { Chain } from "$lib/schema/chain.ts"
import { getCosmosOfflineSigner } from "$lib/services/transfer-cosmos/offline-signer.ts"
import { GasPrice } from "@cosmjs/stargate"
import { getGasPriceForChain } from "$lib/services/cosmos/chain-info"
import { CosmWasmError } from "$lib/services/transfer-cosmos"

export const getCosmWasmClient = (chain: Chain, connectedWallet: CosmosWalletId) =>
  Effect.gen(function* () {
    if (!chain.rpcs) {
      throw new CosmWasmError({
        cause: "No RPCs available for chain"
      })
    }

    const offlineSigner = yield* Effect.mapError(
      getCosmosOfflineSigner(chain, connectedWallet),
      error =>
        new CosmWasmError({
          cause: String(error.cause) || "Failed to get offline signer"
        })
    )

    if (!offlineSigner) {
      throw new CosmWasmError({ cause: "Offline signer is undefined" })
    }

    // Get gas price
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
      throw new CosmWasmError({
        cause: "No RPC URL of type 'rpc' available for chain"
      })
    }

    return yield* Effect.tryPromise({
      try: () =>
        SigningCosmWasmClient.connectWithSigner(rpcUrl.value.toString(), offlineSigner, {
          gasPrice
        }),
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
      new Error(`Failed to create CosmWasm client with RPC ${rpc} ${err}`, { cause: err })
  })

export const getCosmosWalletClient = (): Effect.Effect<CosmosWallet, Error, never> =>
  Effect.try({
    try: () => {
      const { connectedWallet, connectionStatus } = cosmosStore
      if (connectionStatus === "connected" && connectedWallet) {
        // Type assertion to help TypeScript understand this will be a cosmos wallet
        const wallet = window[connectedWallet as keyof Window] as CosmosWallet
        if (!wallet) {
          throw new Error(`Wallet ${connectedWallet} not found`)
        }
        return wallet
      }

      throw new Error("Wallet not connected")
    },
    catch: err => new Error(`Failed to get cosmos wallet client`, { cause: err })
  })

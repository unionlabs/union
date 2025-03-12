import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { Effect } from "effect"
import {cosmosStore} from "$lib/wallet/cosmos";

export const getCosmWasmClient = (rpc: URL | string) =>
  Effect.tryPromise({
    try: () => {
      const rpcString = typeof rpc === "string" ? rpc : rpc.toString()
      return CosmWasmClient.connect(rpcString)
    },
    catch: err =>
      new Error(`Failed to create CosmWasm client with RPC ${rpc} ${err}`, { cause: err })
  })


//Move into schema?
type CosmosWallet = Window['keplr'] | Window['leap']

export const getWalletClient = (): Effect.Effect<CosmosWallet, Error, never> =>
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
    catch: (err) =>
      new Error(`Failed to get cosmos wallet client`, { cause: err })
  })

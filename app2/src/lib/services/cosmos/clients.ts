import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { Effect } from "effect"
import {cosmosStore, type CosmosWalletId} from "$lib/wallet/cosmos";
import type {CosmosWallet} from "$lib/services/cosmos/types.ts";
import type {Chain} from "$lib/schema/chain.ts";
import {getCosmosOfflineSigner} from "$lib/services/transfer-cosmos/offline-signer.ts";
import {GasPrice, SigningStargateClient} from "@cosmjs/stargate";
import {StargateClientError} from "$lib/services/transfer-cosmos";
import {getGasPriceForChain} from "$lib/services/cosmos/chain-info";

export const getCosmosClient = (
  chain: Chain,
  connectedWallet: CosmosWalletId
) =>
  Effect.gen(function* () {
    if (!chain.rpcs) {
      yield* Effect.fail(new StargateClientError({ cause: "No rpcs" }))
      return
    }

    const offlineSigner = yield* getCosmosOfflineSigner(chain, connectedWallet)

    if (!offlineSigner) {
      yield* Effect.fail(new StargateClientError({ cause: "Offline signer is undefined" }))
      return
    }

    const gasPriceInfo = yield* Effect.mapError(
      getGasPriceForChain(chain, connectedWallet),
      error => new StargateClientError({ cause: `Failed to get gas price: ${error.cause}` })
    )

    const gasPrice = GasPrice.fromString(`${gasPriceInfo.amount}${gasPriceInfo.denom}`)

    return yield* Effect.tryPromise({
      try: () => SigningStargateClient.connectWithSigner(
        chain.getRpcUrl('rpc').toString(),
        offlineSigner,
        { gasPrice }
      ),
      catch: err => new StargateClientError({ cause: String(err) })
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
    catch: (err) =>
      new Error(`Failed to get cosmos wallet client`, { cause: err })
  })
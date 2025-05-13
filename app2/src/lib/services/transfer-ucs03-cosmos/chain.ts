import { getCosmosWalletClient, getCosmWasmClient } from "$lib/services/cosmos/clients.ts"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { cosmosStore } from "$lib/wallet/cosmos/index.ts"
import type { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import type { Chain } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils/extract-error-details.ts"
import { Effect } from "effect"
import {
  CosmosSwitchChainError,
  CosmosWalletNotConnectedError,
  NoCosmosChainInfoError,
} from "./errors.ts"

type SwitchChainSuccess = {
  success: true
  chainId: string
  signingClient: SigningCosmWasmClient
}

export const switchChain = (chain: Chain) =>
  Effect.gen(function*() {
    // TODO: make pure and DRY
    const { connectedWallet, connectionStatus } = cosmosStore
    if (connectionStatus !== "connected" || !connectedWallet) {
      return yield* new CosmosWalletNotConnectedError({
        // TODO: move to `message`
        cause: "wallet not connected according to cosmosStore",
      })
    }
    // END TODO

    const wallet = yield* getCosmosWalletClient

    const chainInfo = getCosmosChainInfo(chain.chain_id)

    if (!chainInfo) {
      return yield* new NoCosmosChainInfoError({ chain })
    }

    yield* Effect.tryPromise({
      try: () => wallet.experimentalSuggestChain(chainInfo),
      catch: err =>
        new CosmosSwitchChainError({
          cause: extractErrorDetails(err as Error),
          chainId: chain.universal_chain_id,
          phase: "suggest",
          chainInfo,
        }),
    })

    yield* Effect.tryPromise({
      try: () => wallet.enable([chain.chain_id]),
      catch: err =>
        new CosmosSwitchChainError({
          cause: extractErrorDetails(err as Error),
          chainId: chain.universal_chain_id,
          phase: "enable",
          chainInfo,
        }),
    })

    yield* Effect.sleep("1.5 seconds")

    const signingClient = yield* getCosmWasmClient(chain, connectedWallet)

    return yield* Effect.succeed<SwitchChainSuccess>({
      success: true,
      chainId: chain.chain_id,
      signingClient,
    })
  })

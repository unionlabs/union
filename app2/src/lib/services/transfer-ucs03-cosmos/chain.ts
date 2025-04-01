import { Effect } from "effect"
import { SwitchChainError } from "./errors.ts"
import { getCosmosWalletClient } from "$lib/services/cosmos/clients.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"

export const switchChain = (chain: Chain) =>
  Effect.gen(function* () {
    const wallet = yield* getCosmosWalletClient()

    if (!wallet) {
      return Effect.fail(new SwitchChainError({ cause: "Wallet client is undefined" }))
    }

    if (!chain.chain_id) {
      return Effect.fail(new SwitchChainError({ cause: "Invalid chain ID" }))
    }

    const chainInfo = getCosmosChainInfo(chain.chain_id)
    if (!chainInfo) {
      return Effect.fail(
        new SwitchChainError({ cause: `Chain info not found for ${chain.chain_id}` })
      )
    }

    yield* Effect.tryPromise({
      try: () => wallet.experimentalSuggestChain(chainInfo),
      catch: err => new SwitchChainError({ cause: `Failed to switch chain: ${String(err)}` })
    })

    yield* Effect.tryPromise({
      try: () => wallet.enable([chain.chain_id]),
      catch: err => new SwitchChainError({ cause: `Failed to enable chain: ${String(err)}` })
    })

    // Not sure if this is needed here, but we do it for evm
    yield* Effect.sleep("1.5 seconds")

    return { success: true, chainId: chain.chain_id }
  })

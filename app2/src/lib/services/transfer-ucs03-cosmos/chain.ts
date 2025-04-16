import { Effect, Match } from "effect"
import { type CosmWasmError, SwitchChainError } from "./errors.ts"
import { getCosmosWalletClient } from "$lib/services/cosmos/clients.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { MODE } from "$lib/constants/config.ts"

type SwitchChainSuccess = {
  success: true
  chainId: string
}

export const switchChain = (
  chain: Chain
): Effect.Effect<SwitchChainSuccess, SwitchChainError | CosmWasmError> =>
  Effect.gen(function* () {
    const wallet = yield* getCosmosWalletClient()

    if (!wallet) {
      return yield* Effect.fail(new SwitchChainError({ cause: "Wallet client is undefined" }))
    }

    if (!chain.chain_id) {
      return yield* Effect.fail(new SwitchChainError({ cause: "Invalid chain ID" }))
    }

    yield* Match.value(MODE).pipe(
      Match.when(
        mode => mode === "testnet" || mode === "mainnet",
        () =>
          Effect.gen(function* () {
            const chainInfo = getCosmosChainInfo(chain.chain_id)

            if (!chainInfo) {
              return yield* Effect.fail(
                new SwitchChainError({ cause: `Chain info not found for ${chain.chain_id}` })
              )
            }

            yield* Effect.tryPromise({
              try: () => wallet.experimentalSuggestChain(chainInfo),
              catch: err =>
                new SwitchChainError({ cause: `Failed to switch chain: ${String(err)}` })
            })

            yield* Effect.tryPromise({
              try: () => wallet.enable([chain.chain_id]),
              catch: err =>
                new SwitchChainError({ cause: `Failed to enable chain: ${String(err)}` })
            })
          })
      ),
      Match.orElse(() => Effect.fail(new SwitchChainError({ cause: "Invalid mode" })))
    )

    yield* Effect.sleep("1.5 seconds")

    return yield* Effect.succeed<SwitchChainSuccess>({ success: true, chainId: chain.chain_id })
  })

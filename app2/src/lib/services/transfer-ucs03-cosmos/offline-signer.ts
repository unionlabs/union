import { Effect } from "effect"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import {
  type CosmWasmError,
  OfflineSignerError
} from "$lib/services/transfer-ucs03-cosmos/errors.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import type { OfflineSigner } from "$lib/services/cosmos/types.ts"
import { getCosmosWalletClient } from "$lib/services/cosmos/clients.ts"

export const getCosmosOfflineSigner: (
  chain: Chain,
  connectedWallet: CosmosWalletId
) => Effect.Effect<OfflineSigner, OfflineSignerError | CosmWasmError, never> = (
  chain,
  connectedWallet
) =>
  Effect.gen(function* () {
    if (!connectedWallet) {
      return yield* new OfflineSignerError({ cause: "No wallet connected" })
    }

    if (!chain?.chain_id) {
      return yield* new OfflineSignerError({ cause: "Invalid chain: missing chain_id" })
    }

    const wallet = yield* getCosmosWalletClient()

    if (!wallet) {
      return yield* new OfflineSignerError({ cause: `Could not get wallet` })
    }

    const tryCallSigner = Effect.tryPromise({
      try: () =>
        wallet.getOfflineSignerAuto(chain.chain_id, {
          disableBalanceCheck: false
        }),
      catch: cause => {
        console.error("[getCosmosOfflineSigner]", cause)
        return new OfflineSignerError({ cause })
      }
    })

    return yield* tryCallSigner
  })

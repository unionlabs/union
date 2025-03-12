import { Effect } from "effect"
import { type CosmosWalletId } from "$lib/wallet/cosmos"
import { OfflineSignerError } from "$lib/services/transfer-cosmos/errors.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { OfflineSigner } from "$lib/services/cosmos/types.ts"

/**
 * Gets an offline signer for the given chain and wallet
 */
export const getCosmosOfflineSigner = (chain: Chain, connectedWallet: CosmosWalletId) =>
  Effect.gen(function* () {
    if (!connectedWallet) {
      yield* Effect.fail(new OfflineSignerError({ cause: "No wallet connected" }))
      return
    }

    if (!chain?.chain_id) {
      yield* Effect.fail(new OfflineSignerError({ cause: "Invalid chain: missing chain_id" }))
      return
    }

    const wallet = window[connectedWallet]
    if (!wallet) {
      yield* Effect.fail(new OfflineSignerError({ cause: `Wallet ${connectedWallet} not found in window object` }))
      return
    }

    const signerMethod = wallet.getOfflineSignerAuto
    if (!signerMethod) {
      yield* Effect.fail(new OfflineSignerError({ cause: `Wallet ${connectedWallet} does not support getOfflineSignerAuto` }))
      return
    }

    return yield* Effect.tryPromise({
      try: async () => {
        const signerResult = signerMethod.call(wallet, chain.chain_id, {
          disableBalanceCheck: false
        })

        const signer = await signerResult

        if (!signer) {
          throw new Error(`Failed to get offline signer for ${connectedWallet}`)
        }

        return signer as unknown as OfflineSigner
      },
      catch: (err) => new OfflineSignerError({ cause: String(err) })
    })
  })
import { Effect } from "effect"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import { OfflineSignerError } from "$lib/services/transfer-ucs03-cosmos/errors.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { OfflineSigner } from "$lib/services/cosmos/types.ts"
import { getCosmosWalletClient } from "$lib/services/cosmos/clients.ts"

export const getCosmosOfflineSigner = (chain: Chain, connectedWallet: CosmosWalletId) => {
  if (!connectedWallet) {
    return Effect.fail(new OfflineSignerError({ cause: "No wallet connected" }))
  }

  if (!chain?.chain_id) {
    return Effect.fail(new OfflineSignerError({ cause: "Invalid chain: missing chain_id" }))
  }

  return Effect.flatMap(getCosmosWalletClient(), wallet => {
    if (!wallet) {
      return Effect.fail(new OfflineSignerError({ cause: `Could not get wallet` }))
    }

    const signerMethod = wallet.getOfflineSignerAuto
    if (!signerMethod) {
      return Effect.fail(
        new OfflineSignerError({
          cause: `Wallet ${connectedWallet} does not support getOfflineSignerAuto`
        })
      )
    }

    return Effect.tryPromise({
      try: async () => {
        const signerResult = signerMethod.call(wallet, chain.chain_id, {
          disableBalanceCheck: false
        })

        const signer = await signerResult

        if (!signer) {
          return Promise.reject(
            new OfflineSignerError({
              cause: `Failed to get offline signer for ${connectedWallet}`
            })
          )
        }

        return signer as unknown as OfflineSigner
      },
      catch: err => new OfflineSignerError({ cause: String(err) })
    })
  })
}
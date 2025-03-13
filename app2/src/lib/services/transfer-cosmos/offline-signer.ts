import { Effect } from "effect"
import type { CosmosWalletId } from "$lib/wallet/cosmos"
import { OfflineSignerError } from "$lib/services/transfer-cosmos/errors.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { OfflineSigner } from "$lib/services/cosmos/types.ts"
import { getCosmosWalletClient } from "$lib/services/cosmos/clients.ts"

export const getCosmosOfflineSigner = (chain: Chain, connectedWallet: CosmosWalletId) =>
  Effect.gen(function* () {
    if (!connectedWallet) {
      throw new OfflineSignerError({ cause: "No wallet connected" })
    }

    if (!chain?.chain_id) {
      throw new OfflineSignerError({ cause: "Invalid chain: missing chain_id" })
    }

    const wallet = yield* getCosmosWalletClient()

    if (!wallet) {
      throw new OfflineSignerError({ cause: `Could not get wallet` })
    }

    const signerMethod = wallet.getOfflineSignerAuto
    if (!signerMethod) {
      throw new OfflineSignerError({
        cause: `Wallet ${connectedWallet} does not support getOfflineSignerAuto`
      })
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
      catch: err => new OfflineSignerError({ cause: String(err) })
    })
  })

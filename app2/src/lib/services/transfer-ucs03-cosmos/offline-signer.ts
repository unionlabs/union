import { Effect } from "effect"
import { OfflineSignerError } from "$lib/services/transfer-ucs03-cosmos/errors.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import type { OfflineSigner } from "$lib/services/cosmos/types.ts"
import { getCosmosWalletClient } from "$lib/services/cosmos/clients.ts"
import { extractErrorDetails } from "@unionlabs/sdk/utils"

export const getCosmosOfflineSigner = (chain: Chain) =>
  Effect.gen(function* () {
    const wallet = yield* getCosmosWalletClient

    return yield* Effect.tryPromise({
      try: (): Promise<OfflineSigner> =>
        wallet.getOfflineSignerAuto(chain.chain_id, {
          disableBalanceCheck: false
        }),
      catch: err => new OfflineSignerError({ cause: extractErrorDetails(err as Error) })
    })
  })

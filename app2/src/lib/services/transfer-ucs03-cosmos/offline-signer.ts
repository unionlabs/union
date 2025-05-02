import { getCosmosWalletClient } from "$lib/services/cosmos/clients.ts"
import type { OfflineSigner } from "$lib/services/cosmos/types.ts"
import { OfflineSignerError } from "$lib/services/transfer-ucs03-cosmos/errors.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Effect } from "effect"

export const getCosmosOfflineSigner = (chain: Chain) =>
  Effect.gen(function*() {
    const wallet = yield* getCosmosWalletClient

    return yield* Effect.tryPromise({
      try: (): Promise<OfflineSigner> =>
        wallet.getOfflineSignerAuto(chain.chain_id, {
          disableBalanceCheck: false,
        }),
      catch: err =>
        new OfflineSignerError({
          cause: extractErrorDetails(err as Error),
          chain_id: chain.chain_id,
        }),
    })
  })

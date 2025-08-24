import { getCosmosWalletClient } from "$lib/services/cosmos/clients"
import type { OfflineSigner } from "$lib/services/cosmos/types"
import { OfflineSignerError } from "$lib/services/transfer-ucs03-cosmos/errors"
import type { Chain } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
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

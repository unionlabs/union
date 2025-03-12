import { Effect } from "effect"
import {cosmosStore, getCosmosOfflineSigner as getSigner} from "$lib/wallet/cosmos";
import {OfflineSignerError} from "$lib/services/transfer-cosmos/errors.ts";

export const getCosmosOfflineSigner = ({ chainId}: {
  connectedWallet: string
  chainId: string
}) =>
  Effect.tryPromise({
    try: async () => {

      const { connectedWallet } = cosmosStore

      if (!connectedWallet) {
        throw new OfflineSignerError({cause: "No wallet connected"})
      }

      if (!chainId) {
        throw new OfflineSignerError({cause: "Chain ID is required"})
      }

      return await getSigner({
        connectedWallet,
        chainId
      })
    },
    catch: (err) => new OfflineSignerError({ cause: String(err) })
  })
import { get } from "svelte/store"
import type { LayoutLoad } from "./$types.ts"
import { sepoliaStore } from "$lib/wallet/evm/config.ts"
import { cosmosStore } from "$lib/wallet/cosmos/config.ts"

export const load = (loadEvent => {
  const addressParam = loadEvent.url.searchParams.get("address")?.toLowerCase()
  if (!addressParam) {
    return {
      addressArray: [
        get(sepoliaStore).address?.toLowerCase(),
        get(cosmosStore).address?.toLowerCase()
      ]
    }
  }

  const addressArray = addressParam?.includes("-") ? addressParam.split("-") : [addressParam]
  return { addressArray }
}) satisfies LayoutLoad

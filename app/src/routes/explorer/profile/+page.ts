import { get } from "svelte/store"
import type { PageLoad } from "../../src/routes/explorer/profile/$types.ts"
import { sepoliaStore } from "$lib/wallet/evm/config.ts"
import { cosmosStore } from "$lib/wallet/cosmos/config.ts"

export const load = (loadEvent => ({
  connectedAddressArray: [
    //
    get(sepoliaStore).address,
    get(cosmosStore).address
  ].filter(Boolean)
})) satisfies PageLoad

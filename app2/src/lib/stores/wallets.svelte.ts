import { Option } from "effect"
import type { AddressNormalized } from "$lib/schema/address"

class WalletsStore {
  evmAddress: Option.Option<typeof AddressNormalized.Type> = $state(Option.none())
  cosmosAddress: Option.Option<typeof AddressNormalized.Type> = $state(Option.none())
  aptosAddress: Option.Option<typeof AddressNormalized.Type> = $state(Option.none())
}

export const wallets = new WalletsStore()

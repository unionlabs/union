import { Option } from "effect"
import type {
  AddressEvmCanonical,
  AddressAptosCanonical,
  AddressCosmosCanonical
} from "$lib/schema/address"

class WalletsStore {
  evmAddress: Option.Option<typeof AddressEvmCanonical.Type> = $state(Option.none())
  cosmosAddress: Option.Option<typeof AddressCosmosCanonical.Type> = $state(Option.none())
  aptosAddress: Option.Option<typeof AddressAptosCanonical.Type> = $state(Option.none())
}

export const wallets = new WalletsStore()

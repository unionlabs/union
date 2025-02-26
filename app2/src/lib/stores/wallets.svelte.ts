import { Option } from "effect"
import {
  AddressEvmCanonical,
  type AddressAptosCanonical,
  type AddressCosmosCanonical
} from "$lib/schema/address"

class WalletsStore {
  evmAddress: Option.Option<typeof AddressEvmCanonical.Type> = $state(
    Option.some(AddressEvmCanonical.make("0xe6831e169d77a861a0e71326afa6d80bcc8bc6aa"))
  )
  cosmosAddress: Option.Option<typeof AddressCosmosCanonical.Type> = $state(Option.none())
  aptosAddress: Option.Option<typeof AddressAptosCanonical.Type> = $state(Option.none())
}

export const wallets = new WalletsStore()

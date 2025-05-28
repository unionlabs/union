import type {
  AddressAptosCanonical,
  AddressCanonicalBytes,
  AddressCosmosCanonical,
  AddressEvmCanonical,
  Chain,
} from "@unionlabs/sdk/schema"
import { Match, Option } from "effect"

class WalletsStore {
  evmAddress: Option.Option<typeof AddressEvmCanonical.Type> = $state(Option.none())
  cosmosAddress: Option.Option<typeof AddressCosmosCanonical.Type> = $state(Option.none())
  aptosAddress: Option.Option<typeof AddressAptosCanonical.Type> = $state(Option.none())
  inputAddress: Option.Option<typeof AddressCanonicalBytes.Type> = $state(Option.none())

  hasAnyWallet() {
    return (
      Option.isSome(this.evmAddress)
      || Option.isSome(this.cosmosAddress)
      || Option.isSome(this.aptosAddress)
      || Option.isSome(this.inputAddress)
    )
  }

  addInputAddress(address: typeof AddressCosmosCanonical.Type) {
    this.inputAddress = Option.some(address)
  }

  clearInputAddress() {
    this.inputAddress = Option.none()
  }

  getCanonicalByteAddressList() {
    const addresses: Array<typeof AddressCanonicalBytes.Type> = []
    if (Option.isSome(this.evmAddress)) {
      addresses.push(this.evmAddress.value)
    }
    if (Option.isSome(this.cosmosAddress)) {
      addresses.push(this.cosmosAddress.value)
    }
    if (Option.isSome(this.aptosAddress)) {
      addresses.push(this.aptosAddress.value)
    }
    return addresses
  }

  getAddressForChain(chain: Chain): Option.Option<AddressCanonicalBytes> {
    return Match.value(chain.rpc_type).pipe(
      Match.when("evm", () => this.evmAddress),
      Match.when("cosmos", () => this.cosmosAddress),
      Match.when("aptos", () => this.aptosAddress),
      Match.exhaustive,
    )
  }
}

export const wallets = new WalletsStore()

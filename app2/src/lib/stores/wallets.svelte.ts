import { Option } from "effect"
import type {
  AddressAptosCanonical,
  AddressCanonicalBytes,
  AddressCosmosCanonical,
  AddressEvmCanonical,
  Chain
} from "@unionlabs/sdk/schema"

class WalletsStore {
  evmAddress: Option.Option<typeof AddressEvmCanonical.Type> = $state(Option.none())
  cosmosAddress: Option.Option<typeof AddressCosmosCanonical.Type> = $state(Option.none())
  aptosAddress: Option.Option<typeof AddressAptosCanonical.Type> = $state(Option.none())
  inputAddress: Option.Option<typeof AddressCanonicalBytes> = $state(Option.none())

  hasAnyWallet() {
    return (
      Option.isSome(this.evmAddress) ||
      Option.isSome(this.cosmosAddress) ||
      Option.isSome(this.aptosAddress) ||
      Option.isSome(this.inputAddress)
    )
  }

  addInputAddress(address: typeof AddressCanonicalBytes.Type) {
    this.inputAddress = Option.some(address)
  }

  clearInputAddress() {
    this.inputAddress = Option.none()
  }

  getCanonicalByteAddressList() {
    const addresses: Array<typeof AddressCanonicalBytes.Type> = []
    if (Option.isSome(this.evmAddress)) addresses.push(this.evmAddress.value)
    if (Option.isSome(this.cosmosAddress)) addresses.push(this.cosmosAddress.value)
    if (Option.isSome(this.aptosAddress)) addresses.push(this.aptosAddress.value)
    return addresses
  }

  getAddressForChain(chain: Chain): Option.Option<AddressCanonicalBytes> {
    if (Option.isSome(this.inputAddress)) {
      return this.inputAddress
    }
    switch (chain.rpc_type) {
      case "cosmos":
        return this.cosmosAddress
      case "evm":
        return this.evmAddress
      case "aptos":
        return this.aptosAddress
      default:
        return Option.none()
    }
  }
}

export const wallets = new WalletsStore()

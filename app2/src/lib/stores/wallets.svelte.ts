import { Option } from "effect"
import type {
  AddressEvmCanonical,
  AddressAptosCanonical,
  AddressCosmosCanonical,
  AddressCanonicalBytes
} from "@unionlabs/sdk/schema"
import type { Chain } from "@unionlabs/sdk/schema"

class WalletsStore {
  evmAddress: Option.Option<typeof AddressEvmCanonical.Type> = $state(Option.none())
  cosmosAddress: Option.Option<typeof AddressCosmosCanonical.Type> = $state(Option.none())
  aptosAddress: Option.Option<typeof AddressAptosCanonical.Type> = $state(Option.none())

  hasAnyWallet() {
    return (
      Option.isSome(this.evmAddress) ||
      Option.isSome(this.cosmosAddress) ||
      Option.isSome(this.aptosAddress)
    )
  }

  getCanonicalByteAddressList() {
    const addresses: Array<typeof AddressCanonicalBytes.Type> = []
    if (Option.isSome(this.evmAddress)) addresses.push(this.evmAddress.value)
    if (Option.isSome(this.cosmosAddress)) addresses.push(this.cosmosAddress.value)
    if (Option.isSome(this.aptosAddress)) addresses.push(this.aptosAddress.value)
    return addresses
  }

  getAddressForChain(chain: Chain): Option.Option<AddressCanonicalBytes> {
    return chain.rpc_type === "cosmos"
      ? this.cosmosAddress
      : chain.rpc_type === "evm"
        ? this.evmAddress
        : chain.rpc_type === "aptos"
          ? this.aptosAddress
          : Option.none()
  }
}

export const wallets = new WalletsStore()

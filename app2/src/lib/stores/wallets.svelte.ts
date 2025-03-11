import { Option } from "effect"
import type {
  AddressEvmCanonical,
  AddressAptosCanonical,
  AddressCosmosCanonical,
  AddressCanonicalBytes
} from "$lib/schema/address"
import type { RpcType } from "$lib/schema/chain.ts"

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

  getAddressForChain(rpcType: typeof RpcType.Type): Option.Option<AddressCanonicalBytes> {
    switch (rpcType) {
      case "evm":
        return Option.map(this.evmAddress, addr => addr as AddressCanonicalBytes)
      case "cosmos":
        return Option.map(this.cosmosAddress, addr => addr as AddressCanonicalBytes)
      case "aptos":
        return Option.map(this.aptosAddress, addr => addr as AddressCanonicalBytes)
      default:
        return Option.none()
    }
  }
}

export const wallets = new WalletsStore()

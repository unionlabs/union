import type { Chain, UserAddresses } from "$lib/types"
import { bech32ToBech32Address } from "@unionlabs/client"

export const userAddrOnChain = (userAddr: UserAddresses, chain?: Chain): string | null => {
  if (!chain) return null

  if (chain.rpc_type === "cosmos") {
    if (userAddr.cosmos?.bytes) {
      return bech32ToBech32Address({
        toPrefix: chain.addr_prefix,
        address: userAddr.cosmos.canonical
      })
    }
    console.log("userAddrOnChain got no cosmos address")
    return null
  }

  if (userAddr.evm?.canonical) {
    return userAddr.evm.canonical
  }
  console.log("userAddrOnChain got no evm address")
  return null
}

export const createCosmosSdkAddressRegex = ({ prefix }: { prefix: string }) =>
  new RegExp(`^${prefix}[a-z0-9]{39}$`)

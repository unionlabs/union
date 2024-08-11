import { bytesToBech32Address } from "@union/client"
import type { Chain, UserAddresses } from "$lib/types"

export const userAddrOnChain = (userAddr: UserAddresses, chain: Chain | null): string | null => {
  if (!chain) {
    return null
  }

  if (chain.rpc_type === "cosmos") {
    if (userAddr.cosmos?.bytes) {
      return bytesToBech32Address({ bytes: userAddr.cosmos.bytes, toPrefix: chain.addr_prefix })
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

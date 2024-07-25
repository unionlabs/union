import { bech32 } from "bech32"
import type { Chain, UserAddresses } from "$lib/types"

export function convertCosmosAddress({ address, toPrefix }: { address: string; toPrefix: string }) {
  const words = bech32.decode(address).words
  return bech32.encode(toPrefix, words)
}

export const rawToHex = (raw: Uint8Array): string =>
  `${Array.from(raw)
    .map(i => i.toString(16).padStart(2, "0"))
    .join("")
    .toLowerCase()}`

export const rawToBech32 = (prefix: string, raw: Uint8Array): string => {
  const words = bech32.toWords(raw)
  return bech32.encode(prefix, words)
}

export const userAddrOnChain = (userAddr: UserAddresses, chain: Chain | null): string | null => {
  if (!chain) {
    return null
  }

  if (chain.rpc_type === "cosmos") {
    if (userAddr.cosmos?.bytes) {
      return rawToBech32(chain.addr_prefix, userAddr.cosmos.bytes)
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

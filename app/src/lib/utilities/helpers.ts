import type { Chain } from "$lib/types.ts"

export const getSupportedAsset = (chain: Chain, identifier: string) => {
  for (const supportedAsset of chain.assets) {
    if (supportedAsset.denom === identifier) {
      if (checkWhitelisted()) return supportedAsset
    }
  }

  return undefined
}

//Implement the list for checks
export const checkWhitelisted = () => {
  return true
}

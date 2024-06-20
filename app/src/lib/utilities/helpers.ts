import type { Chain } from "$lib/types.ts"

export const getSupportedAsset = (chain: Chain, denom: any) => {
  for (const supportedAsset of chain.assets) {
    if (supportedAsset.denom === denom) {
      if (checkWhitelisted()) return supportedAsset
    }
  }

  return undefined
}

//Implement the list for checks
export const checkWhitelisted = () => {
  return true
}

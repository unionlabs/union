import type { Chain } from "$lib/types.ts"

export const getSupportedAsset = (chain: Chain, identifier: string) => {
  for (const supportedAsset of chain.assets) {
    if (supportedAsset.denom === identifier) {
      return supportedAsset
    }
  }

  return undefined
}

import type { Chain } from "$lib/types.ts"

export const findAsset = (chain: Chain, denom: any) => {
  for (const supportedAsset of chain.assets) {
    if (supportedAsset.denom === denom) {
      return supportedAsset
    }
  }
  return undefined
}

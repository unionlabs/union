import type { Chain } from "$lib/types.ts"

export const findAsset = (chain: Chain, denom: string) => {
  for (const asset of chain.assets) {
    if (asset.denom === denom) {
      return asset
    }
  }
  return undefined
}

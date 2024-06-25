import type { Chain } from "$lib/types.ts"

export const getSupportedAsset = (chain: Chain, identifier: string) => {
  for (const supportedAsset of chain.assets) {
    if (supportedAsset.denom === identifier) {
      return supportedAsset
    }
  }

  return undefined
}


export function zip<S1, S2>(firstCollection: Array<S1>, lastCollection: Array<S2>): Array<[S1, S2]> {
    const length = Math.min(firstCollection.length, lastCollection.length);
    const zipped: Array<[S1, S2]> = [];

    for (let index = 0; index < length; index++) {
        zipped.push([firstCollection[index], lastCollection[index]]);
    }

    return zipped;
}

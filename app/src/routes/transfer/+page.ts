import type { PageLoad } from "./$types.ts"
import { chainMap, chainIds } from "$lib/constants/chains.ts"

interface ILoadData {
  chainIds: typeof chainIds
  chainMap: typeof chainMap
}

export const load = (_context => ({
  chainIds,
  chainMap
})) satisfies PageLoad<ILoadData>

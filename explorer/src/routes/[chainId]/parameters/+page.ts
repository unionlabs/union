import { createChainRuntime } from "$lib/runtime"
import { fetchAllParams } from "$lib/queries/params"
import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ parent }) => {
  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  return {
    params: runtime.runPromise(fetchAllParams()),
    chain,
  }
}

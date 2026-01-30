import { CHAINS, DEFAULT_CHAIN } from "$lib/chains/config"
import { redirect } from "@sveltejs/kit"
import type { LayoutLoad } from "./$types"

export const load: LayoutLoad = ({ params }) => {
  const { chainId } = params

  // chainId in URL is the universal_chain_id (e.g., "union.union-1")
  const chain = CHAINS[chainId]

  if (!chain) {
    // If invalid chain, redirect to default
    redirect(307, `/${DEFAULT_CHAIN}`)
  }

  return {
    chainId: chain.universal_chain_id,
    chainName: chain.chain_name,
    chain,
  }
}

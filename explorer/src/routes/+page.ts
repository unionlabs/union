import { redirect } from "@sveltejs/kit"
import { CHAINS, DEFAULT_CHAIN } from "$lib/chains/config"
import type { PageLoad } from "./$types"

export const load: PageLoad = async () => {
  // Redirect root to default chain
  redirect(307, `/${CHAINS[DEFAULT_CHAIN].chain_id}`)
}

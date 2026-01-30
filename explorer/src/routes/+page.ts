import { DEFAULT_CHAIN } from "$lib/chains/config"
import { redirect } from "@sveltejs/kit"
import type { PageLoad } from "./$types"

export const load: PageLoad = async () => {
  // Redirect root to default chain (use universal_chain_id for URL routing)
  redirect(307, `/${DEFAULT_CHAIN}`)
}

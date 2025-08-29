import { requireAuth } from "$lib/dashboard/auth-guard"
import type { LayoutLoad } from "./$types"

export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

export const load: LayoutLoad = async ({ url }) => {
  return await requireAuth(url, "/auth/sign-in", ["/auth"])
}

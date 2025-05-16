import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { redirect } from "@sveltejs/kit"
import { Option } from "effect"
import type { LayoutLoad } from "./$types"

export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

export const load: LayoutLoad = async () => {
  if (Option.isNone(dashboard.session)) {
    throw redirect(302, "/")
  }

  return {
    session: dashboard.session,
  }
}

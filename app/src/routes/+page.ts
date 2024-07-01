import { redirect } from "@sveltejs/kit"
import type { PageLoad } from "./$types.ts"

export const load = (() => {
  redirect(302, "/explorer")
}) satisfies PageLoad

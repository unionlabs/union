import { checkAuth, type SessionError } from "$lib/utils/auth.ts"
import type { LayoutLoad } from "./$types.ts"
import { redirect } from "@sveltejs/kit"


export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

export const load: LayoutLoad = async ({ url }) => {
  const pathname = url.pathname

  if (pathname) {
    const segments = pathname.split("/").filter(Boolean)
    if (segments[0] === "app") {
      const authCheck = await checkAuth()
      return authCheck.match(
        () => {
          return {}
        },
        (error: SessionError) => {
          console.error(error.message)
          throw redirect(302, "/auth/register")
        }
      )
    }
  }

  return {}
}

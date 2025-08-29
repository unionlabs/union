import { browser } from "$app/environment"
import { runPromise } from "$lib/runtime"
import type { Session } from "@supabase/supabase-js"
import { redirect } from "@sveltejs/kit"
import { Effect, pipe } from "effect"

export interface AuthGuardResult {
  session?: Session | null
  isAuthenticated: boolean
}

/**
 * Generic authentication guard for SvelteKit load functions
 *
 * @param url - The current URL from the load function
 * @param redirectTo - Where to redirect if not authenticated
 * @param excludePaths - Paths that should be excluded from auth check
 * @param status - HTTP status code for redirect (default: 302)
 *
 * @example
 * ```ts
 * // In +layout.ts
 * export const load: LayoutLoad = async ({ url }) => {
 *   return await requireAuth(url, '/udrop/check', ['/udrop/check'])
 * }
 *
 * // With custom status code
 * export const load: LayoutLoad = async ({ url }) => {
 *   return await requireAuth(url, '/login', [], 301)
 * }
 * ```
 */
export async function requireAuth(
  url: URL,
  redirectTo: string,
  excludePaths: string[] = [],
  status: number = 302,
): Promise<AuthGuardResult> {
  // Only run in browser
  if (!browser) {
    return { isAuthenticated: false }
  }

  // Check if current path should be excluded from auth check
  const isExcludedPath = excludePaths.some(path => url.pathname.includes(path))

  const sessionResult = await runPromise(
    pipe(
      Effect.promise(() => import("$lib/dashboard/client")),
      Effect.flatMap(({ SupabaseClient }) =>
        pipe(
          SupabaseClient,
          Effect.flatMap((client) =>
            Effect.tryPromise({
              try: async () => {
                const { data, error } = await client.auth.getSession()
                if (error) {
                  throw error
                }
                return data.session
              },
              catch: (cause) => cause,
            })
          ),
        )
      ),
      Effect.catchAll(() => Effect.succeed(null)),
    ),
  )

  const isAuthenticated = !!sessionResult

  // Handle redirects outside of Effect
  if (!isAuthenticated && !isExcludedPath) {
    throw redirect(status, redirectTo)
  }

  return {
    session: sessionResult,
    isAuthenticated,
  }
}

import type { Reroute } from "@sveltejs/kit"

/**
 * `hooks.{js|ts}` is a special SvelteKit file called "Universal Hooks"
 * @see https://kit.svelte.dev/docs/hooks#universal-hooks
 */

export const reroute = (event => {
  const url = new URL(event.url)
  const pathname = url.pathname

  if (pathname === "/") return "/explorer/transfers"
}) satisfies Reroute

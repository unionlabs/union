import type { HandleClientError } from "@sveltejs/kit"

/**
 * `hooks.client.{js|ts}` is a special SvelteKit file called "Shared Hooks"
 * @see https://kit.svelte.dev/docs/hooks#shared-hooks
 */

// biome-ignore lint/suspicious/useAwait: no need
export const handleError = (async ({ error, event, status, message, ...context }) => {
  const errorId = crypto.randomUUID()

  return { errorId, message: `${message} - ${error}` }
}) satisfies HandleClientError

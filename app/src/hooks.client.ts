import type { HandleClientError } from "@sveltejs/kit"

// biome-ignore lint/suspicious/useAwait: no need
export const handleError = (async ({ error, event, status, message, ...context }) => {
  const errorId = crypto.randomUUID()

  return { errorId, message: `${message} - ${error}` }
}) satisfies HandleClientError

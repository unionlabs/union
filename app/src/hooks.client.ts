import type { HandleClientError } from "@sveltejs/kit"

export const handleError = (context => {
  console.warn(JSON.stringify(context, undefined, 2))
  const errorId = crypto.randomUUID()

  return { errorId, message: `${context.message} - ${context.error}` }
}) satisfies HandleClientError

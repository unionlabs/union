import { generateUserErrorMessage, sendSvelteErrorLog } from "$lib/utils/error"
import type { HandleClientError } from "@sveltejs/kit"

export const handleError = (async ({ error, event, message, status }) => {
  const errorId = await sendSvelteErrorLog({ error, event, message, status }, "client")
  return {
    message: generateUserErrorMessage(errorId)
  }
}) satisfies HandleClientError

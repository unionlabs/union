import * as Sentry from "@sentry/sveltekit"
import type { HandleClientError } from "@sveltejs/kit"

Sentry.init({
  enabled: import.meta.env.MODE === "production",
  dsn: "https://b410cea864cbfaefea5fc8b18e40ae4f@o4506911891783680.ingest.us.sentry.io/4507500708954112",
  tracesSampleRate: 1,
  replaysOnErrorSampleRate: 1,
  replaysSessionSampleRate: 0.1,
  integrations: [
    Sentry.replayIntegration(),
    Sentry.breadcrumbsIntegration(),
    Sentry.extraErrorDataIntegration()
  ]
})

// biome-ignore lint/suspicious/useAwait: no need
export const handleError = (async ({ error, event, status, message, ...context }) => {
  const errorId = crypto.randomUUID()

  if (import.meta.env.MODE === "production") {
    Sentry.captureException(error, {
      extra: { event, errorId, status, message }
    })
  }

  return { errorId, message: `${message} - ${error}` }
}) satisfies HandleClientError

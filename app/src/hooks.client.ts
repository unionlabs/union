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
    Sentry.extraErrorDataIntegration(),
    Sentry.feedbackIntegration({
      showBranding: false,
      colorScheme: "system",
      submitButtonLabel: "Submit",
      formTitle: "Feedback / Bug Report",
      buttonLabel: "Feedback / Bug Report",
      messagePlaceholder:
        "Please describe the issue you encountered or provide feedback on how we can improve."
    })
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

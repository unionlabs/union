import { browser } from "$app/environment"
import { axiom } from "$lib/utils/axiom.ts"

const browserErrorSessionId = browser ? crypto.randomUUID() : undefined

export const generateUserErrorMessage = (errorId: string) => {
  return `An unexpected error occurred, please try again! Error ID: "${errorId}"`
}

export const sendSvelteErrorLog = async (svelteError: SvelteError, type: "client") => {
  return await sendErrorLog({ svelte: svelteError }, type)
}

export const sendWindowErrorLog = async (event: Event) => {
  const origin = event.target instanceof Window ? event.target.origin : undefined
  const location = event.target instanceof Window ? event.target.location.href : undefined
  const message = event instanceof ErrorEvent ? event.message : undefined
  const stack = event instanceof ErrorEvent ? event.error?.stack : undefined
  const windowEventError = {
    origin,
    location,
    message,
    stack,
    type: event.type,
    sessionId: browserErrorSessionId
  }
  return await sendErrorLog({ window: windowEventError }, "window")
}

export const sendWindowRejectionLog = async (event: PromiseRejectionEvent) => {
  const origin = event.target instanceof Window ? event.target.origin : undefined
  const location = event.target instanceof Window ? event.target.location.href : undefined
  const windowEventError = {
    origin,
    location,
    message: event.reason?.message,
    stack: event.reason?.stack,
    type: event.type,
    sessionId: browserErrorSessionId
  }
  return await sendErrorLog({ window: windowEventError }, "window")
}

const sendErrorLog = (detail: ErrorDetail, type: "client" | "window") => {
  const errorId = crypto.randomUUID()
  axiom.ingest("errors", [{ errorId, type, detail }, errorJsonReplacer])
  return errorId
}

const errorJsonReplacer = (key: string, value: unknown) => {
  try {
    if (key === "error" && !!value && typeof value === "object") {
      const sanitizedError: { [k: string]: unknown } = {}
      const errorValue = value as Record<string, unknown>
      for (const propertyName of Object.getOwnPropertyNames(errorValue)) {
        sanitizedError[propertyName] = errorValue[propertyName]
      }
      return sanitizedError
    }
  } catch (e) {
    console.error("Failed to sanitize error", e)
  }
  return value
}

type SvelteError = {
  error: unknown
  event: unknown
  message: string
  status: number
}

type WindowEventError = {
  origin: string | undefined
  location: string | undefined
  message: unknown | undefined
  stack: unknown | undefined
  type: string
  sessionId: string | undefined
}

type ErrorDetail = {
  svelte?: SvelteError
  window?: WindowEventError
}

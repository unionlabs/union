export function extractErrorDetails<T extends Error>(
  error: T,
  withOwnProperties: boolean | undefined = true
): {
  [K in keyof T]: T[K]
} & {
  message: string
  name: string
  stack?: string
  cause?: unknown
} {
  const extractedError = {} as {
    [K in keyof T]: T[K]
  } & {
    message: string
    name: string
    stack?: string
    cause?: unknown
  }

  // Extract all own properties, including non-enumerable ones
  if (withOwnProperties) {
    Object.getOwnPropertyNames(error).forEach(key => {
      extractedError[key as keyof T] = error[key as keyof T]
    })
  }

  // Explicitly copy inherited properties
  extractedError.message = error.message
  extractedError.name = error.name
  if (error.stack) extractedError.stack = error.stack
  if ("cause" in error) extractedError.cause = error.cause

  return extractedError
}

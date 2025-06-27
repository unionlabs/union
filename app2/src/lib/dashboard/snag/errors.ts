import { Data } from "effect"

export type BaseErrorArgs = {
  cause: unknown
  operation: string
  message?: string
}

export class SnagClientError extends Data.TaggedError("SnagClientError")<BaseErrorArgs> {}

export class SnagAPIError extends Data.TaggedError("SnagAPIError")<
  BaseErrorArgs & {
    status?: number
    headers?: Record<string, string>
  }
> {}

export class SnagBadRequestError extends Data.TaggedError("SnagBadRequestError")<
  BaseErrorArgs & {
    status: 400
    headers?: Record<string, string>
  }
> {}

export class SnagAuthenticationError extends Data.TaggedError("SnagAuthenticationError")<
  BaseErrorArgs & {
    status: 401
    headers?: Record<string, string>
  }
> {}

export class SnagPermissionDeniedError extends Data.TaggedError("SnagPermissionDeniedError")<
  BaseErrorArgs & {
    status: 403
    headers?: Record<string, string>
  }
> {}

export class SnagNotFoundError extends Data.TaggedError("SnagNotFoundError")<
  BaseErrorArgs & {
    status: 404
    headers?: Record<string, string>
  }
> {}

export class SnagUnprocessableEntityError extends Data.TaggedError("SnagUnprocessableEntityError")<
  BaseErrorArgs & {
    status: 422
    headers?: Record<string, string>
  }
> {}

export class SnagRateLimitError extends Data.TaggedError("SnagRateLimitError")<
  BaseErrorArgs & {
    status: 429
    headers?: Record<string, string>
  }
> {}

export class SnagInternalServerError extends Data.TaggedError("SnagInternalServerError")<
  BaseErrorArgs & {
    status: number
    headers?: Record<string, string>
  }
> {}

export class SnagConnectionError extends Data.TaggedError("SnagConnectionError")<BaseErrorArgs> {}

export function mapSnagError(error: unknown, operation: string) {
  // Check if it's a Snag SDK APIError here we need to allow null afaik lol
  if (typeof error === "object" && error !== null && "name" in error && typeof (error as {name: unknown}).name === "string") {
    const apiError = error as {
      name: string
      status?: number
      headers?: Record<string, string>
      message?: string
    }
    
    const status = apiError.status || 0
    const headers = apiError.headers || {}
    const message = apiError.message || "Unknown Snag API error"
    
    switch (apiError.name) {
      case "BadRequestError":
        return new SnagBadRequestError({
          operation,
          cause: error,
          message: `Bad Request: ${message}`,
          status: 400,
          headers,
        })
      case "AuthenticationError":
        return new SnagAuthenticationError({
          operation,
          cause: error,
          message: `Authentication Failed: ${message}`,
          status: 401,
          headers,
        })
      case "PermissionDeniedError":
        return new SnagPermissionDeniedError({
          operation,
          cause: error,
          message: `Permission Denied: ${message}`,
          status: 403,
          headers,
        })
      case "NotFoundError":
        return new SnagNotFoundError({
          operation,
          cause: error,
          message: `Not Found: ${message}`,
          status: 404,
          headers,
        })
      case "UnprocessableEntityError":
        return new SnagUnprocessableEntityError({
          operation,
          cause: error,
          message: `Unprocessable Entity: ${message}`,
          status: 422,
          headers,
        })
      case "RateLimitError":
        return new SnagRateLimitError({
          operation,
          cause: error,
          message: `Rate Limit Exceeded: ${message}`,
          status: 429,
          headers,
        })
      case "InternalServerError":
        return new SnagInternalServerError({
          operation,
          cause: error,
          message: `Internal Server Error: ${message}`,
          status: status >= 500 ? status : 500,
          headers,
        })
      case "APIConnectionError":
        return new SnagConnectionError({
          operation,
          cause: error,
          message: `Connection Error: ${message}`,
        })
      default:
        // For any other API error
        return new SnagAPIError({
          operation,
          cause: error,
          message: `Snag API Error: ${message}`,
          status,
          headers,
        })
    }
  }

  // For non-API errors (network, timeout, etc.)
  const errorMessage = typeof error === "object" && error !== null && "message" in error && typeof (error as {message: unknown}).message === "string" 
    ? (error as {message: string}).message 
    : "Unknown connection error"
    
  return new SnagConnectionError({
    operation,
    cause: error,
    message: errorMessage,
  })
} 
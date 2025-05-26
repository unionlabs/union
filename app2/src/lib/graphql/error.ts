import { Schema as S } from "effect"
import type { ClientError } from "graphql-request"

export class GraphQLError extends S.TaggedError<GraphQLError>("GraphQLError")("GraphQLError", {
  message: S.String,
  status: S.Number,
  errors: S.optional(S.Any),
  cause: S.Any,
}) {
  constructor(error: ClientError) {
    const query = error.request.query
    const variables = error.request.variables
    super({
      message: error.message,
      status: error.response.status,
      errors: error.response.errors,
      cause: error,
    })
  }
}

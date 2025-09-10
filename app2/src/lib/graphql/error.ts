import { Match, pipe, Schema as S } from "effect"
import { ClientError } from "graphql-request"

export class GraphQLError extends S.TaggedError<GraphQLError>("GraphQLError")("GraphQLError", {
  message: S.String,
  status: S.Number,
  errors: S.optional(S.Any),
  cause: S.Any,
}) {
  static fromUnknown(error: unknown) {
    return pipe(
      Match.value(error),
      Match.when(Match.instanceOf(ClientError), this.fromClientError),
      Match.orElse((error) =>
        this.make({
          cause: error,
          message: String(error) ?? "Unknown error",
          status: -1,
        })
      ),
    )
  }
  static fromClientError(error: ClientError) {
    return this.make({
      message: error.message,
      status: error.response.status,
      errors: error.response.errors,
      cause: error,
    })
  }
}

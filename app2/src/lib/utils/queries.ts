import type { GraphQLError } from "$lib/graphql/error"
import { GraphQL, GraphQLRequest } from "$lib/graphql/service"
import type { Persistence } from "@effect/experimental"
import { FetchHttpClient, HttpClient } from "@effect/platform"
import type { HttpClientError } from "@effect/platform/HttpClientError"
import { Effect, Option, pipe, Schedule, Schema } from "effect"
import type { TimeoutException, UnknownException } from "effect/Cause"
import type { DurationInput } from "effect/Duration"
import type { ParseError } from "effect/ParseResult"
import type { TadaDocumentNode } from "gql.tada"
import { type Variables } from "graphql-request"

export type FetchDecodeError = HttpClientError | ParseError | TimeoutException

/**
 * @deprecated Migrate to `@unionlabs/sdk` query functions.
 */
export const fetchDecode = <S>(schema: Schema.Schema<S>, url: string) =>
  Effect.gen(function*() {
    const client = yield* HttpClient.HttpClient
    const response = yield* client.get(url)
    const json = yield* response.json
    return yield* Schema.decodeUnknown(schema)(json)
  }).pipe(
    Effect.tap(Effect.log("request.http")),
    Effect.tapErrorCause((cause) => Effect.logError("request.http", cause)),
    Effect.annotateLogs({
      url,
    }),
    Effect.withLogSpan("fetchDecode"),
  )

export type FetchDecodeGraphqlError = GraphQLError | Persistence.PersistenceError | ParseError
/**
 * TODO: Adjust calling convention to be `GraphQL` dependency injeciton.
 * @deprecated Migrate to `@unionlabs/sdk` query functions.
 */
export const fetchDecodeGraphql = <S, E, D, V extends Variables = Variables>(
  schema: Schema.Schema<S, E>,
  document: TadaDocumentNode<D, V>,
  variables?: V,
): Effect.Effect<S, FetchDecodeGraphqlError, GraphQL> =>
  Effect.andThen(
    GraphQL,
    ({ fetch }) =>
      pipe(
        fetch(new GraphQLRequest({ document, variables })),
        Effect.flatMap(Schema.decodeUnknown(schema)),
      ),
  )

export const createQuery = <S>({
  url,
  schema,
  refetchInterval,
  writeData,
  writeError,
}: {
  url: string
  schema: Schema.Schema<S>
  refetchInterval: DurationInput
  writeData: (data: Option.Option<S>) => void
  writeError: (error: Option.Option<FetchDecodeError>) => void
}) => {
  const fetcherPipeline = pipe(
    fetchDecode(schema, url).pipe(Effect.retry({ times: 4 }), Effect.timeout("10 seconds")),
    Effect.tapBoth({
      onSuccess: data =>
        Effect.sync(() => {
          writeData(Option.some(data))
          writeError(Option.none())
        }),
      onFailure: error =>
        Effect.sync(() => {
          writeError(Option.some(error))
        }),
    }),
    Effect.catchAll(_ => Effect.succeed(null)),
    Effect.scoped,
    Effect.provide(FetchHttpClient.layer),
  )

  const program = Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => refetchInterval),
  )
  return program
}

export const createQueryGraphql = <S, E, D, V extends Variables = Variables>({
  schema,
  document,
  variables,
  refetchInterval,
  writeData,
  writeError,
}: {
  schema: Schema.Schema<S, E>
  document: TadaDocumentNode<D, V>
  variables: V
  refetchInterval: DurationInput
  writeData: (data: Option.Option<S>) => void
  writeError: (error: Option.Option<FetchDecodeGraphqlError | TimeoutException>) => void
}) => {
  const fetcherPipeline = pipe(
    fetchDecodeGraphql(schema, document, variables).pipe(
      Effect.retry({ times: 4 }),
      Effect.timeout("10 seconds"),
    ),
    Effect.tapBoth({
      onSuccess: data =>
        Effect.sync(() => {
          writeData(Option.some(data))
          writeError(Option.none())
        }),
      onFailure: error =>
        Effect.sync(() => {
          writeError(Option.some(error))
        }),
    }),
    Effect.catchAll(_ => Effect.succeed(null)),
    Effect.scoped,
    Effect.provide(FetchHttpClient.layer),
  )

  const program = Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => refetchInterval),
  )
  return program
}

import { URLS } from "$lib/constants"
import { FetchHttpClient, HttpClient } from "@effect/platform"
import type { HttpClientError } from "@effect/platform/HttpClientError"
import { operationNamesFromDocumentNode } from "@unionlabs/sdk/utils"
import { Array as A, Effect, Option, pipe, Schedule, Schema } from "effect"
import type { TimeoutException, UnknownException } from "effect/Cause"
import type { DurationInput } from "effect/Duration"
import type { ParseError } from "effect/ParseResult"
import type { TadaDocumentNode } from "gql.tada"
import { request } from "graphql-request"

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

export type FetchDecodeGraphqlError = UnknownException | ParseError
/**
 * @deprecated Migrate to `@unionlabs/sdk` query functions.
 */
export const fetchDecodeGraphql = <S, E, D, V extends object | undefined>(
  schema: Schema.Schema<S, E>,
  document: TadaDocumentNode<D, V>,
  variables?: V,
): Effect.Effect<S, FetchDecodeGraphqlError, never> => {
  const operationName = pipe(
    document,
    operationNamesFromDocumentNode,
    A.head,
    Option.getOrElse(() => "unknown"),
  )
  const message = `request.gql.${operationName}`
  return pipe(
    Effect.tryPromise(() => request(URLS().GRAPHQL, document, variables)),
    Effect.withSpan("fetch"),
    Effect.flatMap(Schema.decodeUnknown(schema)),
    Effect.withSpan("decode"),
    Effect.tap(Effect.log(message)),
    Effect.tapErrorCause((cause) => Effect.logError(message, cause)),
    Effect.annotateLogs({
      operationName,
      variables,
    }),
    Effect.withLogSpan("fetchDecodeGraphql"),
  )
}

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

export const createQueryGraphql = <S, E, D, V extends object | undefined>({
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

import { Effect, Option, pipe, Schedule, Schema } from "effect"
import { FetchHttpClient, HttpClient } from "@effect/platform"
import type { DurationInput } from "effect/Duration"
import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { ParseError } from "effect/ParseResult"
import type { TadaDocumentNode } from "gql.tada"
import { request } from "graphql-request"
import { URLS } from "$lib/constants"
import type { TimeoutException, UnknownException } from "effect/Cause"

export type FetchDecodeError = HttpClientError | ParseError | TimeoutException

export const fetchDecode = <S>(schema: Schema.Schema<S>, url: string) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const response = yield* client.get(url)
    const json = yield* response.json
    return yield* Schema.decodeUnknown(schema)(json)
  })

export type FetchDecodeGraphqlError = UnknownException | ParseError | TimeoutException

export const fetchDecodeGraphql = <S, E, D, V extends object | undefined>(
  schema: Schema.Schema<S, E>,
  document: TadaDocumentNode<D, V>,
  variables?: V
) =>
  Effect.gen(function* () {
    const data = yield* Effect.tryPromise(() => request(URLS().GRAPHQL, document, variables))
    return yield* Schema.decodeUnknown(schema)(data)
  })

export const createQuery = <S>({
  url,
  schema,
  refetchInterval,
  writeData,
  writeError
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
        })
    }),
    Effect.catchAll(_ => Effect.succeed(null)),
    Effect.scoped,
    Effect.provide(FetchHttpClient.layer)
  )

  const program = Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => refetchInterval)
  )
  return program
}

export const createQueryGraphql = <S, E, D, V extends object | undefined>({
  schema,
  document,
  variables,
  refetchInterval,
  writeData,
  writeError
}: {
  schema: Schema.Schema<S, E>
  document: TadaDocumentNode<D, V>
  variables: V
  refetchInterval: DurationInput
  writeData: (data: Option.Option<S>) => void
  writeError: (error: Option.Option<FetchDecodeGraphqlError>) => void
}) => {
  const fetcherPipeline = pipe(
    fetchDecodeGraphql(schema, document, variables).pipe(
      Effect.retry({ times: 4 }),
      Effect.timeout("10 seconds")
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
        })
    }),
    Effect.catchAll(_ => Effect.succeed(null)),
    Effect.scoped,
    Effect.provide(FetchHttpClient.layer)
  )

  const program = Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => refetchInterval)
  )
  return program
}

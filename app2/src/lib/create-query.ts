import { Option, Effect, Schema, pipe, Random, Schedule } from "effect"
import { FetchHttpClient, HttpClient } from "@effect/platform"
import type { DurationInput } from "effect/Duration"
import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { ParseError } from "effect/ParseResult"

export const fetchDecode = <S>(schema: Schema.Schema<S>, url: string) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const response = yield* client.get(url)
    const json = yield* response.json
    return yield* Schema.decodeUnknown(schema)(json)
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
  writeError: (error: Option.Option<ParseError | HttpClientError>) => void
}) => {
  const fetcherPipeline = pipe(
    fetchDecode(schema, url),
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

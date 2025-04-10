import { Effect, Schema } from "effect"
import { HttpClient } from "@effect/platform"
// import type { DurationInput } from "effect/Duration"
// import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { ParseError } from "effect/ParseResult"
import type { TadaDocumentNode } from "gql.tada"
import { request } from "graphql-request"
import { GRAPHQL_URL } from "../constants/graphql.js"
import type { TimeoutException, UnknownException } from "effect/Cause"

// Deprecated, use the one from ts-sdk
export const fetchDecode = <S>(schema: Schema.Schema<S>, url: string) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const response = yield* client.get(url)
    const json = yield* response.json
    return yield* Schema.decodeUnknown(schema)(json)
  })

export type FetchDecodeGraphqlError = UnknownException | ParseError | TimeoutException

// Deprecated, use the one from ts-sdk
export const fetchDecodeGraphql = <S, E, D, V extends object | undefined>(
  schema: Schema.Schema<S, E>,
  document: TadaDocumentNode<D, V>,
  variables?: V
) =>
  Effect.gen(function* () {
    const data = yield* Effect.tryPromise(() => request(GRAPHQL_URL, document, variables))
    return yield* Schema.decodeUnknown(schema)(data)
  })

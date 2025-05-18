import { HttpClient } from "@effect/platform"
import { Array as A, Effect, Option as O, pipe, Schema } from "effect"
import type { TimeoutException, UnknownException } from "effect/Cause"
import type { ParseError } from "effect/ParseResult"
import type { TadaDocumentNode } from "gql.tada"
import { request } from "graphql-request"
import { GRAPHQL_URL } from "../constants/graphql.js"
import { operationNamesFromDocumentNode } from "./gql.js"

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
export const fetchDecodeGraphql = <S, E, D, V extends object | undefined>(
  schema: Schema.Schema<S, E>,
  document: TadaDocumentNode<D, V>,
  variables?: V,
): Effect.Effect<S, FetchDecodeGraphqlError, never> => {
  const operationName = pipe(
    document,
    operationNamesFromDocumentNode,
    A.head,
    O.getOrElse(() => "unknown"),
  )
  const message = `request.gql.${operationName}`
  return pipe(
    Effect.tryPromise(() => request(GRAPHQL_URL, document, variables)),
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

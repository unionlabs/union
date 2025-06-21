import { FetchHttpClient, HttpClient, HttpClientResponse, Url } from "@effect/platform"
import { Effect, flow, Match, pipe, RequestResolver, Struct } from "effect"
import { constTrue, identity } from "effect/Function"
import * as O from "effect/Option"
import * as S from "effect/Schema"
import * as staking from "./schema/staking.js"
import * as Validators from "./schema/validators.js"
import { flattenObject } from "./utils/flattenObject.js"

// TODO: fix  for transforms
const omitIfNullable = <A>(
  self: S.Schema<A, A, never>,
) =>
  S.optionalToOptional(
    S.NullishOr(self),
    self,
    { decode: O.flatMap(O.fromNullable), encode: O.flatMap(O.fromNullable) },
  )

const GetValidatorsResponse = S.Array(Validators.ValidatorWithImage)
type GetValidatorsResponse = typeof GetValidatorsResponse.Type

const GetPaginatedValidatorsResponse = S.Struct({
  validators: S.Array(Validators.ValidatorWithImage),
  pagination: S.Struct({
    next_key: S.NullOr(S.String),
    total: S.NullOr(S.NumberFromString),
  }),
})
type GetPaginatedValidatorsResponse = typeof GetPaginatedValidatorsResponse.Type

const GetValidatorsError = S.Struct({
  code: S.Int,
  message: S.String,
  details: S.Array(S.Any),
})
type GetValidatorsError = typeof GetValidatorsError.Type

export class GetValidators
  extends S.TaggedRequest<GetValidators>("GetValidators")("GetValidators", {
    failure: GetValidatorsError,
    success: GetPaginatedValidatorsResponse,
    payload: {
      status: S.optional(Validators.Status),
      pagination: S.optional(S.Struct({
        key: omitIfNullable(S.String),
        offset: omitIfNullable(S.Int),
        limit: omitIfNullable(S.Int),
        countTotal: omitIfNullable(S.Boolean),
        reverse: omitIfNullable(S.Boolean),
      })),
    },
  })
{}

export class GetParams extends S.TaggedRequest<GetParams>("GetParams")("GetParams", {
  failure: S.Any,
  success: staking.Params,
  payload: {},
}) {}

export class Staking extends Effect.Service<Staking>()("@unionlabs/sdk/Staking", {
  effect: Effect.gen(function*() {
    const client = yield* pipe(
      HttpClient.HttpClient,
      Effect.map(HttpClient.withTracerDisabledWhen(constTrue)),
      Effect.map(HttpClient.filterStatusOk),
    )

    // TODO: move into context
    const base = "https://api.rpc-node.union-testnet-10.union.build"

    const maxPageLimit = 30

    // const SearchDadJokesResolver = RequestResolver.fromEffect((request: SearchDadJokes) => {
    //   const getLimit = (total: number) => Math.min(request.limit - total, maxPageLimit)

    //   const maybeNextPage = (
    //     page: number,
    //     total: number,
    //     response: DadJokeSearchResponse,
    //   ): Option.Option<[page: number, total: number]> => {
    //     const newTotal = total + response.results.length
    //     // If the new total satisifes the requested limit or if there are not
    //     // enough jokes to satisfy the requested count, break out of pagination
    //     return newTotal >= request.limit || newTotal >= response.total_jokes
    //       ? Option.none()
    //       : Option.some([page + 1, newTotal])
    //   }

    //   const baseHttpRequest = Http.request.get("/search").pipe(
    //     Http.request.appendUrlParam("term", request.term),
    //   )
    //   return Stream.paginateChunkEffect(
    //     [0, 0] as [page: number, total: number],
    //     ([page, total]) => {
    //       const httpRequest = baseHttpRequest.pipe(
    //         Http.request.appendUrlParams({
    //           limit: String(getLimit(total)),
    //           page: String(page),
    //         }),
    //       )
    //       return client(httpRequest).pipe(
    //         Effect.flatMap(decodeDadJokeSearch),
    //         Effect.catchTags({
    //           ParseError: (error) => new DadJokeError({ message: error.toString() }),
    //           RequestError: (error) => new DadJokeError({ message: String(error.error) }),
    //           ResponseError: (error) => new DadJokeError({ message: String(error.error) }),
    //         }),
    //         Effect.map((response) => [
    //           Chunk.unsafeFromArray(response.results),
    //           maybeNextPage(page, total, response),
    //         ]),
    //       )
    //     },
    //   ).pipe(
    //     Stream.runCollect,
    //     Effect.map(Chunk.toReadonlyArray),
    //     Effect.scoped,
    //   )
    // })
    const streamResolver = RequestResolver.fromEffect((request: GetValidators) => {
      const getLimit = (total: number) =>
        Math.min(request.pagination?.limit ?? 0 - total, maxPageLimit)

      const maybeNextPage = (
        page: number,
        total: number,
        response: GetPaginatedValidatorsResponse,
      ): O.Option<[page: number, total: number]> => {
        const newTotal = total + response.validators.length
        // If the new total satisifes the requested limit or if there are not
        // enough jokes to satisfy the requested count, break out of pagination
        return newTotal >= (request.pagination?.limit ?? 0)
            || newTotal >= (response.pagination.total ?? 0)
          ? O.none()
          : O.some([page + 1, newTotal])
      }
    })

    const resolver = RequestResolver.fromEffectTagged<GetValidators | GetParams>()({
      GetValidators: Effect.forEach((req) =>
        pipe(
          Url.fromString(
            "/cosmos/staking/v1beta1/validators",
            base,
          ),
          Effect.flatMap((u) =>
            client.get(u, {
              acceptJson: true,
              urlParams: pipe(
                Struct.omit(req, "_tag"),
                flattenObject,
              ),
            })
          ),
          Effect.flatMap(HttpClientResponse.schemaBodyJson(S.Struct({
            validators: S.Array(Validators.ValidatorWithImage),
          }))),
          Effect.provide(FetchHttpClient.layer),
          Effect.map(Struct.get("validators")),
          Effect.mapError(flow(
            Match.value,
            Match.tagsExhaustive({
              IllegalArgumentException: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
              ParseError: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
              RequestError: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
              ResponseError: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
            }),
          )),
        )
      ),
      GetParams: Effect.forEach((_req) =>
        pipe(
          Url.fromString(
            "/cosmos/staking/v1beta1/params",
            base,
          ),
          Effect.flatMap((u) => client.get(u, { acceptJson: true })),
          Effect.flatMap(HttpClientResponse.schemaBodyJson(S.Struct({
            params: staking.Params,
          }))),
          Effect.map(Struct.get("params")),
          Effect.mapError(flow(
            Match.value,
            Match.tagsExhaustive({
              IllegalArgumentException: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
              ParseError: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
              RequestError: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
              ResponseError: (e) => ({
                code: 0,
                message: e.message,
                details: [],
              }),
            }),
          )),
        )
      ),
    })

    const getValidators = Effect.fn("getValidators")(
      (req: GetValidators) => Effect.request(req, resolver),
    )

    const getParams = Effect.fn("getParams")(
      (req: GetParams) => Effect.request(req, resolver),
    )

    return {
      getValidators,
      getParams,
    } as const
  }),
  dependencies: [FetchHttpClient.layer],
}) {}

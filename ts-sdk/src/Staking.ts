import { FetchHttpClient, HttpClient, HttpClientResponse, Url } from "@effect/platform"
import { Effect, flow, Match, pipe, RequestResolver, Struct } from "effect"
import { constTrue } from "effect/Function"
import * as S from "effect/Schema"
import * as Validators from "./schema/validators.js"

export class GetValidators
  extends S.TaggedRequest<GetValidators>("GetValidators")("GetValidators", {
    failure: S.Struct({
      code: S.Int,
      message: S.String,
      details: S.Array(S.Any),
    }),
    success: S.Array(Validators.Validator),
    payload: {
      status: Validators.Status,
      pagination: S.partial(S.Struct({
        key: S.String,
        offset: S.Int,
        limit: S.Int,
        countTotal: S.Boolean,
        reverse: S.Boolean,
      })),
    },
  })
{}

export class Staking extends Effect.Service<Staking>()("Staking", {
  effect: Effect.gen(function*() {
    const client = yield* pipe(
      HttpClient.HttpClient,
      Effect.map(HttpClient.withTracerDisabledWhen(constTrue)),
      Effect.map(HttpClient.filterStatusOk),
    )

    // TODO: move into context
    const base = "https://api.rpc-node.union-testnet-10.union.build"

    const resolver = RequestResolver.fromEffectTagged<GetValidators>()({
      GetValidators: (reqs) => {
        return Effect.forEach(
          reqs,
          // TODO: add query parameters
          (_req) =>
            pipe(
              Url.fromString(
                "/cosmos/staking/v1beta1/validators",
                base,
              ),
              Effect.flatMap((u) => client.get(u, { acceptJson: true })),
              Effect.flatMap(HttpClientResponse.schemaBodyJson(S.Struct({
                validators: S.Array(Validators.Validator),
              }))),
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
            ),
        )
      },
    })

    const getValidators = Effect.fn("getValidators")(
      (req: GetValidators) => Effect.request(req, resolver),
    )

    return {
      getValidators,
    } as const
  }),
  dependencies: [FetchHttpClient.layer],
}) {}

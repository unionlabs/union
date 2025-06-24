import { FetchHttpClient, HttpClient, HttpClientResponse, Url } from "@effect/platform"
import { Effect, flow, Match, pipe, RequestResolver, Struct } from "effect"
import { constTrue } from "effect/Function"
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

const GetValidatorsError = S.Struct({
  code: S.Int,
  message: S.String,
  details: S.Array(S.Any),
})
type GetValidatorsError = typeof GetValidatorsError.Type

export class GetValidators
  extends S.TaggedRequest<GetValidators>("GetValidators")("GetValidators", {
    failure: GetValidatorsError,
    success: GetValidatorsResponse,
    payload: {
      status: omitIfNullable(Validators.Status),
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

class GetSpendableBalancesOfAddress
  extends S.TaggedRequest<GetSpendableBalancesOfAddress>("GetSpendableBalancesOfAddress")(
    "GetSpendableBalancesOfAddress",
    {
      failure: S.Any,
      success: S.Struct({
        balances: S.Array(S.Struct({
          denom: S.String,
          amount: S.String,
        })),
      }),
      payload: {
        address: S.String,
      },
    },
  )
{}

// curl -s "$REST/cosmos/bank/v1beta1/spendable_balances/$ADDR" \
//      | jq --arg DENOM "$BOND_DENOM" '.balances[] | select(.denom==$DENOM) | .amount'

export class Staking extends Effect.Service<Staking>()("@unionlabs/sdk/Staking", {
  effect: Effect.gen(function*() {
    const client = yield* pipe(
      HttpClient.HttpClient,
      Effect.map(HttpClient.withTracerDisabledWhen(constTrue)),
      Effect.map(HttpClient.filterStatusOk),
    )

    // TODO: move into context
    const base = "https://api.rpc-node.union-testnet-10.union.build"

    const resolver = RequestResolver.fromEffectTagged<
      GetValidators | GetParams | GetSpendableBalancesOfAddress
    >()({
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
      GetSpendableBalancesOfAddress: Effect.forEach((req) =>
        pipe(
          Url.fromString(
            `/cosmos/bank/v1beta1/spendable_balances/${req.address}`,
            base,
          ),
          Effect.flatMap((u) => client.get(u, { acceptJson: true })),
          Effect.flatMap(HttpClientResponse.schemaBodyJson(GetSpendableBalancesOfAddress.success)),
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

    const getSpendableBalancesOfAddress = Effect.fn("getSpendableBalancesOfAddress")(
      (req: GetSpendableBalancesOfAddress) => Effect.request(req, resolver),
    )

    return {
      getValidators,
      getSpendableBalancesOfAddress,
      getParams,
    } as const
  }),
  dependencies: [FetchHttpClient.layer],
}) {}

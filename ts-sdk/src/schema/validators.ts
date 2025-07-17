import { HttpClient, HttpClientResponse } from "@effect/platform"
import { Effect, flow, pipe } from "effect"
import * as A from "effect/Array"
import * as E from "effect/Either"
import * as O from "effect/Option"
import * as S from "effect/Schema"
import * as Struct from "effect/Struct"

export const Status = S.Union(
  S.Literal("BOND_STATUS_BONDED"),
  S.Literal("BOND_STATUS_UNBONDED"),
  S.Literal("BOND_STATUS_UNBONDING"),
  S.Literal("BOND_STATUS_UNSPECIFIED"),
)
export type Status = typeof Status.Type

export const Validator = S.Struct({
  operator_address: S.String,
  consensus_pubkey: S.Struct({
    "@type": S.String,
    "key": S.String,
  }),
  jailed: S.Boolean,
  status: Status,
  tokens: S.BigInt.pipe(
    S.greaterThanOrEqualToBigInt(0n),
  ),
  delegator_shares: S.BigDecimal,
  description: S.Struct({
    moniker: S.String,
    identity: S.OptionFromNonEmptyTrimmedString,
    website: S.URL.pipe(
      S.OptionFromUndefinedOr,
      S.annotations({
        decodingFallback: () => E.right(O.none()),
      }),
    ),
    security_contact: S.String,
    details: S.String,
  }),
  unbonding_height: S.BigInt.pipe(
    S.greaterThanOrEqualToBigInt(0n),
  ),
  unbonding_time: S.DateTimeUtc,
  commission: S.Struct({
    commission_rates: S.Struct({
      rate: S.BigDecimal,
      max_rate: S.BigDecimal,
      max_change_rate: S.BigDecimal,
    }),
    update_time: S.DateTimeUtc,
  }),
  min_self_delegation: S.NumberFromString.pipe(
    S.int(),
  ),
  unbonding_on_hold_ref_count: S.NumberFromString.pipe(
    S.int(),
  ),
  unbonding_ids: S.Array(S.String),
})
export type Validator = typeof Validator.Type

/**
 * TODO:
 * - Make icon a service and invertible
 * - Provide proper `encode`
 * - Remove duplicate `HttpClient` dependencies from `Staking` service
 */
export const ValidatorWithImage = S.transformOrFail(
  Validator,
  S.extend(
    Validator,
    S.Struct({ icon: S.OptionFromSelf(S.URLFromSelf) }),
  ),
  {
    decode: (fromA, _, _ast, fromI) =>
      pipe(
        HttpClient.HttpClient.pipe(
          Effect.map(HttpClient.withTracerDisabledWhen(() => true)),
          Effect.map(HttpClient.filterStatusOk),
        ),
        Effect.andThen((client) =>
          pipe(
            O.map(fromA.description.identity, (id) =>
              pipe(
                `https://keybase.io/_/api/1.0/user/lookup.json?key_suffix=${id}&fields=pictures`,
                (url) => client.get(url, { acceptJson: true }),
                Effect.flatMap(HttpClientResponse.schemaBodyJson(S.Struct({
                  them: S.Array(S.Struct({
                    pictures: S.Struct({
                      primary: S.Struct({
                        url: S.URL,
                      }),
                    }),
                  })),
                }))),
                Effect.map(flow(
                  Struct.get("them"),
                  A.head,
                  O.map(x => x.pictures.primary.url),
                )),
              )),
            Effect.flatten,
          )
        ),
        Effect.catchAll(() => Effect.succeed(O.none<URL>())),
        Effect.map((icon) =>
          ({
            ...fromI,
            icon,
          }) as const
        ),
      ),
    encode: () => Effect.succeed(void 0 as unknown as Validator),
    strict: true,
  },
)
export type ValidatorWithImage = typeof ValidatorWithImage.Type

import * as E from "effect/Either"
import * as O from "effect/Option"
import * as S from "effect/Schema"

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
    identity: S.String,
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

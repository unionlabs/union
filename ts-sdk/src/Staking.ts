/**
 * This module handles liquid staking.
 *
 * @since 2.0.0
 */

import { Effect, Hash, Layer, pipe, PrimaryKey, Request, Struct } from "effect"
import * as A from "effect/Array"
import * as O from "effect/Option"
import * as S from "effect/Schema"
import { graphql } from "gql.tada"
import { ChainFragment } from "./graphql/fragments/ChainFragment.js"
import { TokenFragment } from "./graphql/fragments/TokenFragment.js"
import { Indexer } from "./Indexer.js"
import { Bond, Unbond } from "./schema/stake.js"
import * as Ucs05 from "./Ucs05.js"

/**
 * @category errors
 * @since 2.0.0
 */
export class StakingError
  extends S.TaggedError<StakingError>("@unionlabs/sdk/Staking/StakingError")("StakingError", {
    message: S.String,
    cause: S.Any,
  })
{}

/**
 * @category requests
 * @since 2.0.0
 */
export class GetBonds
  extends S.TaggedRequest<GetBonds>()("@unionlabs/sdk/Staking/GetBondsRequest", {
    failure: StakingError,
    success: S.Option(S.NonEmptyArray(Bond)),
    payload: {
      addresses: S.ArrayEnsure(S.Union(Ucs05.AnyDisplay, Ucs05.AnyDisplayFromString)),
    },
  })
{
  /**
   * @since 2.0.0
   */
  [PrimaryKey.symbol]() {
    return pipe(
      this.addresses,
      A.map(Struct.get("address")),
      Hash.array,
    )
  }
}

/**
 * @category requests
 * @since 2.0.0
 */
export class GetUnbonds
  extends S.TaggedRequest<GetUnbonds>()("@unionlabs/sdk/Staking/GetUnbondsRequest", {
    failure: StakingError,
    success: S.Option(S.NonEmptyArray(Unbond)),
    payload: {
      addresses: S.ArrayEnsure(S.Union(Ucs05.AnyDisplay, Ucs05.AnyDisplayFromString)),
    },
  })
{
  /**
   * @since 2.0.0
   */
  [PrimaryKey.symbol]() {
    return pipe(
      this.addresses,
      A.map(Struct.get("address")),
      Hash.array,
    )
  }
}

/**
 * @category services
 * @since 2.0.0
 */
export class Staking extends Effect.Service<Staking>()("@unionlabs/sdk/Staking", {
  effect: Effect.gen(function*() {
    const client = yield* Indexer

    const getBonds: (request: GetBonds) => Effect.Effect<
      Request.Request.Success<GetBonds>,
      Request.Request.Error<GetBonds>
    > = Effect.fn((request) =>
      pipe(
        client.fetch({
          document: graphql(
            `
query GetBondsByAddress($addresses: jsonb!) @cached(ttl: 10) {
  v2_bonds(args: { p_addresses_canonical: $addresses }) {
    packet_hash
    delivery_packet_hash
    bond_success
    delivery_success
    packet_shape
    source_universal_chain_id
    remote_universal_chain_id
    destination_universal_chain_id
    sender_canonical
    sender_display
    sender_zkgm
    receiver_canonical
    receiver_display
    receiver_zkgm
    base_token
    base_amount
    quote_token
    quote_amount
    remote_base_token
    remote_base_amount
    remote_quote_token
    remote_quote_amount
    bond_send_timestamp
    bond_send_transaction_hash
    bond_recv_timestamp
    bond_recv_transaction_hash
    bond_timeout_timestamp
    bond_timeout_transaction_hash
    delivery_send_timestamp
    delivery_send_transaction_hash
    delivery_recv_timestamp
    delivery_recv_transaction_hash
    delivery_timeout_timestamp
    delivery_timeout_transaction_hash
    sort_order
    source_chain {
      ...ChainFragment
    }
    destination_chain {
      ...ChainFragment
    }
    base_token_meta {
      ...TokenFragment
    }
    quote_token_meta {
      ...TokenFragment
    }
    remote_base_token_meta {
      ...TokenFragment
    }
    remote_quote_token_meta {
      ...TokenFragment
    }
  }
}`,
            [ChainFragment, TokenFragment],
          ),
          variables: {
            addresses: pipe(
              request.addresses,
              A.map(Ucs05.anyDisplayToCanonical),
            ),
          },
        }),
        Effect.map(Struct.get("v2_bonds")),
        Effect.flatMap(O.liftPredicate(A.isNonEmptyArray)),
        Effect.map(A.map(x =>
          ({
            _tag: "Bond" as const,
            ...x,
          }) as const
        )),
        Effect.flatMap(S.decodeUnknown(S.NonEmptyArray(Bond))),
        Effect.optionFromOptional,
        Effect.catchTags({
          IndexerError: (cause) =>
            new StakingError({
              message: cause.message,
              cause,
            }),
          ParseError: (cause) =>
            new StakingError({
              message: "failed to decode",
              cause,
            }),
        }),
      )
    )

    const getUnbonds: (request: GetUnbonds) => Effect.Effect<
      Request.Request.Success<GetUnbonds>,
      Request.Request.Error<GetUnbonds>
    > = Effect.fn((request) =>
      pipe(
        client.fetch({
          document: graphql(
            `
query GetUnbondsByAddress($addresses: jsonb!) @cached(ttl: 10) {
  v2_unbonds(args: { p_addresses_canonical: $addresses }) {
    packet_hash
    success
    packet_shape
    source_universal_chain_id
    destination_universal_chain_id
    sender_canonical
    sender_display
    sender_zkgm
    base_token
    base_amount
    unbond_send_timestamp
    unbond_send_transaction_hash
    unbond_recv_timestamp
    unbond_recv_transaction_hash
    unbond_timeout_timestamp
    unbond_timeout_transaction_hash
    sort_order
    source_chain {
      ...ChainFragment
    }
    destination_chain {
      ...ChainFragment
    }
    base_token_meta {
      ...TokenFragment
    }
  }
}`,
            [ChainFragment, TokenFragment],
          ),
          variables: {
            addresses: pipe(
              request.addresses,
              A.map(Ucs05.anyDisplayToCanonical),
            ),
          },
        }),
        Effect.map(Struct.get("v2_unbonds")),
        Effect.flatMap(O.liftPredicate(A.isNonEmptyArray)),
        Effect.map(A.map(x =>
          ({
            _tag: "Unbond" as const,
            ...x,
          }) as const
        )),
        Effect.flatMap(S.decodeUnknown(S.NonEmptyArray(Unbond))),
        Effect.optionFromOptional,
        Effect.catchTags({
          IndexerError: (cause) =>
            new StakingError({
              message: cause.message,
              cause,
            }),
          ParseError: (cause) =>
            new StakingError({
              message: "failed to decode",
              cause,
            }),
        }),
      )
    )

    return {
      getBonds,
      getUnbonds,
    } as const
  }),
  dependencies: [Indexer.Default],
}) {
  static Test = Layer.effect(
    Staking,
    Effect.gen(function*() {
      const fc = yield* Effect.tryPromise({
        try: () => import("effect/FastCheck"),
        catch: (cause) =>
          new StakingError({
            message: `Could not import "effect/FastCheck"`,
            cause,
          }),
      })

      const Arbitrary = yield* Effect.tryPromise({
        try: () => import("effect/Arbitrary"),
        catch: (cause) =>
          new StakingError({
            message: `Could not import "effect/Arbitrary"`,
            cause,
          }),
      })

      const ArbitraryBonds = fc.array(
        Arbitrary.make(Bond),
        {
          minLength: 2,
        },
      )
      const ArbitraryUnbonds = fc.array(
        Arbitrary.make(Unbond),
        {
          minLength: 2,
        },
      )

      return Staking.make(
        {
          getBonds: (request: GetBonds): Effect.Effect<
            Request.Request.Success<GetBonds>,
            Request.Request.Error<GetBonds>
          > =>
            pipe(
              Hash.hash(request),
              (seed) =>
                fc.sample(ArbitraryBonds, {
                  numRuns: 1,
                  seed,
                })[0],
              O.liftPredicate(A.isNonEmptyArray),
              Effect.optionFromOptional,
              Effect.delay("200 millis"),
            ),
          getUnbonds: (request: GetUnbonds): Effect.Effect<
            Request.Request.Success<GetUnbonds>,
            Request.Request.Error<GetUnbonds>
          > =>
            pipe(
              Hash.hash(request),
              (seed) =>
                fc.sample(ArbitraryUnbonds, {
                  numRuns: 1,
                  seed,
                })[0],
              O.liftPredicate(A.isNonEmptyArray),
              Effect.optionFromOptional,
              Effect.delay("200 millis"),
            ),
        },
      )
    }),
  )
}

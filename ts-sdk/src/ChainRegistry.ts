/**
 * This module interfaces with the indexer via the GraphQL protocol.
 *
 * @since 2.0.0
 */
import {
  Array as A,
  Data,
  Effect,
  Hash,
  Layer,
  Option as O,
  pipe,
  Request,
  RequestResolver,
  Schema,
  Struct,
} from "effect"
import * as Predicate from "effect/Predicate"
import { graphql } from "gql.tada"
import { Indexer } from "./Indexer.js"
import { Chain, UniversalChainId } from "./schema/chain.js"

/**
 * @category requests
 * @since 2.0.0
 */
interface GetChainById extends Request.Request<Chain, ChainRegistryError> {
  readonly _tag: "GetChainById"
  readonly id: UniversalChainId
}
/**
 * @category models
 * @since 2.0.0
 */
const GetChainById = Request.tagged<GetChainById>("GetChainById")

/**
 * @category errors
 * @since 2.0.0
 */
export class ChainRegistryError
  extends Data.TaggedError("@unionlabs/sdk/ChainRegistry/ChainRegistryError")<{
    message: string
    cause?: unknown
  }>
{}

/**
 * @category services
 * @since 2.0.0
 */
export class ChainRegistry extends Effect.Service<ChainRegistry>()("@unionlabs/sdk/ChainRegistry", {
  effect: Effect.gen(function*() {
    const client = yield* Indexer

    const byUniversalId = Effect.fn((id: UniversalChainId) =>
      pipe(
        client.fetch({
          document: graphql(`
query GetChainByUniversalId($id: String!) @cached(ttl: 60) {
    v2_chains(args: { p_universal_chain_id: $id }) {
        chain_id
        universal_chain_id
        minter_address_display
        display_name
        addr_prefix
        rpc_type
        testnet
        editions {
            environment
            name
        }
        features(where: { environment: { _eq: "PRODUCTION" } }) {
            channel_list
            connection_list
            index_status
            packet_list
            transfer_submission
            transfer_list
        }
        rpcs {
            type
            url
        }
        explorers {
            address_url
            block_url
            description
            display_name
            home_url
            name
            tx_url
        }
    }
}
    `),
          variables: {
            id,
          },
        }),
        Effect.map(Struct.get("v2_chains")),
        Effect.flatMap(O.liftPredicate(Predicate.isTupleOf(1))),
        Effect.map(A.headNonEmpty),
        Effect.flatMap(Schema.decodeUnknown(Chain)),
        Effect.catchTags({
          IndexerError: (cause) =>
            new ChainRegistryError({
              message: cause.message,
              cause,
            }),
          NoSuchElementException: () =>
            new ChainRegistryError({
              message: `no such element or duplicate elements for ${id}`,
            }),
          ParseError: (cause) =>
            new ChainRegistryError({
              message: "failed to decode",
              cause,
            }),
        }),
      )
    )

    return {
      byUniversalId,
    } as const
  }),
  dependencies: [Indexer.Default],
  accessors: true,
}) {
  static Test = Layer.effect(
    this,
    Effect.gen(function*() {
      const fc = yield* Effect.tryPromise({
        try: () => import("effect/FastCheck"),
        catch: (cause) =>
          new ChainRegistryError({
            message: `Could not import "effect/FastCheck"`,
            cause,
          }),
      })

      const Arbitrary = yield* Effect.tryPromise({
        try: () => import("effect/Arbitrary"),
        catch: (cause) =>
          new ChainRegistryError({
            message: `Could not import "effect/Arbitrary"`,
            cause,
          }),
      })

      const ArbitraryChain = Arbitrary.make(Chain)

      return ChainRegistry.make(
        {
          byUniversalId: (id: UniversalChainId) =>
            pipe(
              Hash.string(id),
              (seed) =>
                fc.sample(ArbitraryChain, {
                  numRuns: 1,
                  seed,
                })[0],
              Effect.succeed,
            ),
        },
      )
    }),
  )
}

/**
 * @category resolvers
 * @since 2.0.0
 */
const GetChainByIdResolver =
  // we create a normal resolver like we did before
  RequestResolver.fromEffect((request: GetChainById) =>
    Effect.andThen(Indexer, (gql) =>
      pipe(
        gql.fetch({
          document: graphql(`
query GetChainByUniversalId($id: String!) @cached(ttl: 60) {
    v2_chains(args: { p_universal_chain_id: $id }) {
        chain_id
        universal_chain_id
        minter_address_display
        display_name
        addr_prefix
        rpc_type
        testnet
        editions {
            environment
            name
        }
        features(where: { environment: { _eq: "PRODUCTION" } }) {
            channel_list
            connection_list
            index_status
            packet_list
            transfer_submission
            transfer_list
        }
        rpcs {
            type
            url
        }
        explorers {
            address_url
            block_url
            description
            display_name
            home_url
            name
            tx_url
        }
    }
}
    `),
          variables: {
            id: request.id,
          },
        }),
        Effect.flatMap(({ v2_chains }) => Schema.decodeUnknown(Chain)(v2_chains[0])),
        Effect.mapError((e) =>
          new ChainRegistryError({
            message: e.message,
            cause: e,
          })
        ),
      ))
  ).pipe(
    RequestResolver.contextFromServices(Indexer),
  )

/**
 * @category utils
 * @since 2.0.0
 */
export const getChainById: (
  id: UniversalChainId,
) => Effect.Effect<Chain, ChainRegistryError, Indexer> = Effect.fn(
  (id) => Effect.request(GetChainById({ id }), GetChainByIdResolver),
)

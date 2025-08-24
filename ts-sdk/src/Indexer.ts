/**
 * This module interfaces with the indexer to source data.
 *
 * @since 2.0.0
 */
import { PersistedCache, Persistence } from "@effect/experimental"
import { KeyValueStore } from "@effect/platform"
import {
  absurd,
  Array as A,
  Config,
  Duration,
  Effect,
  Exit,
  flow,
  Hash,
  HashMap,
  identity,
  Layer,
  Match,
  Option as O,
  Order,
  pipe,
  PrimaryKey,
  Stream,
  Struct,
  SubscriptionRef,
} from "effect"
import * as Request from "effect/Request"
import * as S from "effect/Schema"
import { graphql, type TadaDocumentNode } from "gql.tada"
import { type ArgumentNode, type DirectiveNode, Kind, print } from "graphql"
import { GraphQLClient, type Variables } from "graphql-request"
import type { ClientError } from "graphql-request"
import { Hex } from "./schema/hex.js"
import { PacketHash } from "./schema/packet.js"

/**
 * @category errors
 * @since 2.0.0
 */
export class IndexerError
  extends S.TaggedError<IndexerError>("@unionlabs/sdk/Indexer/IndexerError")("IndexerError", {
    message: S.String,
    status: S.Number,
    errors: S.optional(S.Any),
    cause: S.Any,
  })
{
  /**
   * @category constructors
   * @since 2.0.0
   */
  static fromClientError(error: ClientError) {
    return new this({
      message: error.message,
      status: error.response.status,
      errors: error.response.errors,
      cause: error,
    })
  }
}

/**
 * @category utils
 * @since 2.0.0
 */
export const operationNamesFromDocumentNode = <T extends TadaDocumentNode<any, any>>(doc: T) =>
  pipe(
    doc.definitions,
    A.filter(x => x.kind === "OperationDefinition"),
    A.map(flow((x) => x.name?.value, O.fromNullable)),
    A.getSomes,
  )

/**
 * Define TTL strategy given some GQL document.
 * NOTE: Determines TTL by virtue of custom `@cached` directive.
 *
 * @category services
 * @since 2.0.0
 */
export class GraphQLCache
  extends Effect.Service<GraphQLCache>()("@unionlabs/sdk/GraphQL/GraphQLCache", {
    effect: Effect.gen(function*() {
      const timeToLive = <
        D,
        V extends Variables = Variables,
      >(document: TadaDocumentNode<D, V>): Duration.DurationInput => {
        const ttlFromArgumentNode = pipe(
          Match.type<ArgumentNode>(),
          Match.when(
            {
              name: { value: "ttl" },
              value: { kind: Kind.INT },
            },
            ({ value: { value } }) => +value,
          ),
          Match.option,
        )

        return pipe(
          document.definitions,
          A.filterMap(x =>
            x.kind === Kind.OPERATION_DEFINITION
              ? O.fromNullable(x.directives)
              : O.none()
          ),
          A.flatten,
          A.filterMap(pipe(
            Match.type<DirectiveNode>(),
            Match.when(
              {
                name: { value: "cached" },
                arguments: A.isNonEmptyReadonlyArray<ArgumentNode>,
              },
              ({ arguments: args }) =>
                pipe(
                  args,
                  A.filterMap(ttlFromArgumentNode),
                  A.head,
                ),
            ),
            Match.option,
          )),
          A.getSomes<O.Option<number>[]>,
          O.liftPredicate(A.isNonEmptyArray<number>),
          O.map(A.min(Order.number)),
          O.getOrElse(() => 0),
          (seconds) => `${seconds} seconds` as const,
        )
      }

      return { timeToLive } as const
    }),
    dependencies: [KeyValueStore.layerStorage(() => globalThis.sessionStorage)],
  })
{}

/**
 * A "generic" GraphQL request containing document and variables.
 * TODO: This should be defined on a per-request basis, which not only will provide better type safety but also abstract over GraphQL backing.
 *
 * @category models
 * @since 2.0.0
 */
export class IndexerRequest
  extends S.TaggedRequest<IndexerRequest>()("@unionlabs/sdk/Indexer/IndexerRequest", {
    failure: IndexerError,
    success: S.Any,
    payload: {
      document: S.Any,
      variables: S.Any,
    },
  })
{
  /**
   * @since 2.0.0
   */
  [PrimaryKey.symbol]() {
    const structure = {
      document: print(this.document),
      variables: this.variables,
    }
    return pipe(
      HashMap.fromIterable(Object.entries(structure)).toString(),
      Hash.string,
      (hash) => `${this._tag}:${hash}`,
    )
  }
}

// export const transferPacketHashQuery = ({ submission_tx_hash }: { submission_tx_hash: string }) =>
//   createQueryGraphql({
//     schema: Schema.Struct({
//       v2_transfers: Schema.Array(
//         Schema.Struct({
//           packet_hash: PacketHash,
//         }),
//       ),
//     }),
//     document: graphql(/* GraphQL */ `
//       query GetPacketHashBySubmissionTxHash($submission_tx_hash: String!) {
//           v2_transfers(args: {
//               p_transaction_hash: $submission_tx_hash
//           }) {
//               packet_hash
//           }
//       }
//   `),
//     variables: {
//       submission_tx_hash,
//     },
//     refetchInterval: "1 seconds",
//     writeData: data => {
//       transferHashStore.data = data.pipe(
//         Option.flatMap(result =>
//           result.v2_transfers.length > 0
//             ? Option.some(result.v2_transfers[0].packet_hash)
//             : Option.none()
//         ),
//       )
//     },

//     writeError: error => {
//       transferHashStore.error = error
//     },
//   })

/**
 * @category requests
 * @since 2.0.0
 */
export class GetPacketHashBySubmissionTxHash
  extends S.TaggedRequest<GetPacketHashBySubmissionTxHash>("GetPacketHashBySubmissionTxHash")(
    "GetPacketHashBySubmissionTxHash",
    {
      success: PacketHash,
      failure: IndexerError,
      payload: {
        submissionTxHash: Hex,
      },
    },
  )
{}

/**
 * @category services
 * @since 2.0.0
 */
export class Indexer extends Effect.Service<Indexer>()("@unionlabs/sdk/Indexer", {
  scoped: Effect.gen(function*() {
    const { timeToLive } = yield* GraphQLCache

    const defaultEndpoint = yield* Config.string("GRAPHQL_ENDPOINT").pipe(
      Effect.catchTag(
        "ConfigError",
        () => Effect.succeed("https://graphql.union.build/v1/graphql"),
      ),
    )

    const endpoint = yield* SubscriptionRef.make(defaultEndpoint)

    const client = new GraphQLClient(defaultEndpoint)

    const fetch = <D, V extends Variables = Variables>(options: {
      document: TadaDocumentNode<D, V>
      variables?: V
    }) =>
      Effect.gen(function*() {
        const { document, variables } = options

        const fetch = Effect.tryPromise({
          try: (signal) =>
            client.request<D, any>({
              document,
              variables,
              signal,
            }),
          catch: (error) => IndexerError.fromClientError(error as ClientError),
        }).pipe(
          Effect.withLogSpan("fetch"),
        )

        const operationName = pipe(
          document,
          operationNamesFromDocumentNode,
          A.head,
          O.getOrElse(() => "unknown"),
        )
        const message = `request.gql.${operationName}`

        return yield* pipe(
          fetch,
          Effect.tap(Effect.log(message)),
          Effect.tapErrorCause((cause) => Effect.logError(message, cause)),
          Effect.annotateLogs({
            operationName,
            variables,
          }),
          Effect.withLogSpan("GraphQL.fetch"),
        )
      })

    const cache = yield* PersistedCache.make({
      storeId: "graphql:",
      lookup: (x: IndexerRequest) =>
        fetch({
          document: x.document,
          variables: x.variables,
        }),
      timeToLive: (req, exit) =>
        // Don't cache failed results
        Exit.match(exit, {
          onSuccess: () => timeToLive(req.document as unknown as any),
          onFailure: () => 0,
        }),
    })

    const fetchCached = Effect.fn("fetchCached")(
      function*<D, V extends Variables = Variables>(options: {
        document: TadaDocumentNode<D, V>
        variables?: V
      }) {
        const { document, variables } = options

        const request = new IndexerRequest(
          { document, variables },
          { disableValidation: true },
        )

        const liveFetch = fetch<D, any>({ document, variables })
        const invalidate = cache.invalidate(request)

        // attempt cache invalidation before ultimatetly querying live endpoint
        const recover = invalidate.pipe(
          Effect.andThen(() => liveFetch),
          Effect.catchTag("PersistenceError", () => liveFetch),
        )

        return identity<D>(
          yield* pipe(
            cache.get(request),
            Effect.tapError((e) =>
              Effect.gen(function*() {
                if (e._tag !== "PersistenceError") {
                  return
                }
                console.log({ ...e })
              })
            ),
            Effect.catchTag("PersistenceError", () => recover),
          ),
        )
      },
    )

    yield* pipe(
      endpoint.changes,
      Stream.mapEffect((url) =>
        Effect.sync(() => {
          client.setEndpoint(url)
        })
      ),
      Stream.tap((a) => Effect.log("Updated GQL endpoint:", a)),
      Stream.runDrain,
      Effect.forkDaemon,
    )

    return {
      fetch: fetchCached,
      updateEndpoint: (url: string) => SubscriptionRef.set(endpoint, url),
      getPacketHashBySubmissionTxHash: (
        request: GetPacketHashBySubmissionTxHash,
      ): Effect.Effect<
        Request.Request.Success<GetPacketHashBySubmissionTxHash>,
        Request.Request.Error<GetPacketHashBySubmissionTxHash>
      > =>
        pipe(
          fetchCached({
            document: graphql(`
query GetPacketHashBySubmissionTxHash($submission_tx_hash: String!) {
    v2_transfers(args: {
        p_transaction_hash: $submission_tx_hash
    }) {
        packet_hash
    }
}
`),
            variables: {
              submission_tx_hash: request.submissionTxHash,
            },
          }),
          Effect.flatMap(S.decodeUnknown(
            S.Struct({
              v2_transfers: S.NonEmptyArray(
                S.Struct({
                  packet_hash: PacketHash,
                }),
              ),
            }),
          )),
          Effect.map(({ v2_transfers }) =>
            pipe(
              A.headNonEmpty(v2_transfers),
              Struct.get("packet_hash"),
            )
          ),
          Effect.catchTag("ParseError", (cause) =>
            IndexerError.make({
              message: cause.message,
              status: 200,
              cause,
            })),
        ),
      getEndpoint: SubscriptionRef.get(endpoint),
      resetCache: Effect.sync(() => {
        localStorage.removeItem("quota_check")
        Object.keys(localStorage).forEach(key => {
          if (key.startsWith("graphql")) {
            localStorage.removeItem(key)
          }
        })
      }),
    } as const
  }),
  accessors: true,
  dependencies: [
    GraphQLCache.Default,
    Persistence.layerResultKeyValueStore.pipe(
      Layer.provide(
        KeyValueStore.layerStorage(() => globalThis.localStorage),
      ),
    ),
  ],
}) {
  static Test = Layer.effect(
    Indexer,
    Effect.gen(function*() {
      // TODO: Make meaningfully invertible GQL mocking solution.
      // const fc = yield* Effect.promise(() => import(`effect/FastCheck`))
      // const Arbitrary = yield* Effect.promise(() => import("effect/Arbitrary"))
      // const Schema = yield* Effect.promise(() => import("effect/Schema"))

      return new Indexer({
        fetch: absurd as unknown as any,
        updateEndpoint: absurd as unknown as any,
        getPacketHashBySubmissionTxHash: absurd as unknown as any,
        getEndpoint: absurd as unknown as any,
        resetCache: absurd as unknown as any,
      })
    }),
  )
}

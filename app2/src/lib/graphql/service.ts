import { URLS } from "$lib/constants"
import { PersistedCache, Persistence } from "@effect/experimental"
import { KeyValueStore } from "@effect/platform"
import { operationNamesFromDocumentNode } from "@unionlabs/sdk/utils"
import {
  Array as A,
  Duration,
  Effect,
  Exit,
  flow,
  Hash,
  Layer,
  Match,
  Option as O,
  pipe,
  PrimaryKey,
  Schema as S,
} from "effect"
import type { TadaDocumentNode } from "gql.tada"
import { type ArgumentNode, Kind } from "graphql"
import { GraphQLClient, type Variables } from "graphql-request"
import { ClientError } from "graphql-request"
import { GraphQLError } from "./error"

/**
 * Define TTL strategy given some GQL document.
 * NOTE: Determines TTL by virtue of custom `@cached` directive.
 * TODO: Should probably define full cache behavior.
 */
class GraphQLCache extends Effect.Service<GraphQLCache>()("app/GraphQL/Cache", {
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
        A.filter(x => x.kind === Kind.OPERATION_DEFINITION),
        A.map(flow((x) => x.directives, O.fromNullable)),
        A.getSomes,
        A.flatten,
        A.map(flow(
          Match.value,
          Match.when(
            {
              name: { value: "cached" },
              arguments: Match.defined,
            },
            ({ arguments: args }) =>
              pipe(
                args,
                A.map(ttlFromArgumentNode),
                A.getSomes,
                A.head,
              ),
          ),
          Match.option,
        )),
        A.map(O.flatten),
        A.getSomes,
        // XXX: get min
        A.head,
        O.getOrElse(() => 0),
        (seconds) => `${seconds} seconds` as const,
      )
    }

    return { timeToLive } as const
  }),
  dependencies: [KeyValueStore.layerStorage(() => globalThis.sessionStorage)],
}) {}

/**
 * A "generic" GraphQL request containing document and variables.
 * TODO: This should be defined on a per-request basis, which not only will provide better type safety but also abstract over GraphQL backing.
 */
export class GraphQLRequest extends S.TaggedRequest<GraphQLRequest>()("GraphQLRequest", {
  failure: GraphQLError,
  success: S.Any,
  payload: {
    document: S.Any,
    variables: S.Any,
  },
}) {
  [PrimaryKey.symbol]() {
    return pipe(
      {
        document: this.document,
        variables: this.variables,
      },
      Hash.structure,
      (hash) => `${this._tag}:${hash}`,
    )
  }
}

export class GraphQL extends Effect.Service<GraphQL>()("app/GraphQL", {
  scoped: Effect.gen(function*() {
    const { timeToLive } = yield* GraphQLCache

    const client = new GraphQLClient(URLS().GRAPHQL)

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
          catch: (error) => GraphQLError.make(error as ClientError),
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
      storeId: "graphql",
      lookup: (x: GraphQLRequest) =>
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

    const fetchCached = <D, V extends Variables = Variables>(options: {
      document: TadaDocumentNode<D, V>
      variables?: V
    }) =>
      pipe(
        cache.get(
          new GraphQLRequest(
            {
              document: options.document,
              variables: options.variables,
            },
            { disableValidation: true },
          ),
        ),
        // XXX: override result type
        Effect.map(x => x as D),
      )

    return {
      fetch: fetchCached,
    } as const
  }),
  dependencies: [
    GraphQLCache.Default,
    Persistence.layerResultKeyValueStore.pipe(
      Layer.provide(
        KeyValueStore.layerStorage(() => globalThis.sessionStorage),
      ),
    ),
  ],
}) {
  static Test = Layer.effect(
    GraphQL,
    Effect.gen(function*() {
      // TODO: Make meaningfully invertible GQL mocking solution.
      // const fc = yield* Effect.promise(() => import(`effect/FastCheck`))
      // const Arbitrary = yield* Effect.promise(() => import("effect/Arbitrary"))
      // const Schema = yield* Effect.promise(() => import("effect/Schema"))

      return new GraphQL({
        fetch: () => {
          throw new Error("unimplemented")
        },
      })
    }),
  )
}

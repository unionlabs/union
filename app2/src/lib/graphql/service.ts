import { ENV } from "$lib/constants"
import * as SvelteConfigProvider from "$lib/services/SvelteConfigProvider.js"
import { PersistedCache, Persistence } from "@effect/experimental"
import { KeyValueStore } from "@effect/platform"
import { operationNamesFromDocumentNode } from "@unionlabs/sdk/utils/index"
import {
  absurd,
  Array as A,
  Config,
  Duration,
  Effect,
  ExecutionPlan,
  Exit,
  Hash,
  HashMap,
  identity,
  Layer,
  Match,
  Option as O,
  Order,
  pipe,
  PrimaryKey,
  Schema as S,
  Stream,
  SubscriptionRef,
} from "effect"
import type { TadaDocumentNode } from "gql.tada"
import { type ArgumentNode, type DirectiveNode, Kind, print } from "graphql"
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
              arguments: Match.defined,
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
        A.getSomes,
        O.liftPredicate(A.isNonEmptyArray),
        O.map(A.min(Order.number)),
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

const ConfigPlan = ExecutionPlan.make(
  { provide: SvelteConfigProvider.SearchParams },
  { provide: SvelteConfigProvider.StaticPublic },
)

export class GraphQL extends Effect.Service<GraphQL>()("app/GraphQL", {
  scoped: Effect.gen(function*() {
    const { timeToLive } = yield* GraphQLCache

    const defaultEndpoint = yield* Config.string("GRAPHQL_ENDPOINT").pipe(
      Effect.withExecutionPlan(ConfigPlan),
      Effect.catchTag(
        "ConfigError",
        () =>
          pipe(
            Match.value(ENV()),
            Match.when("DEVELOPMENT", () => "https://development.graphql.union.build/v1/graphql"),
            Match.orElse(() => "https://graphql.union.build/v1/graphql"),
            Effect.succeed,
          ),
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
      storeId: "graphql:",
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

    const fetchCached = Effect.fn("fetchCached")(
      function*<D, V extends Variables = Variables>(options: {
        document: TadaDocumentNode<D, V>
        variables?: V
      }) {
        const { document, variables } = options

        const request = new GraphQLRequest(
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
    GraphQL,
    Effect.gen(function*() {
      // TODO: Make meaningfully invertible GQL mocking solution.
      // const fc = yield* Effect.promise(() => import(`effect/FastCheck`))
      // const Arbitrary = yield* Effect.promise(() => import("effect/Arbitrary"))
      // const Schema = yield* Effect.promise(() => import("effect/Schema"))

      return new GraphQL({
        fetch: absurd as unknown as any,
        updateEndpoint: absurd as unknown as any,
        getEndpoint: absurd as unknown as any,
        resetCache: absurd as unknown as any,
      })
    }),
  )
}

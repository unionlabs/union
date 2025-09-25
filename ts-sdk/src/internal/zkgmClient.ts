import { Cause, Context, Effect, Exit, Fiber, Inspectable, Layer, Predicate, Stream } from "effect"
import { dual } from "effect/Function"
import { globalValue } from "effect/GlobalValue"
import { pipeArguments } from "effect/Pipeable"
import type * as Client from "../ZkgmClient.js"
import type * as ClientError from "../ZkgmClientError.js"
import type * as ClientRequest from "../ZkgmClientRequest.js"
import type * as ClientResponse from "../ZkgmClientResponse.js"
import * as IncomingMessage from "../ZkgmIncomingMessage.js"
import * as internalResponse from "./zkgmClientResponse.js"

/** @internal */
export const TypeId: Client.TypeId = Symbol.for(
  "@unionlabs/sdk/ZkgmClient",
) as Client.TypeId

/** @internal */
export const tag = Context.GenericTag<Client.ZkgmClient>("@unionlabs/sdk/ZkgmClient")

export const {
  /** @internal */
  execute,
} = Effect.serviceFunctions(tag)

const ClientProto = {
  [TypeId]: TypeId,
  pipe() {
    return pipeArguments(this, arguments)
  },
  ...Inspectable.BaseProto,
  toJSON() {
    return {
      _id: "@unionlabs/sdk/ZkgmClient",
    }
  },
}

// const isClient = (u: unknown): u is Client.ZkgmClient.With<unknown, unknown> =>
//   Predicate.hasProperty(u, TypeId)

interface ZkgmClientImpl<E, R> extends Client.ZkgmClient.With<E, R> {
  readonly preprocess: Client.ZkgmClient.Preprocess<E, R>
  readonly postprocess: Client.ZkgmClient.Postprocess<E, R>
}

/** @internal */
export const makeWith = <E2, R2, E, R>(
  postprocess: (
    request: Effect.Effect<ClientRequest.ZkgmClientRequest, E2, R2>,
  ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
  preprocess: Client.ZkgmClient.Preprocess<E2, R2>,
): Client.ZkgmClient.With<E, R> => {
  const self = Object.create(ClientProto)
  self.preprocess = preprocess
  self.postprocess = postprocess
  self.execute = function(request: ClientRequest.ZkgmClientRequest) {
    return postprocess(preprocess(request))
  }
  return self
}

const scopedRequests = globalValue(
  "@unionlabs/sdk/ZkgmClient/scopedRequests",
  () => new WeakMap<ClientRequest.ZkgmClientRequest, AbortController>(),
)

const responseRegistry = globalValue(
  "@unionlabs/sdk/ZkgmClient/responseRegistry",
  () => {
    if ("FinalizationRegistry" in globalThis && globalThis.FinalizationRegistry) {
      const registry = new FinalizationRegistry((controller: AbortController) => {
        controller.abort()
      })
      return {
        register(response: ClientResponse.ZkgmClientResponse, controller: AbortController) {
          registry.register(response, controller, response)
        },
        unregister(response: ClientResponse.ZkgmClientResponse) {
          registry.unregister(response)
        },
      }
    }

    const timers = new Map<ClientResponse.ZkgmClientResponse, any>()
    return {
      register(response: ClientResponse.ZkgmClientResponse, controller: AbortController) {
        timers.set(response, setTimeout(() => controller.abort(), 5000))
      },
      unregister(response: ClientResponse.ZkgmClientResponse) {
        const timer = timers.get(response)
        if (timer === undefined) {
          return
        }
        clearTimeout(timer)
        timers.delete(response)
      },
    }
  },
)

class InterruptibleResponse implements ClientResponse.ZkgmClientResponse {
  constructor(
    readonly original: ClientResponse.ZkgmClientResponse,
    readonly controller: AbortController,
  ) {}

  readonly [internalResponse.TypeId]: ClientResponse.TypeId = internalResponse.TypeId
  readonly [IncomingMessage.TypeId]: IncomingMessage.TypeId = IncomingMessage.TypeId

  private applyInterrupt<A, E, R>(effect: Effect.Effect<A, E, R>) {
    return Effect.suspend(() => {
      responseRegistry.unregister(this.original)
      return Effect.onInterrupt(effect, () =>
        Effect.sync(() => {
          this.controller.abort()
        }))
    })
  }

  get request() {
    return this.original.request
  }

  get txHash() {
    return this.original.txHash
  }

  get safeHash() {
    return this.original.safeHash
  }

  get stream() {
    return Stream.suspend(() => {
      responseRegistry.unregister(this.original)
      return Stream.ensuringWith(this.original.stream, (exit) => {
        if (Exit.isInterrupted(exit)) {
          this.controller.abort()
        }
        return Effect.void
      })
    })
  }

  waitFor<A extends IncomingMessage.LifecycleEvent>(
    refinement: Predicate.Refinement<NoInfer<IncomingMessage.LifecycleEvent>, A>,
  ) {
    return this.applyInterrupt(this.original.waitFor(refinement))
  }

  toJSON() {
    return this.original.toJSON()
  }

  [Inspectable.NodeInspectSymbol]() {
    return this.original[Inspectable.NodeInspectSymbol]()
  }
}

/** @internal */
export const make = (
  f: (
    request: ClientRequest.ZkgmClientRequest,
    signal: AbortSignal,
    fiber: Fiber.RuntimeFiber<ClientResponse.ZkgmClientResponse, ClientError.ClientError>,
  ) => Effect.Effect<ClientResponse.ZkgmClientResponse, ClientError.ClientError>,
): Client.ZkgmClient =>
  makeWith((effect) =>
    Effect.flatMap(effect, (request) =>
      Effect.withFiberRuntime((fiber) => {
        const scopedController = scopedRequests.get(request)
        const controller = scopedController ?? new AbortController()
        // TODO: at some point, return encode request, return Either, map error to `ZkgmClientError`
        const effect = f(request, controller.signal, fiber)
        if (scopedController) {
          return effect
        }
        return Effect.uninterruptibleMask((restore) =>
          Effect.matchCauseEffect(restore(effect), {
            onSuccess(response) {
              responseRegistry.register(response, controller)
              return Effect.succeed(new InterruptibleResponse(response, controller))
            },
            onFailure(cause) {
              if (Cause.isInterrupted(cause)) {
                controller.abort()
              }
              return Effect.failCause(cause)
            },
          })
        )
      })), Effect.succeed as Client.ZkgmClient.Preprocess<never, never>)

/** @internal */
export const transform = dual<
  <E, R, E1, R1>(
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
      request: ClientRequest.ZkgmClientRequest,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ) => (self: Client.ZkgmClient.With<E, R>) => Client.ZkgmClient.With<E | E1, R | R1>,
  <E, R, E1, R1>(
    self: Client.ZkgmClient.With<E, R>,
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
      request: ClientRequest.ZkgmClientRequest,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ) => Client.ZkgmClient.With<E | E1, R | R1>
>(2, (self, f) => {
  const client = self as ZkgmClientImpl<any, any>
  return makeWith(
    Effect.flatMap((request) => f(client.postprocess(Effect.succeed(request)), request)),
    client.preprocess,
  )
})

/** @internal */
export const transformResponse = dual<
  <E, R, E1, R1>(
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ) => (self: Client.ZkgmClient.With<E, R>) => Client.ZkgmClient.With<E1, R1>,
  <E, R, E1, R1>(
    self: Client.ZkgmClient.With<E, R>,
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ) => Client.ZkgmClient.With<E1, R1>
>(2, (self, f) => {
  const client = self as ZkgmClientImpl<any, any>
  return makeWith((request) => f(client.postprocess(request)), client.preprocess)
})

/** @internal */
export const layerMergedContext = <E, R>(
  effect: Effect.Effect<Client.ZkgmClient, E, R>,
): Layer.Layer<Client.ZkgmClient, E, R> =>
  Layer.effect(
    tag,
    Effect.flatMap(Effect.context<never>(), (context) =>
      Effect.map(effect, (client) =>
        transformResponse(
          client,
          Effect.mapInputContext((input: Context.Context<never>) =>
            Context.merge(context, input)
          ),
        ))),
  )

// /** @internal */
// export const withFee = <E, R>(
//   self: Client.ZkgmClient.With<E, R>,
// ): Client.ZkgmClient.With<E, R | FeeCalculator> =>
//   transform(
//     self,
//     (effect, request) => Effect.flatMap(
//       effect,
//       (response) => pipe(
//         Gas
//       ))
//     },
//   )

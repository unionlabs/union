import { Context, Effect, Fiber, Inspectable, Predicate } from "effect"
import { dual } from "effect/Function"
import { globalValue } from "effect/GlobalValue"
import { pipeArguments } from "effect/Pipeable"
import type * as Client from "../Client.js"
import type * as ClientError from "../ClientError.js"
import type * as ClientRequest from "../ClientRequest.js"
import type * as ClientResponse from "../ClientResponse.js"
import * as internalRequest from "./clientRequest.js"

/** @internal */
export const TypeId: Client.TypeId = Symbol.for(
  "@unionlabs/sdk/Client",
) as Client.TypeId

/** @internal */
export const tag = Context.GenericTag<Client.Client>("@unionlabs/sdk/Client")

export const {
  /** @internal */
  execute,
  /** @internal */
  simulate,
} = Effect.serviceFunctions(tag)

const ClientProto = {
  [TypeId]: TypeId,
  pipe() {
    return pipeArguments(this, arguments)
  },
  ...Inspectable.BaseProto,
  toJSON() {
    return {
      _id: "@unionlabs/sdk/Client",
    }
  },
  send(
    this: Client.Client,
    sender: string,
    receiver: string,
    options?: ClientRequest.Options.Send,
  ) {
    return this.execute(internalRequest.send(sender, receiver, options))
  },
}

const isClient = (u: unknown): u is Client.Client.With<unknown, unknown> =>
  Predicate.hasProperty(u, TypeId)

interface ClientImpl<E, R> extends Client.Client.With<E, R> {
  readonly preprocess: Client.Client.Preprocess<E, R>
  readonly postprocess: Client.Client.Postprocess<E, R>
}

/** @internal */
export const makeWith = <E2, R2, E, R>(
  postprocess: (
    request: Effect.Effect<ClientRequest.ClientRequest, E2, R2>,
  ) => Effect.Effect<ClientResponse.ClientResponse, E, R>,
  preprocess: Client.Client.Preprocess<E2, R2>,
): Client.Client.With<E, R> => {
  const self = Object.create(ClientProto)
  self.preprocess = preprocess
  self.postprocess = postprocess
  self.execute = function(request: ClientRequest.ClientRequest) {
    return postprocess(preprocess(request))
  }
  return self
}

/** @internal */
export const transform = dual<
  <E, R, E1, R1>(
    f: (
      effect: Effect.Effect<ClientResponse.ClientResponse, E, R>,
      request: ClientRequest.ClientRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E1, R1>,
  ) => (self: Client.Client.With<E, R>) => Client.Client.With<E | E1, R | R1>,
  <E, R, E1, R1>(
    self: Client.Client.With<E, R>,
    f: (
      effect: Effect.Effect<ClientResponse.ClientResponse, E, R>,
      request: ClientRequest.ClientRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E1, R1>,
  ) => Client.Client.With<E | E1, R | R1>
>(2, (self, f) => {
  const client = self as ClientImpl<any, any>
  return makeWith(
    Effect.flatMap((request) => f(client.postprocess(Effect.succeed(request)), request)),
    client.preprocess,
  )
})

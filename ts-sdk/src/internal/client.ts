import { Context, Effect, Fiber, Inspectable } from "effect"
import { pipeArguments } from "effect/Pipeable"
import type * as Client from "../Client.js"
import type * as ClientRequest from "../ClientRequest.js"
import type * as ClientResponse from "../ClientResponse.js"
import type * as ClientError from "../ClientError.js"
import * as internalRequest from "./clientRequest.js"
import { globalValue } from "effect/GlobalValue"

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

const scopedRequests = globalValue(
  "@unionlabs/sdk/Client/scopedRequests",
  () => new WeakMap<ClientRequest.ClientRequest, AbortController>()
)

/** @internal */
export const make = (
  f: (
    request: ClientRequest.ClientRequest,
    url: URL,
    signal: AbortSignal,
    fiber: Fiber.RuntimeFiber<ClientResponse.ClientResponse, ClientError.ClientError>,
  ) => Effect.Effect<ClientResponse.ClientResponse, ClientError.ClientError>,
): Client.Client =>
  makeWith((effect) =>
    Effect.flatMap(effect, (request) =>
      Effect.withFiberRuntime((fiber) => {
        const a = f(
        const scopedController = scopedRequests.get(request)
        const controller = scopedController ?? new AbortController()
        const urlResult = UrlParams.makeUrl(request.url, request.urlParams, request.hash)
        if (urlResult._tag === "Left") {
          return Effect.fail(
            new Error.RequestError({ request, reason: "InvalidUrl", cause: urlResult.left }),
          )
        }
        const url = urlResult.right
        const tracerDisabled = !fiber.getFiberRef(FiberRef.currentTracerEnabled)
          || fiber.getFiberRef(currentTracerDisabledWhen)(request)
        if (tracerDisabled) {
          const effect = f(request, url, controller.signal, fiber)
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
        }
        const nameGenerator = Context.get(fiber.currentContext, SpanNameGenerator)
        return Effect.useSpan(
          nameGenerator(request),
          { kind: "client", captureStackTrace: false },
          (span) => {
            span.attribute("http.request.method", request.method)
            span.attribute("server.address", url.origin)
            if (url.port !== "") {
              span.attribute("server.port", +url.port)
            }
            span.attribute("url.full", url.toString())
            span.attribute("url.path", url.pathname)
            span.attribute("url.scheme", url.protocol.slice(0, -1))
            const query = url.search.slice(1)
            if (query !== "") {
              span.attribute("url.query", query)
            }
            const redactedHeaderNames = fiber.getFiberRef(Headers.currentRedactedNames)
            const redactedHeaders = Headers.redact(request.headers, redactedHeaderNames)
            for (const name in redactedHeaders) {
              span.attribute(`http.request.header.${name}`, String(redactedHeaders[name]))
            }
            request = fiber.getFiberRef(currentTracerPropagation)
              ? internalRequest.setHeaders(request, TraceContext.toHeaders(span))
              : request
            return Effect.uninterruptibleMask((restore) =>
              restore(f(request, url, controller.signal, fiber)).pipe(
                Effect.withParentSpan(span),
                Effect.matchCauseEffect({
                  onSuccess: (response) => {
                    span.attribute("http.response.status_code", response.status)
                    const redactedHeaders = Headers.redact(response.headers, redactedHeaderNames)
                    for (const name in redactedHeaders) {
                      span.attribute(`http.response.header.${name}`, String(redactedHeaders[name]))
                    }
                    if (scopedController) {
                      return Effect.succeed(response)
                    }
                    responseRegistry.register(response, controller)
                    return Effect.succeed(new InterruptibleResponse(response, controller))
                  },
                  onFailure(cause) {
                    if (!scopedController && Cause.isInterrupted(cause)) {
                      controller.abort()
                    }
                    return Effect.failCause(cause)
                  },
                }),
              )
            )
          },
        )
      })), Effect.succeed as Client.HttpClient.Preprocess<never, never>)

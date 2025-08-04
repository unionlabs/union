import * as UrlParams from "@effect/platform/UrlParams"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as Error from "@unionlabs/sdk/ZkgmClientError"
import type * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import * as Context from "effect/Context"
import * as Effect from "effect/Effect"
import * as FiberRef from "effect/FiberRef"
import { type LazyArg } from "effect/Function"
import { globalValue } from "effect/GlobalValue"
import * as Inspectable from "effect/Inspectable"
import * as Option from "effect/Option"
import * as Stream from "effect/Stream"
import * as HeaderParser from "multipasta/HeadersParser"

/** @internal */
export const xhrTagKey = "@effect/platform-browser/BrowserHttpClient/XMLHttpRequest"

const xhrTag = Context.GenericTag<LazyArg<XMLHttpRequest>>(xhrTagKey)

/** @internal */
export const currentXHRResponseType = globalValue(
  "@effect/platform-browser/BrowserHttpClient/currentXHRResponseType",
  () => FiberRef.unsafeMake<"text" | "arraybuffer">("text"),
)

/** @internal */
export const withXHRArrayBuffer = <A, E, R>(
  effect: Effect.Effect<A, E, R>,
): Effect.Effect<A, E, R> =>
  Effect.locally(
    effect,
    currentXHRResponseType,
    "arraybuffer",
  )

const makeXhr = () => new XMLHttpRequest()

const makeXMLHttpRequest = Client.make((request, url, signal, fiber) =>
  Effect.suspend(() => {
    const xhr = Context.getOrElse(
      fiber.getFiberRef(FiberRef.currentContext),
      xhrTag,
      () => makeXhr,
    )()
    signal.addEventListener("abort", () => {
      xhr.abort()
      xhr.onreadystatechange = null
    }, { once: true })
    xhr.open(request.method, url.toString(), true)
    xhr.responseType = fiber.getFiberRef(currentXHRResponseType)
    Object.entries(request.headers).forEach(([k, v]) => {
      xhr.setRequestHeader(k, v)
    })
    return Effect.zipRight(
      sendBody(xhr, request),
      Effect.async<ClientResponseImpl, Error.RequestError>((resume) => {
        let sent = false
        const onChange = () => {
          if (!sent && xhr.readyState >= 2) {
            sent = true
            resume(Effect.succeed(new ClientResponseImpl(request, xhr)))
          }
        }
        xhr.onreadystatechange = onChange
        xhr.onerror = (_event) => {
          resume(Effect.fail(
            new Error.RequestError({
              request,
              reason: "Transport",
              cause: xhr.statusText,
            }),
          ))
        }
        onChange()
        return Effect.void
      }),
    )
  })
)

const sendBody = (
  xhr: XMLHttpRequest,
  request: ClientRequest.ZkgmClientRequest,
): Effect.Effect<void, Error.RequestError> => {
  const body = request.body
  switch (body._tag) {
    case "Empty":
      return Effect.sync(() => xhr.send())
    case "Raw":
      return Effect.sync(() => xhr.send(body.body as any))
    case "Uint8Array":
      return Effect.sync(() => xhr.send(body.body))
    case "FormData":
      return Effect.sync(() => xhr.send(body.formData))
    case "Stream":
      return Effect.matchEffect(
        Stream.runFold(body.stream, new Uint8Array(0), (acc, chunk) => {
          const next = new Uint8Array(acc.length + chunk.length)
          next.set(acc, 0)
          next.set(chunk, acc.length)
          return next
        }),
        {
          onFailure: (cause) =>
            Effect.fail(
              new Error.RequestError({
                request,
                reason: "Encode",
                cause,
              }),
            ),
          onSuccess: (body) => Effect.sync(() => xhr.send(body)),
        },
      )
  }
}

const encoder = new TextEncoder()

/** @internal */
export abstract class IncomingMessageImpl<E> extends Inspectable.Class
  implements IncomingMessage.HttpIncomingMessage<E>
{
  readonly [IncomingMessage.TypeId]: IncomingMessage.TypeId

  constructor(
    readonly source: XMLHttpRequest,
    readonly onError: (error: unknown) => E,
  ) {
    super()
    this[IncomingMessage.TypeId] = IncomingMessage.TypeId
    this._rawHeaderString = source.getAllResponseHeaders()
  }

  private _rawHeaderString: string
  private _rawHeaders: Record<string, string | Array<string>> | undefined
  private _headers: Headers.Headers | undefined
  get headers() {
    if (this._headers) {
      return this._headers
    }
    if (this._rawHeaderString === "") {
      return this._headers = Headers.empty
    }
    const parser = HeaderParser.make()
    const result = parser(encoder.encode(this._rawHeaderString + "\r\n"), 0)
    this._rawHeaders = result._tag === "Headers" ? result.headers : undefined
    const parsed = result._tag === "Headers" ? Headers.fromInput(result.headers) : Headers.empty
    return this._headers = parsed
  }

  cachedCookies: Cookies.Cookies | undefined
  get cookies() {
    if (this.cachedCookies) {
      return this.cachedCookies
    }
    if (this._rawHeaders === undefined) {
      return Cookies.empty
    } else if (this._rawHeaders["set-cookie"] === undefined) {
      return this.cachedCookies = Cookies.empty
    }
    return this.cachedCookies = Cookies.fromSetCookie(this._rawHeaders["set-cookie"])
  }

  get remoteAddress() {
    return Option.none()
  }

  _textEffect: Effect.Effect<string, E> | undefined
  get text(): Effect.Effect<string, E> {
    if (this._textEffect) {
      return this._textEffect
    }
    return this._textEffect = Effect.async<string, E>((resume) => {
      if (this.source.readyState === 4) {
        resume(Effect.succeed(this.source.responseText))
        return
      }

      const onReadyStateChange = () => {
        if (this.source.readyState === 4) {
          resume(Effect.succeed(this.source.responseText))
        }
      }
      const onError = () => {
        resume(Effect.fail(this.onError(this.source.statusText)))
      }
      this.source.addEventListener("readystatechange", onReadyStateChange)
      this.source.addEventListener("error", onError)
      return Effect.sync(() => {
        this.source.removeEventListener("readystatechange", onReadyStateChange)
        this.source.removeEventListener("error", onError)
      })
    }).pipe(
      Effect.cached,
      Effect.runSync,
    )
  }

  get json(): Effect.Effect<unknown, E> {
    return Effect.tryMap(this.text, {
      try: (_) => _ === "" ? null : JSON.parse(_) as unknown,
      catch: this.onError,
    })
  }

  get urlParamsBody(): Effect.Effect<UrlParams.UrlParams, E> {
    return Effect.flatMap(this.text, (_) =>
      Effect.try({
        try: () => UrlParams.fromInput(new URLSearchParams(_)),
        catch: this.onError,
      }))
  }

  get stream(): Stream.Stream<Uint8Array, E> {
    return Stream.async<Uint8Array, E>((emit) => {
      let offset = 0
      const onReadyStateChange = () => {
        if (this.source.readyState === 3) {
          emit.single(encoder.encode(this.source.responseText.slice(offset)))
          offset = this.source.responseText.length
        } else if (this.source.readyState === 4) {
          if (offset < this.source.responseText.length) {
            emit.single(encoder.encode(this.source.responseText.slice(offset)))
          }
          emit.end()
        }
      }
      const onError = () => {
        emit.fail(this.onError(this.source.statusText))
      }
      this.source.addEventListener("readystatechange", onReadyStateChange)
      this.source.addEventListener("error", onError)
      onReadyStateChange()
      return Effect.sync(() => {
        this.source.removeEventListener("readystatechange", onReadyStateChange)
        this.source.removeEventListener("error", onError)
      })
    })
  }

  _arrayBufferEffect: Effect.Effect<ArrayBuffer, E> | undefined
  get arrayBuffer(): Effect.Effect<ArrayBuffer, E> {
    if (this._arrayBufferEffect) {
      return this._arrayBufferEffect
    }
    return this._arrayBufferEffect = Effect.async<ArrayBuffer, E>((resume) => {
      if (this.source.readyState === 4) {
        resume(Effect.succeed(this.source.response))
        return
      }

      const onReadyStateChange = () => {
        if (this.source.readyState === 4) {
          resume(Effect.succeed(this.source.response))
        }
      }
      const onError = () => {
        resume(Effect.fail(this.onError(this.source.statusText)))
      }
      this.source.addEventListener("readystatechange", onReadyStateChange)
      this.source.addEventListener("error", onError)
      return Effect.sync(() => {
        this.source.removeEventListener("readystatechange", onReadyStateChange)
        this.source.removeEventListener("error", onError)
      })
    }).pipe(
      Effect.map((response) => {
        if (typeof response === "string") {
          const arr = encoder.encode(response)
          return arr.byteLength !== arr.buffer.byteLength
            ? arr.buffer.slice(arr.byteOffset, arr.byteOffset + arr.byteLength)
            : arr.buffer
        }
        return response
      }),
      Effect.cached,
      Effect.runSync,
    )
  }
}

class ClientResponseImpl extends IncomingMessageImpl<Error.ResponseError>
  implements ClientResponse.HttpClientResponse
{
  readonly [ClientResponse.TypeId]: ClientResponse.TypeId

  constructor(
    readonly request: ClientRequest.HttpClientRequest,
    source: XMLHttpRequest,
  ) {
    super(source, (cause) =>
      new Error.ResponseError({
        request,
        response: this,
        reason: "Decode",
        cause,
      }))
    this[ClientResponse.TypeId] = ClientResponse.TypeId
  }

  get status() {
    return this.source.status
  }

  get formData(): Effect.Effect<FormData, Error.ResponseError> {
    return Effect.die("Not implemented")
  }

  toString(): string {
    return `ClientResponse(${this.status})`
  }

  toJSON(): unknown {
    return IncomingMessage.inspect(this, {
      _id: "@effect/platform/HttpClientResponse",
      request: this.request.toJSON(),
      status: this.status,
    })
  }
}

/** @internal */
export const layerXMLHttpRequest = Client.layerMergedContext(Effect.succeed(makeXMLHttpRequest))

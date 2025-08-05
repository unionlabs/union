import * as Evm from "@unionlabs/sdk/Evm"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as Error from "@unionlabs/sdk/ZkgmClientError"
import type * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import * as Effect from "effect/Effect"
import * as Inspectable from "effect/Inspectable"
import * as Stream from "effect/Stream"

const fromWallet = (wallet: Evm.WalletClient): Client.ZkgmClient =>
  Client.make((request, signal, fiber) => {
    return Effect.succeed(void 0 as unknown as ClientResponseImpl)
  })

// const makeXMLHttpRequest = Client.make((request, signal, fiber) =>
//   Effect.suspend(() => {
//     // TODO: get wallet tag (or make browser wallet client)
//     const xhr = Context.getOrElse(
//       fiber.getFiberRef(FiberRef.currentContext),
//       xhrTag,
//       () => makeXhr,
//     )()
//     // TODO: abort handling
//     signal.addEventListener("abort", () => {
//       xhr.abort()
//       xhr.onreadystatechange = null
//     }, { once: true })
//     xhr.open(request.method, url.toString(), true)
//     xhr.responseType = fiber.getFiberRef(currentXHRResponseType)
//     Object.entries(request.headers).forEach(([k, v]) => {
//       xhr.setRequestHeader(k, v)
//     })
//     return Effect.zipRight(
//       sendBody(xhr, request),
//       Effect.async<ClientResponseImpl, Error.RequestError>((resume) => {
//         let sent = false
//         const onChange = () => {
//           if (!sent && xhr.readyState >= 2) {
//             sent = true
//             resume(Effect.succeed(new ClientResponseImpl(request, xhr)))
//           }
//         }
//         xhr.onreadystatechange = onChange
//         xhr.onerror = (_event) => {
//           resume(Effect.fail(
//             new Error.RequestError({
//               request,
//               reason: "Transport",
//               cause: xhr.statusText,
//             }),
//           ))
//         }
//         onChange()
//         return Effect.void
//       }),
//     )
//   })
// )

/** @internal */
export abstract class IncomingMessageImpl<E> extends Inspectable.Class
  implements IncomingMessage.ZkgmIncomingMessage<E>
{
  readonly [IncomingMessage.TypeId]: IncomingMessage.TypeId

  constructor(
    readonly onError: (error: unknown) => E,
  ) {
    super()
    this[IncomingMessage.TypeId] = IncomingMessage.TypeId
  }

  get stream() {
    return Stream.fail("not implemented") as unknown as Stream.Stream<any, any>
  }

  waitFor(
    // TODO: use subset specific to evm
    pred: (a: IncomingMessage.LifecycleEvent) => boolean,
  ) {
    return Effect.die("not implemented")
  }
}

class ClientResponseImpl extends IncomingMessageImpl<Error.ResponseError>
  implements ClientResponse.ZkgmClientResponse
{
  readonly [ClientResponse.TypeId]: ClientResponse.TypeId

  constructor(
    readonly request: ClientRequest.ZkgmClientRequest,
  ) {
    super((error) =>
      new Error.ResponseError({
        reason: "Decode",
        request,
        response: this,
        cause: error,
      })
    )
    this[ClientResponse.TypeId] = ClientResponse.TypeId
  }

  toString(): string {
    return `EvmZkgmClient::ClientResponseImpl::toString not implemented`
  }

  toJSON(): unknown {
    return IncomingMessage.inspect(this, {
      _id: "@unionlabs/sdk/ZkgmClientResponse",
      request: this.request.toJSON(),
    })
  }
}

/** @internal */
export const make = Effect.map(Evm.WalletClient.Service, fromWallet)

/** @internal */
export const layerWithoutWallet = Client.layerMergedContext(make)

import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Utils from "@unionlabs/sdk/Utils"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as ClientError from "@unionlabs/sdk/ZkgmClientError"
import type * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import { pipe } from "effect"
import * as Effect from "effect/Effect"
import * as Inspectable from "effect/Inspectable"
import * as Stream from "effect/Stream"
import * as EvmWallet from "../EvmWallet.js"
import { ZkgmIncomingMessageImpl } from "./zkgmIncomingMessage.js"

const fromWallet = (wallet: EvmWallet.EvmWallet): Client.ZkgmClient =>
  Client.make((request, signal, fiber) =>
    Effect.gen(function*() {
      const timeoutTimestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()
      const salt = yield* Utils.generateSalt("evm").pipe(
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "crypto error",
          })
        ),
      )
      const operand = yield* pipe(
        request.instruction.encode,
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "instruction encode",
          })
        ),
      )
      const a = EvmWallet.writeContract({
        account: wallet.account,
        abi: Ucs03.Abi,
        chain: wallet.chain,
        functionName: "send",
        address: request.ucs03Address,
        args: [
          request.channelId,
          0n,
          timeoutTimestamp,
          salt,
          {
            opcode: request.instruction.opcode,
            version: request.instruction.version,
            operand,
          },
        ],
      }).pipe(
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "writeContract",
          })
        ),
        Effect.provideService(EvmWallet.EvmWallet, wallet),
      )

      return void 0 as unknown as ClientResponseImpl
    })
  )

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

class ClientResponseImpl extends IncomingMessageImpl<ClientError.ResponseError>
  implements ClientResponse.ZkgmClientResponse
{
  readonly [ClientResponse.TypeId]: ClientResponse.TypeId

  constructor(
    readonly request: ClientRequest.ZkgmClientRequest,
  ) {
    super((error) =>
      new ClientError.ResponseError({
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

  stream() {
    return
  }
}

/** @internal */
export const make = Effect.map(EvmWallet.EvmWallet, fromWallet)

/** @internal */
export const layerWithoutWallet = Client.layerMergedContext(make)

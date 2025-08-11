import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Utils from "@unionlabs/sdk/Utils"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as ClientError from "@unionlabs/sdk/ZkgmClientError"
import type * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import { pipe, Predicate } from "effect"
import * as Effect from "effect/Effect"
import * as Inspectable from "effect/Inspectable"
import * as Schema from "effect/Schema"
import * as Stream from "effect/Stream"
import * as Cosmos from "../Cosmos.js"

const fromSigningClient = (
  opts: { client: Cosmos.Cosmos.PublicClient; signingClient: Cosmos.Cosmos.SigningClient },
): Client.ZkgmClient =>
  Client.make((request, signal, fiber) =>
    Effect.gen(function*() {
      const {
        signingClient,
        client,
      } = opts

      const timeout_timestamp = Utils.getTimeoutInNanoseconds24HoursFromNow().toString()

      const salt = yield* Utils.generateSalt("cosmos").pipe(
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "crypto error",
          })
        ),
      )

      const instruction = yield* pipe(
        request.instruction.encode,
        Effect.flatMap(Schema.decode(Ucs03.Ucs03FromHex)),
        Effect.flatMap(Schema.encode(Ucs03.Ucs03WithInstructionFromHex)),
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "instruction encode",
          })
        ),
      )

      const funds = [] as const

      const sendInstruction = Cosmos.executeContract(
        signingClient.address,
        request.ucs03Address,
        {
          send: {
            channel_id: request.channelId,
            timeout_height: "0",
            timeout_timestamp,
            salt,
            instruction,
          },
        },
        funds,
      ).pipe(
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "writeContract",
          })
        ),
        Effect.provideService(Cosmos.SigningClient, signingClient),
      )

      return yield* pipe(
        sendInstruction,
        Effect.map((result) => new ClientResponseImpl(request, client, result)),
      )
    })
  )

/** @internal */
export abstract class IncomingMessageImpl<E> extends Inspectable.Class
  implements IncomingMessage.ZkgmIncomingMessage<E>
{
  readonly [IncomingMessage.TypeId]: IncomingMessage.TypeId

  constructor(
    readonly client: Cosmos.Cosmos.PublicClient,
    readonly result: ExecuteResult,
    readonly onError: (error: unknown) => E,
  ) {
    super()
    this[IncomingMessage.TypeId] = IncomingMessage.TypeId
  }

  get stream() {
    return Stream.empty
  }

  waitFor<A extends IncomingMessage.LifecycleEvent>(
    refinement: Predicate.Refinement<NoInfer<IncomingMessage.LifecycleEvent>, A>,
  ) {
    return Stream.runHead(this.stream)
  }
}

class ClientResponseImpl extends IncomingMessageImpl<ClientError.ResponseError>
  implements ClientResponse.ZkgmClientResponse
{
  readonly [ClientResponse.TypeId]: ClientResponse.TypeId

  constructor(
    readonly request: ClientRequest.ZkgmClientRequest,
    readonly client: Cosmos.Cosmos.PublicClient,
    readonly result: ExecuteResult,
  ) {
    super(
      client,
      result,
      (error) =>
        new ClientError.ResponseError({
          reason: "OnChain",
          request,
          response: this,
          cause: error,
        }),
    )
    this[ClientResponse.TypeId] = ClientResponse.TypeId
  }

  get txHash() {
    return this.result.transactionHash as `0x${string}`
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
export const make = Effect.map(
  Effect.all({ client: Cosmos.Client, signingClient: Cosmos.SigningClient }),
  fromSigningClient,
)

/** @internal */
export const layerWithoutSigningClient = Client.layerMergedContext(make)

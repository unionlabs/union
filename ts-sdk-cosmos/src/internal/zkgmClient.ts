import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate"
import { Call, Token } from "@unionlabs/sdk"
import { tokenMetaOverride } from "@unionlabs/sdk/Constants"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Utils from "@unionlabs/sdk/Utils"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as ClientError from "@unionlabs/sdk/ZkgmClientError"
import * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import * as ZkgmInstruction from "@unionlabs/sdk/ZkgmInstruction"
import { Cause, ParseResult, Predicate } from "effect"
import { pipe } from "effect"
import * as A from "effect/Array"
import * as Effect from "effect/Effect"
import * as E from "effect/Either"
import * as Inspectable from "effect/Inspectable"
import * as Match from "effect/Match"
import * as O from "effect/Option"
import * as S from "effect/Schema"
import * as Stream from "effect/Stream"
import * as Cosmos from "../Cosmos.js"

export const fromSigningClient = (
  opts: { client: Cosmos.Cosmos.PublicClient; signingClient: Cosmos.Cosmos.SigningClient },
): Client.ZkgmClient =>
  Client.make((request, signal, fiber) =>
    Effect.gen(function*() {
      const {
        signingClient,
        client,
      } = opts

      const encodeInstruction: (
        u: ZkgmInstruction.ZkgmInstruction,
      ) => Effect.Effect<
        Ucs03.Ucs03,
        ParseResult.ParseError | Cause.TimeoutException | Cosmos.QueryContractError
      > = pipe(
        Match.type<ZkgmInstruction.ZkgmInstruction>(),
        Match.tagsExhaustive({
          Batch: (batch) =>
            pipe(
              batch.instructions,
              A.map(encodeInstruction),
              Effect.allWith({ concurrency: "unbounded" }),
              Effect.map((operand) =>
                new Ucs03.Batch({
                  opcode: batch.opcode,
                  version: batch.version,
                  operand,
                })
              ),
            ),
          TokenOrder: (self) =>
            pipe(
              Match.value(self),
              Match.when(
                { version: 1 },
                (v1) =>
                  Effect.gen(function*() {
                    const meta = yield* pipe(
                      Cosmos.readCw20TokenInfo(v1.baseToken.address as unknown as any),
                      Effect.either,
                      Effect.map(E.getOrElse(() => tokenMetaOverride(v1.baseToken.address))),
                      Effect.provideService(Cosmos.Client, client),
                    )

                    return yield* TokenOrder.encodeV1(v1)({
                      ...meta,
                      sourceChannelId: request.channelId,
                    })
                  }),
              ),
              Match.when(
                { version: 2 },
                (v2) => TokenOrder.encodeV2(v2),
              ),
              Match.exhaustive,
            ),
          Call: Call.encode,
        }),
      )

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
        encodeInstruction(request.instruction),
        Effect.flatMap(S.encode(Ucs03.Ucs03WithInstructionFromHex)),
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "instruction encode",
          })
        ),
      )

      const funds = pipe(
        ClientRequest.requiredFunds(request),
        O.map(A.filter(([x]) => Token.isNative(x))),
        O.map(A.map(([token, amount]) => ({
          denom: token.address,
          amount: `${amount}`,
        }))),
        O.getOrElse(A.empty),
      )

      const args = [
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
      ] as const

      const sendInstruction = Cosmos.executeContract(
        ...args,
      ).pipe(
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "writeContract",
          })
        ),
        Effect.tapErrorCause((cause) => Effect.logError("Cosmos submission failed", cause)),
        Effect.annotateLogs({
          args,
          signingClient,
          client,
        }),
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
    return pipe(
      this.stream,
      Stream.filter(refinement),
      Stream.runHead,
    )
  }
}

export class ClientResponseImpl extends IncomingMessageImpl<ClientError.ResponseError>
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
    return `0x${this.result.transactionHash}` as const
  }

  get safeHash() {
    return O.none()
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

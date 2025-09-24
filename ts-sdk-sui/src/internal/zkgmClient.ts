import { Indexer, ZkgmIncomingMessage } from "@unionlabs/sdk"
import * as Call from "@unionlabs/sdk/Call"
import type { Hex } from "@unionlabs/sdk/schema/hex"
import * as Token from "@unionlabs/sdk/Token"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Utils from "@unionlabs/sdk/Utils"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as ClientError from "@unionlabs/sdk/ZkgmClientError"
import * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import * as ZkgmInstruction from "@unionlabs/sdk/ZkgmInstruction"
import { Brand, Chunk, flow, Match, ParseResult, pipe, Predicate, Tuple } from "effect"
import * as A from "effect/Array"
import * as Effect from "effect/Effect"
import * as Inspectable from "effect/Inspectable"
import * as O from "effect/Option"
import * as S from "effect/Schema"
import * as Stream from "effect/Stream"
import * as Sui from "../Sui.js"
import * as Safe from "../Safe.js"
// import { Sui } from "../index.js"

export const fromWallet = (
  opts: { client: Sui.Sui.PublicClient; wallet: Sui.Sui.WalletClient },
): Client.ZkgmClient =>
  Client.make((request, signal, fiber) =>
    Effect.gen(function*() {
      const {
        wallet,
        client,
      } = opts

      const encodeInstruction: (
        u: ZkgmInstruction.ZkgmInstruction,
      ) => Effect.Effect<Ucs03.Ucs03, ParseResult.ParseError | Sui.ReadContractError> = pipe(
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
                      Sui.readCoinMetadata(
                        v1.baseToken.address as unknown as any
                      ),
                      Effect.provideService(Sui.PublicClient, client),
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
        }
      ),
      )

      console.log("[@unionlabs/sdk-sui/internal/zkgmClient]", { wallet, client })

      const timeoutTimestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()
      const salt = yield* Utils.generateSalt("sui").pipe(
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "crypto error",
          })
        ),
      )

      console.log("[@unionlabs/sdk-sui/internal/zkgmClient]", { salt, timeoutTimestamp })

      const operand = yield* pipe(
        encodeInstruction(request.instruction),
        Effect.flatMap(S.encode(Ucs03.Ucs03FromHex)),
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "instruction encode",
          })
        ),
      )

      console.log("[@unionlabs/sdk-sui/internal/zkgmClient]", { operand })

      const funds = ClientRequest.requiredFunds(request).pipe(
        O.map(A.filter(([x]) => Token.isNative(x))),
        O.flatMap(O.liftPredicate(A.isNonEmptyReadonlyArray)),
        O.map(A.map(flow(Tuple.getSecond))),
        O.map(A.reduce(0n, (acc, n) => acc + n)),
        O.getOrUndefined,
      )

      console.log("[@unionlabs/sdk-sui/internal/zkgmClient]", { funds })

      const args = [
        request.channelId,
        0n,
        timeoutTimestamp,
        salt,
        {
          opcode: request.instruction.opcode,
          version: request.instruction.version,
          operand,
        },
      ] as const

      console.log("[@unionlabs/sdk-sui/internal/zkgmClient]", { args })

      // // TODO: Fix writecontract calling, decide parameters etc.
      // const sendInstruction = Sui.writeContract({
      //   client: client,
      //   account: wallet.signer,
      //   abi: Ucs03.Abi,
      //   chain: wallet.chain,
      //   functionName: "send",
      //   address: request.ucs03Address as unknown as any,
      //   args,
      //   value: funds,
      // }).pipe(
      //   Effect.mapError((cause) =>
      //     new ClientError.RequestError({
      //       reason: "Transport",
      //       request,
      //       cause,
      //       description: "writeContract",
      //     })
      //   ),
      //   Effect.provideService(Evm.WalletClient, wallet),
      // )

      // return yield* pipe(
      //   sendInstruction,
      //   Effect.map((txHash) => new ClientResponseImpl(request, client, txHash)),
      // )
    })
  )

/** @internal */
export abstract class IncomingMessageImpl<E> extends Inspectable.Class
  implements IncomingMessage.ZkgmIncomingMessage<E>
{
  readonly [IncomingMessage.TypeId]: IncomingMessage.TypeId

  constructor(
    readonly client: Sui.Sui.PublicClient,
    readonly txHash: Hex,
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
    readonly client: Sui.Sui.PublicClient,
    readonly txHash: Hex,
  ) {
    super(client, txHash, (error) =>
      new ClientError.ResponseError({
        reason: "OnChain",
        request,
        response: this,
        cause: error,
      }))
    this[ClientResponse.TypeId] = ClientResponse.TypeId
  }

  toString(): string {
    return `SuiZkgmClient::ClientResponseImpl::toString not implemented`
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
  Effect.all({ client: Sui.PublicClient, wallet: Sui.WalletClient }),
  fromWallet,
)

/** @internal */
export const layerWithoutWallet = Client.layerMergedContext(make)

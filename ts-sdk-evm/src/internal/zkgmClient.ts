import { Indexer, ZkgmIncomingMessage } from "@unionlabs/sdk"
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
import * as Evm from "../Evm.js"
import * as Safe from "../Safe.js"

export const fromWallet = (
  opts: { client: Evm.Evm.PublicClient; wallet: Evm.Evm.WalletClient },
): Client.ZkgmClient =>
  Client.make((request, signal, fiber) =>
    Effect.gen(function*() {
      const {
        wallet,
        client,
      } = opts

      const encodeInstruction: (
        u: ZkgmInstruction.ZkgmInstruction,
      ) => Effect.Effect<Ucs03.Ucs03, ParseResult.ParseError | Evm.ReadContractError> = pipe(
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
                      Evm.readErc20Meta(
                        v1.baseToken.address as unknown as any,
                        request.source.universal_chain_id,
                      ),
                      Effect.provideService(Evm.PublicClient, client),
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
        }),
      )

      console.log("[@unionlabs/sdk-evm/internal/zkgmClient]", { wallet, client })

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

      console.log("[@unionlabs/sdk-evm/internal/zkgmClient]", { salt, timeoutTimestamp })

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

      console.log("[@unionlabs/sdk-evm/internal/zkgmClient]", { operand })

      const funds = ClientRequest.requiredFunds(request).pipe(
        O.map(A.filter(([x]) => Token.isNative(x))),
        O.flatMap(O.liftPredicate(A.isNonEmptyReadonlyArray)),
        O.map(A.map(flow(Tuple.getSecond))),
        O.map(A.reduce(0n, (acc, n) => acc + n)),
        O.getOrUndefined,
      )

      console.log("[@unionlabs/sdk-evm/internal/zkgmClient]", { funds })

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

      console.log("[@unionlabs/sdk-evm/internal/zkgmClient]", { args })

      const sendInstruction = Evm.writeContract({
        account: wallet.account,
        abi: Ucs03.Abi,
        chain: wallet.chain,
        functionName: "send",
        address: request.ucs03Address as unknown as any,
        args,
        value: funds,
      }).pipe(
        Effect.mapError((cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "writeContract",
          })
        ),
        Effect.provideService(Evm.WalletClient, wallet),
      )

      return yield* pipe(
        sendInstruction,
        Effect.map((txHash) => new ClientResponseImpl(request, client, txHash)),
      )
    })
  )

/** @internal */
export abstract class IncomingMessageImpl<E> extends Inspectable.Class
  implements IncomingMessage.ZkgmIncomingMessage<E>
{
  readonly [IncomingMessage.TypeId]: IncomingMessage.TypeId

  constructor(
    readonly client: Evm.Evm.PublicClient,
    readonly txHash: Hex,
    readonly onError: (error: unknown) => E,
  ) {
    super()
    this[IncomingMessage.TypeId] = IncomingMessage.TypeId
  }

  get stream(): Stream.Stream<ZkgmIncomingMessage.LifecycleEvent, any> {
    return Stream.async<ZkgmIncomingMessage.LifecycleEvent, any>((emit) => {
      const self = this

      const waitForReceipt = (hash: `0x${string}`) =>
        pipe(
          Evm.waitForTransactionReceipt(this.txHash),
          Effect.map((a) =>
            ZkgmIncomingMessage.LifecycleEvent.EvmTransactionReceiptComplete({
              transactionHash: a.transactionHash as `0x${string}` & Brand.Brand<"Hash">,
              blockHash: a.blockHash as `0x${string}` & Brand.Brand<"Hash">,
              gasUsed: a.gasUsed,
            })
          ),
          Effect.provideService(Evm.PublicClient, this.client),
        )

      const maybeWaitForReceipt = pipe(
        Effect.serviceOption(Safe.Safe),
        Effect.flatMap(
          O.match({
            onNone: () =>
              pipe(
                waitForReceipt(this.txHash),
                Effect.map(Chunk.of),
                Effect.mapError(O.some),
              ),
            onSome: () => Effect.succeed(Chunk.empty<ZkgmIncomingMessage.LifecycleEvent>()),
          }),
        ),
      )

      const maybeWaitForSafe = pipe(
        Effect.serviceOption(Safe.Safe),
        Effect.flatMap(
          O.match({
            onNone: () => Effect.succeed(Chunk.empty<ZkgmIncomingMessage.LifecycleEvent>()),
            onSome: (safe) =>
              pipe(
                safe.resolveTxHash(
                  this.txHash,
                ),
                Effect.map((hash) =>
                  ZkgmIncomingMessage.LifecycleEvent.WaitForSafeWalletHash({
                    hash: hash as Hex & Brand.Brand<"Hash">,
                  })
                ),
                Effect.map(Chunk.of),
                Effect.mapError(O.some),
              ),
          }),
        ),
      )

      const maybeIndex = pipe(
        Effect.serviceOption(Indexer.Indexer),
        Effect.flatMap(
          O.match({
            onNone: () => Effect.succeed(Chunk.empty<ZkgmIncomingMessage.LifecycleEvent>()),
            onSome: (indexer) =>
              pipe(
                indexer.getPacketHashBySubmissionTxHash(
                  new Indexer.GetPacketHashBySubmissionTxHash({
                    submissionTxHash: self.txHash,
                  }),
                ),
                Effect.map((packetHash) =>
                  ZkgmIncomingMessage.LifecycleEvent.Indexed({ packetHash })
                ),
                Effect.map(Chunk.of),
                Effect.mapError(O.some),
              ),
          }),
        ),
      )

      emit(maybeWaitForSafe)
      emit(maybeWaitForReceipt)
      emit(maybeIndex)
    })
  }

  waitFor<A extends ZkgmIncomingMessage.LifecycleEvent>(
    refinement: Predicate.Refinement<NoInfer<ZkgmIncomingMessage.LifecycleEvent>, A>,
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
    readonly client: Evm.Evm.PublicClient,
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
  Effect.all({ client: Evm.PublicClient, wallet: Evm.WalletClient }),
  fromWallet,
)

/** @internal */
export const layerWithoutWallet = Client.layerMergedContext(make)

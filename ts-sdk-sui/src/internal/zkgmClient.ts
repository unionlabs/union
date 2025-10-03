import { Transaction } from "@mysten/sui/transactions"
import * as Call from "@unionlabs/sdk/Call"
import type { Hex } from "@unionlabs/sdk/schema/hex"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Utils from "@unionlabs/sdk/Utils"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as ClientError from "@unionlabs/sdk/ZkgmClientError"
import * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import * as ZkgmInstruction from "@unionlabs/sdk/ZkgmInstruction"
import { Match, ParseResult, pipe, Predicate } from "effect"
import * as A from "effect/Array"
import * as Effect from "effect/Effect"
import * as Inspectable from "effect/Inspectable"
import * as S from "effect/Schema"
import * as Stream from "effect/Stream"
import * as Sui from "../Sui.js"

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
                      Sui.readCoinMeta(
                        v1.baseToken.address as unknown as any,
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
        }),
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

      const tx = new Transaction()
      const CLOCK_OBJECT_ID = "0x6" // Sui system clock
      const tHeight = 0n
      const module = "zkgm" // zkgm module name

      const suiParams = request.transport?.sui
      console.log("request.transport:", request.transport)
      if (!suiParams) {
        return yield* Effect.fail(
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause: new Error("Missing Sui transport params on ZkgmClientRequest.transport.sui"),
            description: "Provide relayStoreId/vaultId/ibcStoreId and coins[]",
          }),
        )
      }

      const { relayStoreId, vaultId, ibcStoreId, coins } = suiParams

      console.log("[@unionlabs/sdk-sui/internal/zkgmClient]", {
        relayStoreId,
        vaultId,
        ibcStoreId,
        coins,
      })

      const hexToBytes = (hex: `0x${string}`): Uint8Array => {
        const s = hex.slice(2)
        const out = new Uint8Array(s.length / 2)
        for (let i = 0; i < out.length; i++) {
          out[i] = parseInt(s.slice(i * 2, i * 2 + 2), 16)
        }
        return out
      }

      // 1) begin_send(channel_id: u32, salt: vector<u8>) -> SendCtx
      let sendCtx = tx.moveCall({
        target: `${request.ucs03Address}::${module}::begin_send`,
        typeArguments: [],
        arguments: [
          tx.pure.u32(Number(request.channelId)),
          tx.pure.vector("u8", hexToBytes(salt as `0x${string}`)),
        ],
      })

      // 2) For each coin: send_with_coin<T>(relay_store, vault, ibc_store, coin, version, opcode, operand, ctx) -> SendCtx
      for (const { typeArg, objectId } of coins) {
        sendCtx = tx.moveCall({
          target: `${request.ucs03Address}::${module}::send_with_coin`,
          typeArguments: [typeArg],
          arguments: [
            tx.object(relayStoreId),
            tx.object(vaultId),
            tx.object(ibcStoreId),
            tx.object(objectId),
            tx.pure.u8(Number(request.instruction.version)),
            tx.pure.u8(Number(request.instruction.opcode)),
            tx.pure.vector("u8", hexToBytes(operand as `0x${string}`)),
            sendCtx,
          ],
        })
      }

      // 3) end_send(ibc_store, clock, t_height: u64, timeout_ns: u64, ctx)
      tx.moveCall({
        target: `${request.ucs03Address}::${module}::end_send`,
        typeArguments: [],
        arguments: [
          tx.object(ibcStoreId),
          tx.object(CLOCK_OBJECT_ID),
          tx.pure.u64(tHeight),
          tx.pure.u64(BigInt(timeoutTimestamp)),
          sendCtx,
        ],
      })

      // sign & execute
      const submit = Effect.tryPromise({
        try: async () =>
          wallet.client.signAndExecuteTransaction({
            signer: wallet.signer,
            transaction: tx,
          }),
        catch: (cause) =>
          new ClientError.RequestError({
            reason: "Transport",
            request,
            cause,
            description: "signAndExecuteTransaction",
          }),
      })

      const res = yield* submit

      console.log("Res.transaction:", res.transaction)
      const txHash = (res.digest ?? res.transaction?.txSignatures[0] ?? "") as Hex

      return new ClientResponseImpl(request, client, txHash)
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

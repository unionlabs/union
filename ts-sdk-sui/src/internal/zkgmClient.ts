import { Transaction } from "@mysten/sui/transactions"
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
import * as Safe from "../Safe.js"
import * as Sui from "../Sui.js"

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

      // These will be fetched from hubble or from deployments.json
      const packageId = "0x8675045186976da5b60baf20dc94413fb5415a7054052dc14d93c13d3dbdf830" // zkgm package id
      // TODO: packageId can be changed when zkgm updated
      const relayStoreId = "0x393a99c6d55d9a79efa52dea6ea253fef25d2526787127290b985222cc20a924" // This won't be changed for a while
      const vaultId = "0x7c4ade19208295ed6bf3c4b58487aa4b917ba87d31460e9e7a917f7f12207ca3" // This won't be changed for a while
      const ibcStoreId = "0xac7814eebdfbf975235bbb796e07533718a9d83201346769e5f281dc90009175" // This won't be changed

      // This 2 will be get by user all the time
      const typeArg = "0x2::sui::SUI" // TODO: This should be dynamic based on the token sent
      const coinObjectId = "0x89c430d35fa9f2778b0a635027b178146eb26d70d16292c289304d476ecf76cd" // TODO: This should be given by user
      // Note: There can be multiple coins, for simplicity we are using one coin here
      // User should be able to provide typeArgs and coinObjectIds array

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
        target: `${packageId}::${module}::begin_send`,
        typeArguments: [],
        arguments: [
          tx.pure.u32(Number(request.channelId)),
          tx.pure.vector("u8", hexToBytes(salt as `0x${string}`)),
        ],
      })

      // 2) send_with_coin<T>(relay_store, vault, ibc_store, coin, version, opcode, operand, ctx) -> SendCtx
      sendCtx = tx.moveCall({
        target: `${packageId}::${module}::send_with_coin`,
        typeArguments: [typeArg],
        arguments: [
          tx.object(relayStoreId),
          tx.object(vaultId),
          tx.object(ibcStoreId),
          tx.object(coinObjectId),
          tx.pure.u8(Number(request.instruction.version)),
          tx.pure.u8(Number(request.instruction.opcode)),
          tx.pure.vector("u8", hexToBytes(operand as `0x${string}`)),
          sendCtx,
        ],
      })

      // 3) end_send(ibc_store, clock, t_height: u64, timeout_ns: u64, ctx)
      tx.moveCall({
        target: `${packageId}::${module}::end_send`,
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

import { ZkgmIncomingMessage } from "@unionlabs/sdk"
import type { Hex } from "@unionlabs/sdk/schema/hex"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Utils from "@unionlabs/sdk/Utils"
import * as Client from "@unionlabs/sdk/ZkgmClient"
import * as ClientError from "@unionlabs/sdk/ZkgmClientError"
import type * as ClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import { Brand, pipe, Predicate } from "effect"
import * as Effect from "effect/Effect"
import * as Inspectable from "effect/Inspectable"
import * as Stream from "effect/Stream"
import * as Evm from "../Evm.js"

const fromWallet = (
  opts: { client: Evm.Evm.PublicClient; wallet: Evm.Evm.WalletClient },
): Client.ZkgmClient =>
  Client.make((request, signal, fiber) =>
    Effect.gen(function*() {
      const {
        wallet,
        client,
      } = opts
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

      const sendInstruction = Evm.writeContract({
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
      emit.fromEffect(pipe(
        Evm.waitForTransactionReceipt(this.txHash),
        Effect.map((a) =>
          ZkgmIncomingMessage.LifecycleEvent.EvmTransactionReceiptComplete({
            transactionHash: a.transactionHash as `0x${string}` & Brand.Brand<"Hash">,
            blockHash: a.blockHash as `0x${string}` & Brand.Brand<"Hash">,
            gasUsed: a.gasUsed,
          })
        ),
        Effect.provideService(Evm.PublicClient, this.client),
      ))
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

class ClientResponseImpl extends IncomingMessageImpl<ClientError.ResponseError>
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

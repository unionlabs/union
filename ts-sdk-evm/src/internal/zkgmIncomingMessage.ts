import * as IncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import * as Inspectable from "effect/Inspectable"
import type * as Stream from "effect/Stream"

/** @internal */
export abstract class ZkgmIncomingMessageImpl<E> extends Inspectable.Class
  implements IncomingMessage.ZkgmIncomingMessage<E>
{
  readonly [IncomingMessage.TypeId]: IncomingMessage.TypeId

  constructor(
    readonly source: Http.IncomingMessage,
    readonly onError: (error: unknown) => E,
    readonly remoteAddressOverride?: string,
  ) {
    super()
    this[IncomingMessage.TypeId] = IncomingMessage.TypeId
  }

  get stream(): Stream.Stream<Uint8Array, E> {
    return NodeStream.fromReadable<E, Uint8Array>(
      () => this.source,
      this.onError,
    )
  }
}

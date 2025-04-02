import { Effect, Fiber, Option } from "effect"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries.ts"
import { transferPacketHashQuery } from "$lib/queries/packet-hash.ts"
import type { PacketHash } from "@unionlabs/sdk/schema"

export class TransferHashStore {
  hash: string = $state("")
  data: Option.Option<PacketHash> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError> = $state(Option.none())
  fiber: Option.Option<Fiber.RuntimeFiber<unknown, unknown>> = $state(Option.none())

  startPolling = (txHash: string) => {
    this.reset()
    this.hash = txHash
    const newFiber = Effect.runFork(
      transferPacketHashQuery({
        submission_tx_hash: txHash
      })
    )

    this.fiber = Option.some(newFiber)
  }

  reset = () => {
    if (this.fiber._tag === "Some") {
      Effect.runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }

    this.data = Option.none()
    this.error = Option.none()
    this.hash = ""
  }
}

export const transferHashStore = new TransferHashStore()

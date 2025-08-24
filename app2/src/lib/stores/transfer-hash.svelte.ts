import { transferPacketHashQuery } from "$lib/queries/packet-hash"
import { runFork, runPromise } from "$lib/runtime"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { PacketHash } from "@unionlabs/sdk/schema"
import { Effect, Fiber, Option } from "effect"
import type { TimeoutException } from "effect/Cause"

export class TransferHashStore {
  hash: string = $state("")
  data: Option.Option<PacketHash> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError | TimeoutException> = $state(Option.none())
  fiber: Option.Option<Fiber.RuntimeFiber<unknown, unknown>> = $state(Option.none())

  startPolling = (txHash: string) => {
    this.reset()
    this.hash = txHash
    const newFiber = runFork(
      transferPacketHashQuery({
        submission_tx_hash: txHash,
      }),
    )

    this.fiber = Option.some(newFiber)
  }

  reset = () => {
    if (Option.isSome(this.fiber)) {
      runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }

    this.data = Option.none()
    this.error = Option.none()
    this.hash = ""
  }

  stopPolling = () => {
    if (this.fiber._tag === "Some") {
      runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }
  }
}

export const transferHashStore = new TransferHashStore()

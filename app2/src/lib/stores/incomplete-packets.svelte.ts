import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Effect, Fiber, Option } from "effect"
import type { PacketCount, PacketList } from "@unionlabs/sdk/schema"

class IncompletePacketsListStore {
  data = $state(Option.none<typeof PacketList.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())

  async runEffect<R>(effect: Effect.Effect<R>) {
    this.data = Option.none()
    await this.interruptFiber()
    const fiber = Effect.runFork(effect)
    this.fiber = Option.some(fiber)
    return fiber
  }

  async clearFiber() {
    await this.interruptFiber()
    this.fiber = Option.none()
  }

  async interruptFiber() {
    if (Option.isSome(this.fiber)) {
      await Effect.runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }
  }
}

class IncompleteTransferCountStore {
  data = $state(Option.none<typeof PacketCount.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
}

export const incompletePacketsList = new IncompletePacketsListStore()
export const transferCount = new IncompleteTransferCountStore()

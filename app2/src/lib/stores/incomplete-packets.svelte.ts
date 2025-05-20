import { runFork, runPromise } from "$lib/runtime"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { PacketCount, PacketList } from "@unionlabs/sdk/schema"
import { Effect, Fiber, Option } from "effect"
import type { TimeoutException, UnknownException } from "effect/Cause"
import type { ParseError } from "effect/ParseResult"

class IncompletePacketsListStore {
  data = $state(Option.none<typeof PacketList.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError | ParseError | UnknownException>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, ParseError | UnknownException>>())

  async runEffect<A>(effect: Effect.Effect<A, ParseError | UnknownException>) {
    this.data = Option.none()
    await this.interruptFiber()
    const fiber = runFork(effect)
    this.fiber = Option.some(fiber)
    return fiber
  }

  async clearFiber() {
    await this.interruptFiber()
    this.fiber = Option.none()
  }

  async interruptFiber() {
    if (Option.isSome(this.fiber)) {
      await runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }
  }
}

class IncompleteTransferCountStore {
  data = $state(Option.none<typeof PacketCount.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError | TimeoutException>())
}

export const incompletePacketsList = new IncompletePacketsListStore()
export const transferCount = new IncompleteTransferCountStore()

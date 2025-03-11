import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Effect, Fiber, Option } from "effect"
import type { TransferList, TransferCount } from "$lib/schema/transfer-list"

class TransferListStore {
  data = $state(Option.none<typeof TransferList.Type>())
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

class TransferCountStore {
  data = $state(Option.none<typeof TransferCount.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
}

export const transferList = new TransferListStore()
export const transferListAddress = new TransferListStore()
export const transferCount = new TransferCountStore()

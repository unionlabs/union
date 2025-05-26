import { type AppContext, runFork, runPromise } from "$lib/runtime"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { TransferCount, TransferList } from "@unionlabs/sdk/schema"
import { Effect, Fiber, Option } from "effect"
import type { TimeoutException } from "effect/Cause"

class TransferListStore {
  data = $state(Option.none<typeof TransferList.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError | TimeoutException>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())

  async runEffect<A>(effect: Effect.Effect<A, never, AppContext>) {
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

class TransferCountStore {
  data = $state(Option.none<typeof TransferCount.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
}

export const transferList = new TransferListStore()
export const transferListAddress = new TransferListStore()
export const transferCount = new TransferCountStore()

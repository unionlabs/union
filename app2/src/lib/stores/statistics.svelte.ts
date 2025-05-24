import { type AppContext, runFork, runPromise } from "$lib/runtime"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { DailyTransfers, Statistics } from "@unionlabs/sdk/schema"
import { Effect, Fiber, Option } from "effect"
import type { TimeoutException } from "effect/Cause"

class StatisticsStore {
  data = $state(Option.none<Statistics>())
  error = $state(Option.none<FetchDecodeGraphqlError | TimeoutException>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())

  async runEffect<A>(effect: Effect.Effect<A, never, AppContext>) {
    this.data = Option.none()
    await this.interruptFiber()
    const fiber = runFork(effect)
    this.fiber = Option.some(fiber)
    return fiber
  }

  async interruptFiber() {
    if (Option.isSome(this.fiber)) {
      await runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }
  }
}

class DailyTransfersStore {
  data = $state(Option.none<DailyTransfers>())
  error = $state(Option.none<FetchDecodeGraphqlError | TimeoutException>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())

  async runEffect<A>(effect: Effect.Effect<A, never, AppContext>) {
    this.data = Option.none()
    await this.interruptFiber()
    const fiber = runFork(effect)
    this.fiber = Option.some(fiber)
    return fiber
  }

  async interruptFiber() {
    if (Option.isSome(this.fiber)) {
      await runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }
  }
}

export const statistics = new StatisticsStore()
export const dailyTransfers = new DailyTransfersStore()

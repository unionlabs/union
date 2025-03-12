import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Effect, Fiber, Option } from "effect"
import type { Statistics, DailyTransfers } from "$lib/schema/statistics"

class StatisticsStore {
  data = $state(Option.none<Statistics>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())

  async runEffect<R>(effect: Effect.Effect<R>) {
    this.data = Option.none()
    await this.interruptFiber()
    const fiber = Effect.runFork(effect)
    this.fiber = Option.some(fiber)
    return fiber
  }

  async interruptFiber() {
    if (Option.isSome(this.fiber)) {
      await Effect.runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }
  }
}

class DailyTransfersStore {
  data = $state(Option.none<DailyTransfers>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())

  async runEffect<R>(effect: Effect.Effect<R>) {
    this.data = Option.none()
    await this.interruptFiber()
    const fiber = Effect.runFork(effect)
    this.fiber = Option.some(fiber)
    return fiber
  }

  async interruptFiber() {
    if (Option.isSome(this.fiber)) {
      await Effect.runPromise(Fiber.interrupt(this.fiber.value))
      this.fiber = Option.none()
    }
  }
}

export const statistics = new StatisticsStore()
export const dailyTransfers = new DailyTransfersStore()

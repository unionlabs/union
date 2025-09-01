import { type AppContext, runFork, runPromise } from "$lib/runtime"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { DailyTransfers, Statistics } from "@unionlabs/sdk/schema"
import { Effect, Fiber, flow, Option, pipe } from "effect"
import * as A from "effect/Array"
import type { TimeoutException } from "effect/Cause"
import * as I from "effect/Iterable"

class StatisticsStore {
  data = $state(Option.none<Statistics>())
  error = $state(Option.none<FetchDecodeGraphqlError | TimeoutException>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())
  totalTransfers = $derived(pipe(
    this.data,
    Option.flatMap(flow(
      A.findFirst(x => x.name === "total_transfers"),
      Option.map(x => x.value),
    )),
  ))
  totalPackets = $derived(pipe(
    this.data,
    Option.flatMap(flow(
      A.findFirst(x => x.name === "total_packets"),
      Option.map(x => x.value),
    )),
  ))

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
  cumData = $derived(
    pipe(
      Option.all([this.data, statistics.totalTransfers]),
      Option.map(([xs, total]) => {
        const rev = A.reverse(xs)
        return pipe(
          rev,
          I.scan(total, (acc, { count }) => acc - count),
          I.zip(rev),
          I.map(([count, { day_date }]) => ({
            count,
            day_date,
          })),
          A.fromIterable,
          A.reverse,
          x => {
            console.log({ x })
            return x
          },
        )
      }),
    ),
  )

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

class DailyPacketsStore {
  data = $state(Option.none<DailyTransfers>())
  error = $state(Option.none<FetchDecodeGraphqlError | TimeoutException>())
  fiber = $state(Option.none<Fiber.RuntimeFiber<any, never>>())
  cumData = $derived(
    pipe(
      Option.all([this.data, statistics.totalPackets]),
      Option.map(([xs, total]) => {
        const rev = A.reverse(xs)
        return pipe(
          rev,
          I.scan(total, (acc, { count }) => acc - count),
          I.zip(rev),
          I.map(([count, { day_date }]) => ({
            count,
            day_date,
          })),
          A.fromIterable,
          A.reverse,
          x => {
            console.log({ x })
            return x
          },
        )
      }),
    ),
  )

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
export const dailyPackets = new DailyPacketsStore()

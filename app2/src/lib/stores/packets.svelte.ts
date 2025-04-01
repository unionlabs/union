import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Effect, Fiber, Option } from "effect"
import type { PacketList, PacketCount, PacketDetails } from "@unionlabs/sdk/schema"

class PacketListStore {
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

class PacketDetailsStore {
  data = $state(Option.none<PacketDetails>())
  error = $state(Option.none<FetchDecodeGraphqlError | { _tag: "NotFound"; message: string }>())
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

class PacketCountStore {
  data = $state(Option.none<typeof PacketCount.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
}

export const packetList = new PacketListStore()
export const packetDetails = new PacketDetailsStore()
export const packetCount = new PacketCountStore()

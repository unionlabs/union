import { Context, Effect, Layer, Logger, LogLevel, ManagedRuntime } from "effect"
import type { RunCallbackOptions, RunForkOptions } from "effect/Runtime"

// XXX: serice should live elsewhere; only for dev testing
export const A = Context.GenericTag<"A">("A")

// TODO: add mocking layer
const LayerLive = Layer.mergeAll(
  Layer.succeed(A, "A"),
  Logger.minimumLogLevel(LogLevel.Trace),
)

const managedRuntime = ManagedRuntime.make(LayerLive)

export const {
  runSync,
  runSyncExit,
  runPromise,
  runPromiseExit,
  runFork,
  runCallback,
  dispose,
} = managedRuntime

export const runtime = await managedRuntime.runtime()

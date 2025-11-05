import * as Runtime from "$lib/Runtime.svelte.js"
import {
  Cause,
  Context,
  Deferred,
  Effect,
  Either,
  Exit,
  Fiber,
  FiberId,
  FiberStatus,
  Layer,
  ManagedRuntime,
  Option,
  pipe,
  Runtime as EffectRuntime,
  RuntimeFlags,
  Stream,
} from "effect"
import { flushSync } from "svelte"
import { assert, describe, expect, it } from "vitest"

const runFork = Runtime.runForkWithRuntime(EffectRuntime.defaultRuntime)
const runPromiseExit = Runtime.runPromiseExitWithRuntime(EffectRuntime.defaultRuntime)

describe("Runtime", () => {
  const anyRuntimeFiberId = FiberId.runtime(expect.any(Number), expect.any(Number))

  describe("runPromiseExit", () => {
    it("current", async ({ onTestFinished }) => {
      const asyncEffect = () => Effect.promise(async () => 1)
      let result = $state<Runtime.Type.PromiseExit<typeof asyncEffect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runPromiseExit(asyncEffect)
      })
      assert.strictEqual(result?.current, Option.none())
      await expect.poll(() => result?.current).toEqual(Option.some(Exit.succeed(1)))
      onTestFinished(cleanup)
    })
    it("either", async ({ onTestFinished }) => {
      const asyncEffect = () => Effect.promise(async () => 1)
      let result = $state<Runtime.Type.PromiseExit<typeof asyncEffect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runPromiseExit(asyncEffect)
      })
      assert.strictEqual(result?.either, Option.none())
      await expect.poll(() => result?.either).toEqual(Option.some(Either.right(1)))
      onTestFinished(cleanup)
    })
    it("interrupts on cleanup", async () => {
      const latch = Deferred.unsafeMake<void>(FiberId.none)
      const asyncEffect = () =>
        pipe(
          Deferred.complete(latch, Effect.void),
          Effect.andThen(() => pipe(Effect.void, Effect.forever)),
        )
      let result = $state<Runtime.Type.PromiseExit<typeof asyncEffect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runPromiseExit(asyncEffect)
        return () => result?.interrupt()
      })
      await Effect.runPromise(Deferred.await(latch))
      cleanup()
      await expect
        .poll(() => result?.current)
        .toStrictEqual(
          expect.objectContaining(
            pipe(Cause.interrupt(anyRuntimeFiberId), Exit.failCause, Option.some),
          ),
        )
    })
    it("is interruptible", async ({ onTestFinished }) => {
      const latch = Deferred.unsafeMake<void>(FiberId.none)
      const asyncEffect = () =>
        pipe(
          Deferred.complete(latch, Effect.void),
          Effect.andThen(() => pipe(Effect.void, Effect.forever)),
        )
      let result = $state<Runtime.Type.PromiseExit<typeof asyncEffect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runPromiseExit(asyncEffect)
      })
      await Effect.runPromise(Deferred.await(latch))
      result?.interrupt()
      await expect
        .poll(() => result?.current)
        .toStrictEqual(
          expect.objectContaining(
            pipe(Cause.interrupt(anyRuntimeFiberId), Exit.failCause, Option.some),
          ),
        )
      onTestFinished(cleanup)
    })
    describe("onInterrupt", () => {
      it("ignore", async ({ onTestFinished }) => {
        let halt = $state(0)
        const effect = () =>
          Effect.if(halt === 1, {
            onFalse: () => Effect.succeed(halt),
            onTrue: () => Effect.interrupt,
          })
        let result = $state<Runtime.Type.PromiseExit<typeof effect> | null>(null)
        const cleanup = $effect.root(() => {
          result = runPromiseExit(effect, { onInterrupt: "ignore" })
        })
        assert.strictEqual(result?.current, Option.none())
        halt = 0
        await expect.poll(() => result?.current).toEqual(Option.some(Exit.succeed(0)))
        halt = 1
        await expect.poll(() => result?.current).toEqual(Option.some(Exit.succeed(0)))
        halt = 2
        await expect.poll(() => result?.current).toEqual(Option.some(Exit.succeed(2)))

        onTestFinished(cleanup)
      })
      it("none", async ({ onTestFinished }) => {
        let halt = $state(false)
        const effect = () =>
          Effect.if(halt, {
            onFalse: () => Effect.succeed(0),
            onTrue: () => Effect.interrupt,
          })
        let result = $state<Runtime.Type.PromiseExit<typeof effect> | null>(null)
        const cleanup = $effect.root(() => {
          result = runPromiseExit(effect, { onInterrupt: "none" })
        })
        assert.strictEqual(result?.current, Option.none())
        await expect.poll(() => result?.current).toEqual(Option.some(Exit.succeed(0)))
        halt = true
        await expect.poll(() => result?.current).toEqual(Option.none())

        onTestFinished(cleanup)
      })
      it("error", async ({ onTestFinished }) => {
        let halt = $state(false)
        const effect = () =>
          Effect.if(halt, {
            onFalse: () => Effect.succeed(0),
            onTrue: () => Effect.interrupt,
          })
        let result = $state<Runtime.Type.PromiseExit<typeof effect> | null>(null)
        const cleanup = $effect.root(() => {
          result = runPromiseExit(effect, { onInterrupt: "error" })
        })
        assert.strictEqual(result?.current, Option.none())
        await expect.poll(() => result?.current).toEqual(Option.some(Exit.succeed(0)))
        halt = true
        await expect
          .poll(() => result?.current)
          .toStrictEqual(
            expect.objectContaining(
              pipe(Cause.interrupt(anyRuntimeFiberId), Exit.failCause, Option.some),
            ),
          )

        onTestFinished(cleanup)
      })
    })
    it("accepts runtime", async ({ onTestFinished }) => {
      class A extends Context.Tag("MyService")<A, { readonly _: Effect.Effect<number> }>() {}
      const managedRuntime = ManagedRuntime.make(Layer.succeed(A, { _: Effect.succeed(1) }))
      const runtime = await managedRuntime.runtime()
      let result = $state<number>(0)
      const cleanup = $effect.root(() => {
        Runtime.runForkWithRuntime(
          runtime,
          () =>
            pipe(
              Effect.andThen(A, ({ _ }) => _),
              Effect.flatMap(x =>
                Effect.sync(() => {
                  result = x
                })
              ),
            ),
        )
      })
      expect(result).toStrictEqual(0)
      await expect.poll(() => result).toStrictEqual(1)
      onTestFinished(cleanup)
    })
  })

  describe("runFork", () => {
    it("provisions fiber reference", ({ onTestFinished }) => {
      const effect = () => Effect.void.pipe(Effect.forever)
      let result = $state<Runtime.Type.RunFork<typeof effect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runFork(effect)
      })
      flushSync()
      expect(Fiber.isRuntimeFiber(result!.fiber)).toBe(true)
      onTestFinished(cleanup)
    })

    it("can mutate $state rune", async ({ onTestFinished }) => {
      let count = $state(0)
      const latch = Deferred.unsafeMake<void>(FiberId.none)
      const cleanup = $effect.root(() => {
        runFork(
          () =>
            pipe(
              Stream.repeatValue(0),
              Stream.take(3),
              Stream.mapEffect(() =>
                Effect.sync(() => {
                  count += 1
                })
              ),
              Stream.onDone(() => Deferred.succeed(latch, undefined)),
              Stream.runDrain,
            ),
        )
      })
      expect(count).toBe(0)
      await Effect.runPromise(Deferred.await(latch))
      expect(count).toBe(3)
      onTestFinished(cleanup)
    })

    it("cleans up fork fiber", async () => {
      const effect = () => Effect.forever(Effect.void)
      let result = $state<Runtime.Type.RunFork<typeof effect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runFork(effect)
      })
      flushSync()
      const running = await Effect.runPromise(result!.fiber.status)
      expect(running).toStrictEqual(
        FiberStatus.running(
          RuntimeFlags.make(
            RuntimeFlags.CooperativeYielding,
            RuntimeFlags.Interruption,
            RuntimeFlags.RuntimeMetrics,
          ),
        ),
      )
      cleanup()
      await expect
        .poll(async () => Effect.runPromise(result!.fiber.status))
        .toStrictEqual(FiberStatus.done)
    })
  })
})

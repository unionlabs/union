import * as Machine from "@effect/experimental/Machine"
import * as Staking from "@unionlabs/sdk/Staking.js"
import { Cause, Chunk, Context, Deferred, Effect, Exit, Layer, Schema as S, Stream } from "effect"

class Initialize extends S.TaggedRequest<Increment>()("Increment", {
  failure: S.Union(
    Staking.GetParams.failure,
  ),
  success: S.Struct({
    step: S.Literal(0),
    params: Staking.GetParams.success,
  }),
  payload: {},
}) {}

class Increment extends S.TaggedRequest<Increment>()("Increment", {
  failure: S.Never,
  success: S.Number,
  payload: {},
}) {}
class Decrement extends S.TaggedRequest<Decrement>()("Decrement", {
  failure: S.Never,
  success: S.Number,
  payload: {},
}) {}
class IncrementBy extends S.TaggedRequest<IncrementBy>()("IncrementBy", {
  failure: S.Never,
  success: S.Number,
  payload: {
    number: S.Number,
  },
}) {}
class DelayedIncrementBy extends S.TaggedRequest<DelayedIncrementBy>()("DelayedIncrementBy", {
  failure: S.Never,
  success: S.Void,
  payload: {
    delay: S.Positive,
    number: S.Number,
  },
}) {}
class Multiply extends S.TaggedRequest<Multiply>()("Multiply", {
  failure: S.Never,
  success: S.Number,
  payload: {},
}) {}

class FailBackground extends S.TaggedRequest<FailBackground>()("FailBackground", {
  failure: S.Never,
  success: S.Void,
  payload: {},
}) {}

const counter = Machine.makeWith<number, number>()(
  (input, previous) =>
    Machine.procedures.make(previous ?? input, {
      identifier: `Counter(${input})`,
    }).pipe(
      Machine.procedures.add<Increment>()("Increment", ({ state }) =>
        Effect.sync(() => {
          const count = state + 1
          return [count, count]
        })),
      Machine.procedures.add<Decrement>()("Decrement", ({ state }) =>
        Effect.sync(() => {
          const count = state - 1
          return [count, count]
        })),
      Machine.procedures.add<IncrementBy>()("IncrementBy", ({ request, state }) =>
        Effect.sync(() => {
          const count = state + request.number
          return [count, count]
        })),
      Machine.procedures.add<FailBackground>()(
        "FailBackground",
        ({ forkWith, state }) => forkWith(Effect.fail("error"), state),
      ),
    ),
)

const counterSerializable = Machine.makeSerializable(
  { state: S.NumberFromString, input: S.Number },
  (input, previous) =>
    Machine.serializable.make(previous ?? input, {
      identifier: `Counter(${input})`,
    }).pipe(
      Machine.serializable.add(Increment, ({ state }) =>
        Effect.sync(() => {
          const count = state + 1
          return [count, count]
        })),
      Machine.serializable.add(Decrement, ({ state }) =>
        Effect.sync(() => {
          const count = state - 1
          return [count, count]
        })),
      Machine.serializable.add(IncrementBy, ({ request, state }) =>
        Effect.sync(() => {
          const count = state + request.number
          return [count, count]
        })),
      Machine.serializable.add(
        FailBackground,
        ({ forkWith, state }) => forkWith(Effect.fail("error"), state),
      ),
    ),
)

const delayedCounter = Machine.makeWith<number, number>()(
  (input, previous) =>
    Machine.procedures.make(previous ?? input, {
      identifier: `Counter(${input})`,
    }).pipe(
      Machine.procedures.addPrivate<IncrementBy>()("IncrementBy", ({ request, state }) =>
        Effect.sync(() => {
          const count = state + request.number
          return [count, count]
        })),
      Machine.procedures.add<DelayedIncrementBy>()(
        "DelayedIncrementBy",
        ({ forkWith, request, sendAwait, state }) =>
          sendAwait(new IncrementBy({ number: request.number })).pipe(
            Effect.delay(request.delay),
            forkWith(state),
          ),
      ),
    ),
)

class Multiplier extends Context.Tag("Multiplier")<Multiplier, number>() {
  static Live = Layer.succeed(this, 2)
}

const withContext = Machine.make(
  (input: number, previous?: number) =>
    Effect.gen(function*(_) {
      const multiplier = yield* _(Multiplier)
      return Machine.procedures.make(previous ?? input).pipe(
        Machine.procedures.add<Multiply>()("Multiply", ({ state }) =>
          Effect.sync(() => {
            const count = state * multiplier
            return [count, count]
          })),
      )
    }),
)

const timerLoop = Machine.make(
  Effect.gen(function*(_) {
    const { unsafeSend } = yield* _(Machine.MachineContext)

    // queue initial message
    yield* _(unsafeSend(new Increment()))

    return Machine.procedures.make(0).pipe(
      Machine.procedures.addPrivate<Increment>()(
        "Increment",
        (ctx) =>
          ctx.send(new Increment()).pipe(
            Effect.delay(20),
            ctx.forkOne("timer"),
            Effect.as([ctx.state + 1, ctx.state + 1]),
          ),
      ),
    )
  }),
)

const deferReply = Machine.make(
  Machine.procedures.make(0).pipe(
    Machine.procedures.add<Increment>()(
      "Increment",
      (ctx) => {
        const count = ctx.state + 1
        return Deferred.succeed(ctx.deferred, count).pipe(
          Effect.delay(10),
          ctx.fork,
          Effect.as([Machine.NoReply, count]),
        )
      },
    ),
  ),
)

import { HttpClient } from "@effect/platform"
import { Context, Effect, ExecutionPlan, Layer, pipe } from "effect"
import { constTrue } from "effect/Function"

export class Favicon extends Context.Tag("Favicon")<Favicon, {
  of: (url: URL) => Effect.Effect<URL, string>
}>() {}

const DDG = Layer.effect(
  Favicon,
  Effect.gen(function*() {
    const client = yield* HttpClient.HttpClient.pipe(
      Effect.map(HttpClient.filterStatusOk),
      Effect.map(HttpClient.withTracerDisabledWhen(constTrue)),
    )

    const of = Effect.fn("of")((url: URL) =>
      pipe(
        url.hostname,
        (hostname) => `https://icons.duckduckgo.com/ip3/${hostname}.ico`,
        (url) => client.get(url),
        Effect.map(x => x.remoteAddress as unknown as URL),
        Effect.mapError((e) => e.message),
      )
    )

    return Favicon.of({
      of,
    })
  }),
)

const GoogleS2 = Layer.effect(
  Favicon,
  Effect.gen(function*() {
    const client = yield* HttpClient.HttpClient.pipe(
      Effect.map(HttpClient.filterStatusOk),
      Effect.map(HttpClient.withTracerDisabledWhen(constTrue)),
    )

    const of = Effect.fn("of")((url: URL) =>
      pipe(
        url.hostname,
        (hostname) => `https://www.google.com/s2/favicons?domain=${hostname}&sz=256`,
        (url) => client.get(url),
        Effect.map(x => x.remoteAddress as unknown as URL),
        Effect.mapError((e) => e.message),
      )
    )

    return Favicon.of({
      of,
    })
  }),
)

export const Plan = ExecutionPlan.make(
  {
    provide: GoogleS2,
    attempts: 1,
  },
  {
    provide: DDG,
    attempts: 1,
  },
)

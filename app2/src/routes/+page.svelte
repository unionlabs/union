<script lang="ts">
import { Console, Context, Option, Effect, Fiber, Schema, pipe, Random, Schedule } from "effect"
import { FetchHttpClient, HttpClient } from "@effect/platform"
import { onMount } from "svelte"
import { decodeUnknown } from "effect/Duration"

const Block = Schema.Struct({
  result: Schema.Struct({
    block_id: Schema.Struct({
      hash: Schema.String
    })
  })
})

const BlockWeird = Schema.Struct({
  resulty: Schema.Struct({
    block_id: Schema.Struct({
      hash: Schema.String
    })
  })
})

let responseData: Option.Option<typeof Block.Type> = $state(Option.none())
let responseError: Option.Option<unknown> = $state(Option.none())

const createQuery = <S>(
  url: string,
  schema: Schema.Schema<S>,
  writeStore: (value: Option.Option<S>) => void
) => {
  const fetcher = Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    yield* Effect.log("fetching data")
    const r = yield* Random.next
    const response = yield* client.get(r > 0.3 ? url : "https://rpc.testnet-9.union.build/genesis")
    const json = yield* response.json
    yield* Effect.log("fetched data")

    const block = yield* Schema.decodeUnknown(schema)(json)
    return block
  })

  const fetcherPipeline = pipe(
    fetcher,
    Effect.tapBoth({
      onSuccess: data =>
        Effect.sync(() => {
          writeStore(Option.some(data))
          responseError = Option.none()
        }),
      onFailure: error =>
        Effect.sync(() => {
          responseError = Option.some(error)
        })
    }),
    Effect.catchAll(_ => Effect.succeed(null)),
    Effect.scoped,
    Effect.provide(FetchHttpClient.layer)
  )

  const program = Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => "2 seconds")
  )
  return program
}

const program = createQuery("https://rpc.testnet-9.union.build/block", Block, data => {
  responseData = data
})

let fiber

const stop = Effect.gen(function* () {
  yield* Fiber.interrupt(fiber)
  responseData = Option.none()
})
onMount(() => {
  fiber = Effect.runFork(program)
  return () => Effect.runPromise(stop)
})
</script>

<button class="bg-red-500" onclick={() => {Effect.runPromise(stop)}}> stop the fetcher </button>


{#if Option.isSome(responseData)}
  <pre class="font-mono">
    {JSON.stringify(responseData.value, null, 2)}
  </pre>
{:else}
  Loading...
{/if}
{#if Option.isSome(responseError)}
  <pre class="font-mono bg-red-500">
    {JSON.stringify(responseError.value, null, 2)}
  </pre>
{/if}

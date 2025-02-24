<script lang="ts">
import { Console, Context, Option, Effect, Fiber, pipe, Random, Schedule } from "effect"
import { FetchHttpClient, HttpClient } from "@effect/platform"
import { onMount } from "svelte"

let responseData: Option.Option<unknown> = $state(Option.none())
let responseError: Option.Option<unknown> = $state(Option.none())

class SvelteStore extends Context.Tag("SvelteStore")<
  SvelteStore,
  { readonly next: Effect.Effect<number> }
>() {}

const fetcher = Effect.gen(function* () {
  const client = yield* HttpClient.HttpClient
  yield* Effect.log("fetching data")
  const r = yield* Random.next
  const response = yield* client.get(
    r > 0.3 ? "https://rpc.testnet-9.union.build/block" : "https://thisisnotavalidurl.com"
  )
  const json = yield* response.json
  yield* Effect.log("fetched data")
  return json
})

const fetcherPipeline = pipe(
  fetcher,
  Effect.tapBoth({
    onSuccess: data =>
      Effect.sync(() => {
        responseData = Option.some(data)
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
    {JSON.stringify(responseData.value.result.block_id.hash, null, 2)}
  </pre>
{:else}
  Loading... yeah wahooo
{/if}
{#if Option.isSome(responseError)}
  <pre class="font-mono bg-red-500">
    {JSON.stringify(responseError.value, null, 2)}
  </pre>
{/if}

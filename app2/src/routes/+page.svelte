<script lang="ts">
import { Effect, Fiber, Random, Schedule } from "effect"
import { FetchHttpClient, HttpClient } from "@effect/platform"

let posts: "loading" | unknown = $state("loading")

const fetcher = Effect.gen(function* () {
  const client = yield* HttpClient.HttpClient
  yield* Effect.log("fetching data")
  // yield* Effect.sleep("3 seconds")
  const r = yield* Random.next
  const response = yield* client.get(
    r > 0.3 ? "https://rpc.testnet-9.union.build/block" : "thisisnotavalidurl"
  )
  const json = yield* response.json
  yield* Effect.log("fetched data")

  posts = json

  return response
}).pipe(
  Effect.catchAll(error => Effect.logError(error.message)),
  Effect.scoped,
  Effect.provide(FetchHttpClient.layer)
)

const program = Effect.repeat(
  fetcher,
  Schedule.addDelay(Schedule.repeatForever, () => "2 seconds")
)

const fiber = Effect.runFork(program)
Effect.runPromiseExit(fiber)
</script>

<button class="bg-red-500" onclick={() => {Effect.runPromise(Fiber.interrupt(fiber))}}> stop the fetcher </button>

<pre class="font-mono">
  {JSON.stringify(posts, null, 2)}
</pre>

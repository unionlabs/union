<script lang="ts">
import { onMount } from "svelte"
import { decodeUnknown } from "effect/Duration"
import { createQuery } from "$lib/create-query"
import { Effect, Fiber, Option, Schema } from "effect"
import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { ParseError } from "effect/ParseResult"

const Block = Schema.Struct({
  result: Schema.Struct({
    block_id: Schema.Struct({
      hash: Schema.String
    })
  })
})

let blockData: Option.Option<typeof Block.Type> = $state(Option.none())
let blockError: Option.Option<HttpClientError | ParseError> = $state(Option.none())

const program = createQuery({
  url: "https://rpc.testnet-9.union.build/block",
  schema: Block,
  refetchInterval: "4 seconds",
  writeData: data => {
    blockData = data
  },
  writeError: error => {
    blockError = error
  }
})

let fiber

const stop = Effect.gen(function* () {
  yield* Fiber.interrupt(fiber)
  blockData = Option.none()
})
onMount(() => {
  fiber = Effect.runFork(program)
  return () => Effect.runPromise(stop)
})
</script>

<button class="bg-red-500" onclick={() => {Effect.runPromise(stop)}}> stop the fetcher </button>


{#if Option.isSome(blockData)}
  <pre class="font-mono">
    {JSON.stringify(blockData.value, null, 2)}
  </pre>
  {#if Option.isSome(blockError)}
    There was an error refetching this data. the data you see is stale
    <pre class="font-mono bg-red-500">
      {JSON.stringify(blockError.value, null, 2)}
    </pre>
  {/if}
{:else}
  Loading...
  {#if Option.isSome(blockError)}
    There was an error fetching this data. We will retry automatically. Error:
    <pre class="font-mono bg-red-500">
      {JSON.stringify(blockError.value, null, 2)}
    </pre>
  {/if}
{/if}

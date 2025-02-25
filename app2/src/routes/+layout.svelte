<script lang="ts">
import "../app.css"
import { onMount } from "svelte"
import { createQuery } from "$lib/utils/queries"
import { Effect, Fiber, Option } from "effect"
import { Block } from "$lib/schemas/block"
import { block } from "$lib/stores/block.svelte"

let { children } = $props()

const blockQuery = createQuery({
  url: "https://rpc.testnet-9.union.build/block",
  schema: Block,
  refetchInterval: "4 seconds",
  writeData: data => {
    block.data = data
  },
  writeError: error => {
    block.error = error
  }
})

onMount(() => {
  const fiber = Effect.runFork(blockQuery)
  return () =>
    Effect.runPromise(
      Effect.gen(function* () {
        yield* Fiber.interrupt(fiber)
        block.data = Option.none()
      })
    )
})
</script>

{@render children()}

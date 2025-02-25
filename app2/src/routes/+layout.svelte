<script lang="ts">
import "../app.css"
import { onMount } from "svelte"
import { Effect, Fiber, Option } from "effect"
import { block } from "$lib/stores/block.svelte"
import { chainsQuery } from "$lib/queries/chains.svelte"

let { children } = $props()

onMount(() => {
  const fiber = Effect.runFork(chainsQuery)
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

<script lang="ts">
import CurrentBlockHash from "$lib/components/current-block-hash.svelte"
import { chainsQueryDocument } from "$lib/queries/chain"
import { fetchDecodeGraphql } from "$lib/utils/queries"
import { Effect, pipe } from "effect"
import { onMount } from "svelte"

let query = pipe(
  fetchDecodeGraphql(chainsQueryDocument),
  Effect.tapBoth({
    onSuccess: data => Effect.log("yay", data),
    onFailure: error => Effect.log("ohno", error)
  })
)

onMount(() => {
  const fiber = Effect.runFork(query)
  // return () =>
  //   Effect.runPromise(
  //     Effect.gen(function* () {
  //       yield* Fiber.interrupt(fiber)
  //       block.data = Option.none()
  //     })
  //   )
})
</script>

<CurrentBlockHash/>
<CurrentBlockHash/>


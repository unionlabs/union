<script lang="ts">
import CurrentBlockHash from "$lib/components/current-block-hash.svelte"
import { chainsQueryDocument } from "$lib/queries/chain"
import { Chain } from "$lib/schemas/chain"
import { fetchDecodeGraphql } from "$lib/utils/queries"
import { Effect, pipe, Schema } from "effect"
import { onMount } from "svelte"

let query = pipe(
  fetchDecodeGraphql(
    Schema.Struct({ v1_ibc_union_chains: Schema.Array(Chain) }),
    chainsQueryDocument
  ),
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


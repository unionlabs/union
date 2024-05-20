<script lang="ts">
import { Shine } from "svelte-ux"
import { getContextClient, subscriptionStore } from "@urql/svelte"
import { cosmosBlocksSubscription } from "$lib/graphql/documents/cosmos-blocks.ts"

$: cosmosBlocks = subscriptionStore({
  client: getContextClient(),
  query: cosmosBlocksSubscription,
  variables: {
    limit: 15
  }
})

$: blocksData = $cosmosBlocks?.data?.v0_blocks ?? []
</script>

<main class="mt-16 flex size-full min-size-full flex-col items-center justify-center">
  <Shine depth={4} lightColor="#a0ecfd">
    <h1
      class="~sm/md:~text-9xl/11xl font-black leading-[9rem] brightness-75 cursor-default select-none text-center"
    >
      zkGM
    </h1>
  </Shine>
  {#each blocksData as block}
    <pre>{JSON.stringify(block, undefined, 2)}</pre>
  {/each}
</main>

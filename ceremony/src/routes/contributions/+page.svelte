<script lang="ts">
import { getContributions } from "$lib/supabase"
import Spinner from "$lib/components/Spinner.svelte"
import Text from "$lib/components/typography/Text.svelte"

let intervalId: NodeJS.Timeout | number
let contributions = $state()

async function loadContributions() {
  contributions = await getContributions()
}

$effect(() => {
  loadContributions()
  intervalId = setInterval(loadContributions, 1000 * 5)

  return () => {
    if (intervalId) clearInterval(intervalId)
  }
})

function getFirstLetter(str: string): string | undefined {
  return str.length > 0 ? str[0] : undefined
}
</script>

{#if contributions}
  <div class="flex flex-col items-center pb-24 pt-36 w-full overflow-y-scroll">
    <div class="w-full h-48 bg-gradient-to-b via-black from-black to-transparent absolute top-0"></div>
    <div class="flex flex-col items-stretch max-w-sm">
      <div class="text-white flex gap-2 items-center border-white border px-3 py-2">
        <Spinner class="size-5 text-union-accent-500"/>
        <span>Next contribution...</span>
      </div>
      {@render ContributionConnector()}
      {#each contributions as contribution, index }
        {@render ContributionBox(contribution)}
      {@render ContributionConnector()}
      {/each}
      {@render ContributionBox({ user_name: "Genesis", payload_id: "00000000-0000-0000-0000-000000000000"})}
    </div>
  </div>

{:else}
  <Spinner class="size-5 text-union-accent-500"/>
{/if}


{#snippet ContributionBox(contribution: unknown)}
  <a href="/contributions/{contribution.public_key_hash}"
     class="text-white flex gap-2 items-center border-white border p-2 pr-3  w-full">
      <div class="flex size-7 bg-union-accent-500 items-center justify-center text-black">{getFirstLetter(contribution.user_name)}</div>
    <Text class="uppercase text-sm font-mono">{contribution.payload_id}</Text>
  </a>
{/snippet}

{#snippet ContributionConnector()}
  <div class="h-8 w-[1px] bg-white self-center"></div>
{/snippet}

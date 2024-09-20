<script lang="ts">
  import { getContributions } from "$lib/supabase"
  import Spinner from "$lib/components/Spinner.svelte"
  import Text from "$lib/components/typography/Text.svelte"
  import Blink from "$lib/components/Blink.svelte"
  import H4 from "$lib/components/typography/H4.svelte"

  let intervalId: NodeJS.Timeout | number
  let contributions = $state()

  async function loadContributions() {
    contributions = await getContributions();
  }

  $effect(() => {
    loadContributions()
    intervalId = setInterval(loadContributions, 1000 * 5)

    return () => {
      if (intervalId) clearInterval(intervalId)
    }
  })
</script>

{#if contributions}
  <div class="flex flex-col-reverse items-center h-svh overflow-y-auto pb-24 pt-36 w-full">
    <div class="w-full h-48 bg-gradient-to-b via-black from-black to-transparent absolute top-0"></div>
    <div class="flex flex-col items-center max-w-sm">
      {#each contributions as contribution, index }
        {#if index !== 0}
          <div class="h-8 w-[2px] bg-white"></div>
        {/if}
        <a href="/contributions/{contribution.public_key_hash}"
           class="text-white flex gap-1 items-center border-white border px-3 py-2 w-full">
          <img class="size-7" src={contribution.avatar_url} alt="">
          <Text class="uppercase max-w-48 truncate">{contribution.user_name}</Text>
        </a>
      {/each}
      <div class="h-8 w-[2px] bg-white"></div>
      <div class="text-white flex gap-2 items-center border-white border px-3 py-2 mb-16">
        <Spinner class="size-5 text-union-accent-500"/>
        <span>Next contribution...</span>
      </div>
      <H4>
        <Blink/>
      </H4>
    </div>
  </div>
{:else}
  <Spinner class="size-5 text-union-accent-500"/>
{/if}
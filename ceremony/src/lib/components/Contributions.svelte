<script lang="ts">
  import Spinner from "$lib/components/Spinner.svelte"
  import H4 from "$lib/components/typography/H4.svelte"
  import Blink from "$lib/components/Blink.svelte"
  import Text from "$lib/components/typography/Text.svelte"
  import {getContributions} from "$lib/supabase"

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
</script>
{#if contributions}

  <div class="flex flex-col items-center h-svh overflow-y-auto pb-24 pt-36 w-full">
    <div class="w-full h-48 bg-gradient-to-b via-black from-black to-transparent absolute top-0"></div>
    <div class="flex flex-col items-center max-w-md">
      <div class="rounded-full border-[2px] h-8 w-8"></div>
      <div class="h-24 w-[2px] bg-white"></div>
      {#each contributions as contribution, index }
        <a href="/contributions?hash={contribution.public_key_hash}" class="flex items-center gap-4 w-full">

          <Text>{(index + 1) * 10}M</Text>
          <div class="text-white flex gap-1 items-center border-white border px-3 py-2 w-full">
            <img class="size-7" src={contribution.avatar_url} alt="">
            <Text class="uppercase max-w-48 truncate">{contribution.user_name}</Text>
          </div>
        </a>
        {#if index !== contributions.length - 1}
          <div class="h-12 w-[2px] bg-white"></div>
        {/if}
      {/each}
    </div>
  </div>
{:else}
  <Spinner class="size-5 text-union-accent-500"/>
{/if}
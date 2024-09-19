<script lang="ts">
import Blink from "$lib/components/Blink.svelte"
import Install from "$lib/components/Install.svelte"
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import type { ContributorState } from "$lib/stores/state.svelte.ts"

type Props = {
  contributor: ContributorState
}
let { contributor }: Props = $props()
</script>

{#if contributor}
  <div class="flex flex-col items-center  text-center mb-4">
    <H1 class="mb-4 text-7xl">
      <Blink loading={contributor.state === 'contributing'} sleep={contributor.clientState === 'offline'}/>
    </H1>
    <H1 class="mb-4 text-6xl">{contributor.clientState}</H1>
    {#if contributor.clientState === 'offline'}
      <Install/>
      <Text class="mt-4">*You must be running the Ceremony Client to be able to contribute.</Text>
    {/if}
  </div>
{/if}
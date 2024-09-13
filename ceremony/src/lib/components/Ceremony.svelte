<script lang="ts">
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import H3 from "$lib/components/typography/H3.svelte"
import H2 from "$lib/components/typography/H2.svelte"
import Install from "$lib/components/Install.svelte"
import { start } from "$lib/client"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

$effect(() => {
  console.log(contributor.state)
  if (contributor?.state === "contribute") {
    start()
  }
})
</script>

<div class="p-8 bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center justify-center flex-col h-48">

  {#if contributor.state === 'inQueue'}
    <H1>Your position: <span class="text-union-accent-500">{contributor.queueState.position}</span></H1>
    <H2>Queue length: <span class="text-union-accent-500">{contributor.queueState.count}</span></H2>
    <H3>Waiting time: <span class="text-union-accent-500">{contributor.queueState.estimatedTime} minutes</span> (est.).</H3>

    {#if contributor.clientState === 'offline'}
      <Install />
    {/if}

  {:else if contributor.state === 'contribute'}
    <H1>Starting contribution...</H1>

  {:else if contributor.state === 'contributing'}
    <H1>Contributing...</H1>

  {:else if contributor.state === 'verifying'}
    <H1>Verifying your contribution...</H1>

  {:else if contributor.state === 'contributed'}
    <H1>Thank you! Your contribution is completed.</H1>

  {:else if contributor.state === 'noClient'}
    <H1>No client. Cannot start contribution.</H1>
    <Install />

  {:else}
    <H1>Loading...</H1>
  {/if}


  
</div>
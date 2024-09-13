<script lang="ts">
  import type {ContributorState} from "$lib/stores/state.svelte.ts";
  import H1 from "$lib/components/typography/H1.svelte";
  import H3 from "$lib/components/typography/H3.svelte";
  import H2 from "$lib/components/typography/H2.svelte";
  import Install from "$lib/components/Install.svelte";
  import { start } from "$lib/client";

  type Props = {
    contributor: ContributorState;
  };

  let {contributor}: Props = $props();

  $effect(() => {
    if (contributor?.state === 'contribute') {
      start()
    }
  })

</script>

<div class="p-8 bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center justify-center flex-col h-48">

  {#if contributor.state === 'inQueue'}
    <H1>Your position: {contributor.queueState.position}</H1>
    <H2>Queue length: {contributor.queueState.count}</H2>
    <H3>{contributor.queueState.estimatedTime} minutes left (est.).</H3>

  {:else if contributor.state === 'contribute'}
    <H1>Starting client...</H1>

  {:else if contributor.state === 'contributing'}
    <H1>Contributing now...</H1>

  {:else if contributor.state === 'verifying'}
    <H1>Verifying your contribution...</H1>

  {:else if contributor.state === 'contributed'}
    <H1>Thank you! Your contribution is completed.</H1>

  {:else if contributor.state === 'noClient'}
    <H1>No client. Cannot start contribution.</H1>

  {:else}
    <H1>Loading...</H1>
  {/if}

  {#if contributor.clientState === 'offline'}
    <Install />
  {/if}
  
</div>
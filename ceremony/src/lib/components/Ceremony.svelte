<script lang="ts">
  import type {ContributorState} from "$lib/stores/state.svelte.ts";
  import H1 from "$lib/components/typography/H1.svelte";
  import H4 from "$lib/components/typography/H4.svelte";

  type Props = {
    contributor: ContributorState;
  };

  let {contributor}: Props = $props();

  $effect(() => {
    console.log(contributor.state)
    if (contributor?.contributionState === 'contribute') {

    }
  })

</script>


<div class="p-8 bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center justify-center flex-col h-48">

  {#if contributor.state === 'inQueue'}
    <H1>{contributor.ctx.queuePosition} / {contributor.ctx.queueCount}</H1>
    <H4>{contributor.ctx.estimatedTime} minutes left (est.).</H4>
  {/if}


</div>
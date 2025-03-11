<script lang="ts">
  import {Option} from "effect";
  import {chains} from "$lib/stores/chains.svelte.ts";
  import {cn} from "$lib/utils";
  import {tokensStore} from "$lib/stores/tokens.svelte.ts";
  import Label from "$lib/components/ui/Label.svelte";
  import {fade, fly} from "svelte/transition";
  import {transfer} from "$lib/components/Transfer/transfer.svelte.ts";

  type Props = {
    type: "source" | "destination";
  };

  let {type}: Props = $props();
  let open = $state(false);

</script>

{#if open}
  <div class="absolute inset-0 dark:border-zinc-700 dark:bg-zinc-900 z-30 p-4" transition:fade="{{ duration: 500 }}">
    <div class="flex flex-col items-start w-full gap-4"
         transition:fly="{{ y: 30, duration: 500, delay: 100, opacity: 0 }}">
      <h2>Select {type} chain</h2>
      <div class="grid grid-cols-3 gap-2 w-full">
        {#if Option.isSome(chains.data)}
          {#each chains.data.value as chain}
            <button
                    class="border border-white bg-white/15 rounded p-2 text-sm hover:bg-white/30 cursor-pointer"
                    onclick={() => {
                transfer.raw.updateField(type, chain.chain_id);
                tokensStore.fetchTokens(chain.universal_chain_id);
                open = !open;
              }}
            >
              {chain.display_name.split(" ")[0]}
            </button>
          {/each}
        {:else}
          Loading...
        {/if}
      </div>
    </div>
  </div>
{/if}
<div>
  <Label>{type}</Label>
  <button
          onclick={() => (open = !open)}
          class={cn(
      "w-full p-2 rounded-lg border border-zinc-600 bg-zinc-700 text-zinc-200",
      "hover:bg-zinc-600 hover:border-zinc-500",
      "focus:outline-none focus:ring-2 focus:ring-sky-500",
      "disabled:opacity-50 disabled:cursor-not-allowed",
      "transition-all duration-200 cursor-pointer"
    )}
  >
    {#if type === "source" && transfer.raw.source && !transfer.sourceChain}
    <span class="flex items-center justify-center">
      <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      Loading...
    </span>
    {:else if type === "destination" && transfer.raw.destination && !transfer.destinationChain}
    <span class="flex items-center justify-center">
      <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      Loading...
    </span>
    {:else if type === "source"}
      {transfer.sourceChain ? transfer.sourceChain.display_name : `Select ${type} chain`}
    {:else}
      {transfer.destinationChain ? transfer.destinationChain.display_name : `Select ${type} chain`}
    {/if}
  </button>
</div>

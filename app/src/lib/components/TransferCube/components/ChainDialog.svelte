<script lang="ts">
import type { Readable } from "svelte/store"
import type { ContextStore } from "$lib/components/TransferCube/transfer/context.ts"

interface Props {
  context: Readable<ContextStore>
  kind: "source" | "destination"
  dialogOpen: boolean
  onChainSelect: (type: "source" | "destination", chain: string) => void
  onClose: () => void
}

export let context: Props["context"]
export let kind: Props["kind"]
export let dialogOpen: Props["dialogOpen"]
export let onChainSelect: Props["onChainSelect"]
export let onClose: Props["onClose"]
</script>

{#if dialogOpen && $context?.chains}
  <dialog
          open
          aria-label={`Select ${kind} chain`}
          class="absolute z-50 inset-0 overflow-y-scroll p-0 bg-transparent m-0 w-full h-full animate-fade-in backdrop-blur-md"
  >
    <button
            type="button"
            class="fixed inset-0 w-full h-full bg-gradient-to-t from-black to-black/10 animate-fade-in"
            on:click|self={onClose}
            aria-label="Close dialog"
    />

    <div class="relative z-10">
      <div class="flex justify-center">
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4 md:gap-8 max-w-4xl py-8 px-4">
          {#each $context.chains as chain, i}
            <button
                    style="animation-delay: {i * 100}ms"
                    on:click={() => onChainSelect(kind, chain.chain_id)}
                    type="button"
                    class="h-72 flex items-end border p-4 bg-secondary group hover:bg-accent transition-colors animate-slide-up"
            >
              <span class="font-supermolot uppercase font-bold text-xl text-start text-secondary-foreground group-hover:text-secondary">
                {chain.display_name}
              </span>
            </button>
          {/each}
        </div>
      </div>
    </div>
  </dialog>
{/if}

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slide-up {
    from {
      transform: translateY(30px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  :global(.animate-fade-in) {
    animation: fade-in 0.3s ease-out forwards;
  }

  :global(.animate-slide-up) {
    animation: slide-up 0.4s ease-out forwards;
    opacity: 0;
  }
</style>
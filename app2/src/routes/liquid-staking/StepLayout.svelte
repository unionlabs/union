<script lang="ts">
    import type { Snippet } from "svelte"
    
    interface Props {
      left: Snippet
      right: Snippet
      children?: Snippet
    }
    
    let { left, right, children }: Props = $props()
    </script>
    
    <div class="grid grid-cols-1 lg:grid-cols-2 relative h-fit">
      <!-- Overlay content -->
      {#if children}
        {@render children()}
      {/if}
    
      <div class="relative lg:aspect-4/5 xl:aspect-square order-2 lg:order-1">
        {@render left()}
      </div>
      <div class="aspect-square lg:aspect-4/5 xl:aspect-square relative order-1">
        {@render right()}
      </div>
    </div>
    
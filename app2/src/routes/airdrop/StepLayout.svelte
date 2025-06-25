<script lang="ts">
import type { Snippet } from "svelte"

interface Props {
  left: Snippet
  right: Snippet
  children?: Snippet
}

let { left, right, children }: Props = $props()
</script>

<div class="grid grid-cols-1 sm:grid-cols-2 relative">
  <!-- Overlay content (like confetti) -->
  {#if children}
    {@render children()}
  {/if}
  
  <div class="relative aspect-video sm:aspect-square order-2 sm:order-1">
    {@render left()}
  </div>
  <div class="aspect-square relative order-1">
    {@render right()}
  </div>
</div> 
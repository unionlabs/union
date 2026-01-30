<script lang="ts">
import { chainStore } from "$lib/stores/chain.svelte"
import { cache } from "$lib/cache/promise.svelte"
import { browser } from "$app/environment"

const { data, children } = $props()

// Sync chain store with URL parameter (universal_chain_id)
$effect(() => {
  if (data.chainId && chainStore.id !== data.chainId) {
    // Clear cache when switching chains
    if (browser) {
      cache.clear()
    }
    chainStore.set(data.chainId)
  }
})
</script>

{@render children()}

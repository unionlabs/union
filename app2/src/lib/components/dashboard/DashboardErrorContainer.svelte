<script lang="ts">
import { uiStore } from "$lib/dashboard/stores/ui"
import DashboardErrorComponent from "./DashboardErrorComponent.svelte"
import { fade } from "svelte/transition"
</script>

{#if $uiStore.error}
  <div 
    class="w-full"
    transition:fade
  >
    <div class="max-w-4xl mx-auto px-4 py-2">
      <DashboardErrorComponent
        error={$uiStore.error}
        onClose={() => uiStore.clearError()}
      />
    </div>
  </div>
{/if} 
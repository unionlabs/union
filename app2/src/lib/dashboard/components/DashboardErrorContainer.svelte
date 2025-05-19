<script lang="ts">
import { errorStore } from "$lib/dashboard/stores/errors.svelte"
import DashboardErrorComponent from "./DashboardErrorComponent.svelte"

$effect(() => {
  if (errorStore.current) {
    console.log("Error detected, scrolling to top...")
    requestAnimationFrame(() => {
      // Try multiple scroll methods for better browser compatibility
      window.scrollTo({ top: 0, behavior: "smooth" })
      document.documentElement.scrollTo({ top: 0, behavior: "smooth" })
      document.body.scrollTo({ top: 0, behavior: "smooth" })

      // Fallback method if smooth scroll isn't working
      document.documentElement.scrollTop = 0
      document.body.scrollTop = 0
    })
  }
})
</script>

{#if errorStore.current}
  <DashboardErrorComponent
    error={errorStore.current}
    onClose={() => errorStore.clearError()}
  />
{/if}

<script lang="ts">
import Button from "../../ui/Button.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { totalErrorCount } from "$lib/stores/app-errors.svelte"
import Breadcrumbs from "./Breadcrumbs.svelte"
</script>

<header class="flex justify-between items-center h-16 px-8 border-b-1 border-zinc-900">
  <Breadcrumbs />
  {#if totalErrorCount() > 0}
    <Button variant="danger" onclick={() => uiStore.openErrorsModal()}>
      {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
    </Button>
  {/if}
</header>

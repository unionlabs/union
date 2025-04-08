<script lang="ts">
import Button from "../../ui/Button.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { totalErrorCount } from "$lib/stores/app-errors.svelte"
import Breadcrumbs from "./Breadcrumbs.svelte"
import CopyLink from "./CopyLink.svelte"
</script>

<header class="flex justify-between items-center h-16 px-8 border-b-1 border-zinc-900">
  <Breadcrumbs />
  <div class="flex items-center gap-3">
    <CopyLink />
    {#if totalErrorCount() > 0}
      <Button variant="danger" onclick={() => uiStore.openErrorsModal()}>
        {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
      </Button>
    {/if}
  </div>
</header>

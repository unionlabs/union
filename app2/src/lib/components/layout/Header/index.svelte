<script lang="ts">
  import Button from "../../ui/Button.svelte"
  import {uiStore} from "$lib/stores/ui.svelte"
  import {totalErrorCount} from "$lib/stores/app-errors.svelte"
  import Breadcrumbs from "./Breadcrumbs.svelte"
  import CopyLink from "./CopyLink.svelte"
  import SharpBaselineArrowBackIcon from "$lib/components/icons/SharpBaselineArrowBackIcon.svelte"

  type Props = {
  showNavigation?: boolean | undefined
}
const { showNavigation = false }: Props = $props()
</script>

<header class="flex items-center h-16 gap-4 px-8 border-b-1 border-zinc-900 bg-zinc-950">
  {#if showNavigation}
    <Button
      tabindex={0}
      aria-label="Go back"
      variant="icon"
      onclick={() => history.back()}
    >
      <SharpBaselineArrowBackIcon width="1.5rem" height="1.5rem" />
    </Button>
  {/if}
  <Breadcrumbs />
  <div class="grow"></div>
  <div class="flex items-center gap-3">
    <CopyLink />
    {#if totalErrorCount() > 0}
      <Button variant="danger" onclick={() => uiStore.openErrorsModal()}>
        {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
      </Button>
    {/if}
  </div>
</header>

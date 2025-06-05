<script lang="ts">
import SharpBaselineArrowBackIcon from "$lib/components/icons/SharpBaselineArrowBackIcon.svelte"
import { totalErrorCount } from "$lib/stores/app-errors.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Button from "../../ui/Button.svelte"
import Breadcrumbs from "./Breadcrumbs.svelte"
import CopyLink from "./CopyLink.svelte"

type Props = {
  showNavigation?: boolean | undefined
}
const { showNavigation = false }: Props = $props()

// Maintenance banner state
let showMaintenanceBanner = $state(true)
</script>

<header class="flex items-center h-16 gap-4 px-8 border-b-1 border-zinc-900 bg-zinc-950">
  {#if showNavigation}
    <Button
      tabindex={0}
      aria-label="Go back"
      variant="icon"
      onclick={() => history.back()}
    >
      <SharpBaselineArrowBackIcon
        width="1.5rem"
        height="1.5rem"
      />
    </Button>
  {/if}
  <Breadcrumbs />
  <div class="grow"></div>
  <div class="flex items-center gap-3">
    <CopyLink />
    {#if totalErrorCount() > 0}
      <Button
        variant="danger"
        onclick={() => uiStore.openErrorsModal()}
      >
        {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
      </Button>
    {/if}
  </div>
</header>

<!-- Maintenance Banner -->
{#if showMaintenanceBanner}
  <div
    class="bg-yellow-400 text-black px-3 sm:px-4 py-2 sm:py-3 text-center text-xs sm:text-sm font-medium relative"
  >
    <div class="flex items-center justify-center gap-1 sm:gap-2 pr-8">
      <svg
        class="w-3 h-3 sm:w-4 sm:h-4 flex-shrink-0"
        fill="currentColor"
        viewBox="0 0 20 20"
      >
        <path
          fill-rule="evenodd"
          d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
          clip-rule="evenodd"
        />
      </svg>
      <span class="leading-tight">
        <span class="hidden sm:inline"
        >Scheduled maintenance in progress. Some features may be temporarily unavailable.</span>
        <span class="sm:hidden">Maintenance in progress. Features may be unavailable.</span>
      </span>
    </div>
    <button
      class="absolute right-2 sm:right-4 top-1/2 transform -translate-y-1/2 text-gray-800 hover:text-gray-900 font-bold cursor-pointer text-base sm:text-lg"
      onclick={() => showMaintenanceBanner = false}
      title="Dismiss banner"
    >
      ×
    </button>
  </div>
{/if}

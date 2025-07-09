<script lang="ts">
  import Menu from './Menu.svelte'
  import Banner from "$lib/components/ui/Banner.svelte"
  import { totalErrorCount } from "$lib/stores/app-errors.svelte"
  import { uiStore } from "$lib/stores/ui.svelte"
  import Button from "$lib/components/ui/Button.svelte"
  import Breadcrumbs from "./Breadcrumbs.svelte"
  import CopyLink from "./CopyLink.svelte" 
  import ConnectWalletButton from '$lib/components/ui/ConnectWalletButton.svelte';

  interface Props {
    chains?: Set<number>
    showNavigation?: boolean
  }

  let menu = $state(false)
</script>

<header
  class="flex h-14 shrink-0 items-center justify-between gap-4 border-b border-zinc-900 px-2 py-2 md:px-4 bg-zinc-950"
>
  <div class="mr-auto flex flex-1 flex-shrink-0 items-center justify-start gap-3">
    <a href="/" class="inline-flex flex-shrink-0 items-center md:hidden">
      <img src="/images/union-logo-glyph.svg" alt="Union" class="h-8 w-8 md:h-12 md:w-12" />
    </a>
  </div>

  <div class="flex flex-1 justify-end gap-2">

    <ConnectWalletButton />

    <Button
      variant="icon"
      class="order-2 md:order-none md:hidden"
      aria-controls="mobile-menu"
      aria-expanded={menu}
      onclick={() => (menu = !menu)}
      title="Open menu"
    >
      <svg class="h-5 w-5" fill="currentColor" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 30"
        ><path
          d="M1.169 6a1.156 1.156 0 0 0-1.08.706 1.146 1.146 0 0 0 .634 1.51c.142.056.293.085.446.082h27.662a1.156 1.156 0 0 0 1.08-.705A1.147 1.147 0 0 0 28.83 6H1.169Zm0 8.044A1.155 1.155 0 0 0 0 15.194a1.147 1.147 0 0 0 .723 1.066c.142.057.293.085.446.083h27.662a1.156 1.156 0 0 0 1.08-.706 1.147 1.147 0 0 0-1.08-1.593H1.169Zm0 8.045a1.156 1.156 0 0 0-1.08.705 1.147 1.147 0 0 0 .634 1.51c.142.057.293.085.446.083h27.662a1.156 1.156 0 0 0 1.08-.706 1.147 1.147 0 0 0-1.08-1.592H1.169Z"
        /></svg
      >
    </Button>
  </div>
</header>

{#if menu}
  <Menu onclose={() => (menu = false)} />
{/if}

<!-- Old Header with Breadcrumbs (Bottom) -->
<header class="flex items-center h-12 gap-4 px-2 md:px-4 border-b border-zinc-900 bg-zinc-950">
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

<Banner />

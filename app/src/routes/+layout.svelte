<script lang="ts">
import "$lib/polyfill.ts"
import "$styles/index.css"
import { onMount } from "svelte"
import { page } from "$app/stores"
import { ModeWatcher } from "mode-watcher"
import { browser } from "$app/environment"
import { shortcut } from "@svelte-put/shortcut"
import { cosmosStore } from "$lib/wallet/cosmos"
import Footer from "$lib/components/footer.svelte"
import { Toaster } from "$lib/components/ui/sonner"
import { notifyManager } from "@tanstack/svelte-query"
import DevTools from "$lib/components/dev-tools.svelte"
import { createQueryClient } from "$lib/query-client.ts"
import Header from "$lib/components/header/header.svelte"
import LoadingBar from "$lib/components/loading-bar.svelte"
import { updateTheme } from "$lib/utilities/update-theme.ts"
import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools"
import { checkWebGLSupport, deviceWidth } from "$lib/utilities/device.ts"
import { disablePinchToZoom } from "$lib/utilities/disable-pinch-to-zoom.ts"
import { crtEffectEnabled } from "$lib/stores/user"

const { queryClient, localStoragePersister, PersistQueryClientProvider } = createQueryClient()
if (browser) notifyManager.setScheduler(window.requestAnimationFrame)

onMount(() => {
  checkWebGLSupport()
  disablePinchToZoom()
})

$: updateTheme({ path: $page.url.pathname, activeTheme: "dark" })
</script>

<svelte:head>
  <title>Union App Beta</title>
  <meta name="description" content="Union Web App" />
</svelte:head>

<svelte:window
  bind:innerWidth={$deviceWidth}
  use:shortcut={{
    trigger: [
      // easily hide tanstack devtools with ctrl + h
      {
        key: "h",
        modifier: ["ctrl"],
        callback: () => {
          console.info("Hiding tanstack devtools")
          const tanstackDevtoolsElement = document.querySelector(
            "div.tsqd-transitions-container"
          )
          if (!tanstackDevtoolsElement) return
          tanstackDevtoolsElement.classList.toggle("hidden")
        }
      }
    ]
  }}
/>

<LoadingBar />

<PersistQueryClientProvider
  client={queryClient}
  persistOptions={{ persister: localStoragePersister }}
>
  <ModeWatcher defaultMode="system" />
  <Toaster position="bottom-right" expand />

  <Header />
  <div class="flex flex-1 overflow-y-auto bg-background">
    <slot />
  </div>
  <Footer />
  <SvelteQueryDevtools
    position="bottom"
    client={queryClient}
    initialIsOpen={false}
    buttonPosition="bottom-right"
  />
  <DevTools />
  <!-- will be enabled once powered by index status !-->
  <!-- <OnlineStatus /> !-->
</PersistQueryClientProvider>

<div class:crt={$crtEffectEnabled} class="absolute top-0 w-dvw h-dvh z-50 pointer-events-none"></div>

<style>
  :global([data-close-button]) {
    background-color: hsl(var(--card) / var(--tw-bg-opacity));
  }
  :global(.grecaptcha-badge) {
    visibility: hidden;
    position: fixed;
    width: 0;
    height: 0;
  }
</style>

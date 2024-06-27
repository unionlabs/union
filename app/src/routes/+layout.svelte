<script lang="ts">
import "$lib/polyfill.ts"
import "$styles/index.css"
import { onMount } from "svelte"
import { ModeWatcher } from "mode-watcher"
import { browser } from "$app/environment"
import { shortcut } from "@svelte-put/shortcut"
import { cosmosStore } from "$lib/wallet/cosmos"
import Footer from "$lib/components/footer.svelte"
import { Toaster } from "$lib/components/ui/sonner"
import { notifyManager } from "@tanstack/svelte-query"
import { createQueryClient } from "$lib/query-client.ts"
import Header from "$lib/components/header/header.svelte"
import LoadingBar from "$lib/components/loading-bar.svelte"
import { partytownSnippet } from "@builder.io/partytown/integration"
import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools"
import { PersistQueryClientProvider } from "@tanstack/svelte-query-persist-client"
import { disablePinchToZoom } from "$lib/utilities/disable-pinch-to-zoom.ts"

const { queryClient, localStoragePersister } = createQueryClient()
if (browser) notifyManager.setScheduler(window.requestAnimationFrame)

onMount(() => {
  disablePinchToZoom()
  const lastConnectedWallet = $cosmosStore["connectedWallet"] as "leap" | "keplr"
  if (
    lastConnectedWallet &&
    window[lastConnectedWallet] &&
    ["leap", "keplr"].includes(lastConnectedWallet)
  )
    return cosmosStore.connect(lastConnectedWallet)

  if (window?.keplr) cosmosStore.connect("keplr")
  else if (window?.leap) cosmosStore.connect("leap")
})
</script>

<svelte:head>
  <title>Union App Beta</title>
  <meta name="description" content="Union Web App" />
  <script>
    partytown = { forward: ['dataLayer.push'] }
  </script>
  {@html '<script>' + partytownSnippet() + '</script>'}
</svelte:head>

<svelte:window
  use:shortcut={{
    trigger: [
      // easily hide tanstack devtools with ctrl + h
      {
        key: 'h',
        modifier: ['ctrl'],
        callback: () => {
          console.info('Hiding tanstack devtools')
          const tanstackDevtoolsElement = document.querySelector('div.tsqd-transitions-container')
          if (!tanstackDevtoolsElement) return
          tanstackDevtoolsElement.classList.toggle('hidden')
        },
      },
    ],
  }}
/>

<LoadingBar />

<PersistQueryClientProvider
  client={queryClient}
  persistOptions={{ persister: localStoragePersister }}
>
  <ModeWatcher />
  <Toaster position="bottom-right" />

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
  <!-- will be enabled once powered by index status !-->
  <!-- <OnlineStatus /> !-->
</PersistQueryClientProvider>

<style>
  :global(.grecaptcha-badge) {
    bottom: 70px !important;
  }
</style>

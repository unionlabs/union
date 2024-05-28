<script lang="ts">
import "$lib/polyfill.ts"
import "$styles/index.css"
import { onMount } from "svelte"
import { ModeWatcher } from "mode-watcher"
import { browser } from "$app/environment"
import { Toaster } from "svelte-french-toast"
import { page, navigating } from "$app/stores"
import { shortcut } from "@svelte-put/shortcut"
import { cosmosStore } from "$lib/wallet/cosmos"
import Footer from "$lib/components/footer.svelte"
import Header from "$lib/components/header/header.svelte"
import { updateTheme } from "$lib/utilities/update-theme.ts"
import OnlineStatus from "$lib/components/online-status.svelte"
import { partytownSnippet } from "@builder.io/partytown/integration"
import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools"
import PreloadingIndicator from "$lib/components/preloading-indicator.svelte"
import { QueryClient, MutationCache, notifyManager } from "@tanstack/svelte-query"
import { PersistQueryClientProvider } from "@tanstack/svelte-query-persist-client"
import { createSyncStoragePersister } from "@tanstack/query-sync-storage-persister"

if (browser) notifyManager.setScheduler(window.requestAnimationFrame)

$: updateTheme({ path: $page.url.pathname, activeTheme: "dark" })

onMount(() => {
  /* fix for iOS Safari viewport zooming on input focus */
  if (navigator.userAgent.indexOf("iPhone") === -1) return
  const metaElement = document.querySelector("meta[name=viewport]")
  if (!metaElement) return
  metaElement.setAttribute("content", "width=device-width, initial-scale=1, maximum-scale=1")
})

onMount(() => {
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

const queryClient: QueryClient = new QueryClient({
  defaultOptions: {
    queries: {
      enabled: browser,
      gcTime: 1_000 * 60 * 60 * 24, // 24 hours
      refetchOnReconnect: () => !queryClient.isMutating()
    }
  },
  mutationCache: new MutationCache({
    onSettled: () => {
      if (queryClient.isMutating() === 1) {
        return queryClient.invalidateQueries()
      }
    }
  })
})

const localStoragePersister = createSyncStoragePersister({
  key: "SVELTE_QUERY",
  storage: browser ? window.localStorage : undefined // or window.sessionStorage
})

$: if ($navigating) console.log("Navigating to", $page.url.pathname)
</script>

<svelte:head>
  <title>Union App Beta</title>
  <meta name="description" content="Union Web App" />
  <script>
    partytown = { forward: ['dataLayer.push'] }
  </script>
  {@html '<script>' + partytownSnippet() + '</script>'}
</svelte:head>

{#if $navigating}
  <PreloadingIndicator />
{/if}

<svelte:window
  use:shortcut={{
    trigger: [
      // easily hide tanstack devtools with ctrl + h
      {
        key: 'h',
        modifier: ['ctrl'],
        callback: () => {
          console.log('Hiding tanstack devtools')
          const tanstackDevtoolsElement = document.querySelector('div.tsqd-transitions-container')
          if (!tanstackDevtoolsElement) return
          tanstackDevtoolsElement.classList.toggle('hidden')
        },
      },
    ],
  }}
/>

<PersistQueryClientProvider
  client={queryClient}
  persistOptions={{ persister: localStoragePersister }}
>
  <ModeWatcher />
  <Toaster position="bottom-right" />

  <Header />
  <slot />
  <Footer />

  <SvelteQueryDevtools
    position="bottom"
    client={queryClient}
    initialIsOpen={false}
    buttonPosition="bottom-right"
  />
  <OnlineStatus />
</PersistQueryClientProvider>

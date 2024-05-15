<script lang="ts">
import "$lib/polyfill.ts"
import "$styles/index.css"
import {
  hydrate,
  dehydrate,
  QueryClient,
  MutationCache,
  notifyManager,
  QueryClientProvider
} from "@tanstack/svelte-query"
import { cn } from "$lib/utilities/shadcn"
import { ModeWatcher } from "mode-watcher"
import { browser } from "$app/environment"
import { setContext, onMount } from "svelte"
import { Toaster } from "svelte-french-toast"
import { page, navigating } from "$app/stores"
import { shortcut } from "@svelte-put/shortcut"
import { setContextClient } from "@urql/svelte"
import { cosmosStore } from "$lib/wallet/cosmos"
import Footer from "$lib/components/footer.svelte"
import { graphqlClient } from "$lib/graphql/client"
import Header from "$lib/components/header/header.svelte"
import { updateTheme } from "$lib/utilities/update-theme.ts"
import OnlineStatus from "$lib/components/online-status.svelte"
import { partytownSnippet } from "@builder.io/partytown/integration"
import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools"
import PreloadingIndicator from "$lib/components/preloading-indicator.svelte"
// import { snaps } from '$lib/wallet/snaps/config';
import { provider } from '$lib/wallet/evm/config.ts';

onMount(async() => {
  console.info()
  // await snaps.installed(window.ethereum)
})
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

/**
 * @see https://commerce.nearform.com/open-source/urql/docs/basics/svelte/#providing-the-client
 */
setContextClient(graphqlClient)

const queryClient: QueryClient = new QueryClient({
  defaultOptions: {
    queries: {
      enabled: browser,
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
setContext("$$_queryClient", queryClient)

function hydrateClient() {
  try {
    const storeValue = localStorage.getItem("QUERY_CLIENT")
    if (!storeValue) return
    const persistedValue = JSON.parse(storeValue) as Record<string, any>
    if ("timestamp" in persistedValue && persistedValue?.["timestamp"]) {
      const MAX_AGE = 1000 * 60 * 60 * 24
      const expired = Date.now() - persistedValue["timestamp"] > MAX_AGE
      if (!expired) hydrate(queryClient, persistedValue.clientState)
    } else localStorage.removeItem("QUERY_CLIENT")
  } catch (error) {
    localStorage.removeItem("QUERY_CLIENT")
  }
}
const saveClient = () =>
  localStorage.setItem(
    "QUERY_CLIENT",
    JSON.stringify({ timestamp: Date.now(), clientState: dehydrate(queryClient, {}) })
  )

const unload = () => saveClient()
onMount(() => {
  hydrateClient()
  queryClient.mount()
  return () => queryClient.unmount()
})

$: if ($navigating) console.log("Navigating to", $page.url.pathname)

/** @docs https://monogram.io/blog/add-partytown-to-svelte */
let partytownScriptElement: HTMLScriptElement
onMount(() => {
  if (!partytownScriptElement) return
  partytownScriptElement.textContent = partytownSnippet()
})
</script>

<svelte:head>
  <title>Union App Beta</title>
  <meta name="description" content="Union Web App" />
  <script>
    partytown = { forward: ['dataLayer.push'] }
  </script>
  <script bind:this={partytownScriptElement}></script>
  <!-- {@html `<script>${partytownSnippet()}</script>`} -->
</svelte:head>

{#if $navigating}
  <PreloadingIndicator />
{/if}

<ModeWatcher />
<Toaster />

<QueryClientProvider client={queryClient}>
  <Header />
  <div
    id="page"
    data-vaul-drawer-wrapper
    class="relative flex flex-col bg-background bg-opacity-10 mb-20"
  >
    <slot />
  </div>
  <Footer />
  <SvelteQueryDevtools
    position="bottom"
    client={queryClient}
    initialIsOpen={false}
    buttonPosition="bottom-right"
  />
</QueryClientProvider>
<OnlineStatus />

<div
  id="background-dotted-grid"
  data-background-dotted-grid="true"
  class={cn(
    'absolute top-0 z-[-2] size-full min-h-screen bg-[size:20px_20px]',
    'bg-[#b9e9ff78] bg-[radial-gradient(#638c91_0.3px,#b9e9ff78_1px)]',
    'dark:bg-[#99e6ff20] dark:bg-[radial-gradient(#4545538c_0.3px,#09090b_1px)]',
  )}
></div>

<svelte:window
  on:beforeunload={unload}
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

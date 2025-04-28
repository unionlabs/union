<script lang="ts">
import "../app.css"
import { onMount } from "svelte"
import { Effect, Fiber, Option } from "effect"
import { chainsQuery } from "$lib/queries/chains.svelte"
import Sidebar from "$lib/components/layout/Sidebar/index.svelte"
import { ENV, MAX_MOBILE_SIZE } from "$lib/constants"
import { wallets } from "$lib/stores/wallets.svelte"
import Wallet from "$lib/components/ui/Wallet/index.svelte"
import SettingsModal from "$lib/components/SettingsModal.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import ErrorsModal from "$lib/components/ErrorsModal.svelte"
import Header from "$lib/components/layout/Header/index.svelte"
import { channelsQuery } from "$lib/queries/channels.svelte.ts"
import { runExample } from "$lib/utils/convert-display.ts"
import { cn } from "$lib/utils"
import { page } from "$app/state"
import { runFork } from "$lib/utils/effect.svelte"

let { children } = $props()

/* Hack to be able to JSON.stringify BigInt */
interface BigInt {
  toJSON: () => string
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}

$effect(() => {
  const hostname = page.url.hostname
  // Clear any existing override when hostname changes
  uiStore.overrideEdition = null

  if (hostname.startsWith("btc.") || hostname.startsWith("staging.btc.")) {
    uiStore.edition = "btc"
  } else if (hostname.startsWith("app.") || hostname.startsWith("staging.app.")) {
    uiStore.edition = "app"
  } else {
    uiStore.edition = "app"
  }
})

onMount(() => {
  runExample()
  runFork(chainsQuery(ENV()))
  runFork(channelsQuery())

  const handler = (e: KeyboardEvent) => {
    const metaOrCtrl = e.metaKey || e.ctrlKey
    if (metaOrCtrl && e.altKey && e.shiftKey) {
      if (e.code === "KeyA") {
        uiStore.overrideEdition = "app"
      } else if (e.code === "KeyB") {
        uiStore.overrideEdition = "btc"
      }
    }
  }

  window.addEventListener("keydown", handler)
  return () => {
    window.removeEventListener("keydown", handler)
  }
})

$effect(() => {
  Effect.runPromise(
    Effect.log(
      "connected wallets",
      wallets.evmAddress.pipe(Option.getOrElse(() => "no evm")),
      wallets.cosmosAddress.pipe(Option.getOrElse(() => "no cosmos")),
      wallets.aptosAddress.pipe(Option.getOrElse(() => "no aptos"))
    )
  )
})

let viewportWidth = $state(0)
const isRootPage = $derived(page.url.pathname === "/")
const isMobile = $derived(viewportWidth < MAX_MOBILE_SIZE)
const hideSidebar = $derived(isMobile && !isRootPage)
const fullPageSidebar = $derived(isRootPage)
let videoLoaded = $state(false)
let currentVideoUrl = $state(uiStore.theme.videoUrl)
let videoKey = $state(0)

$effect(() => {
  const newUrl = uiStore.theme.videoUrl
  if (newUrl !== currentVideoUrl) {
    currentVideoUrl = newUrl
    videoLoaded = false
    videoKey++
  }
})
</script>

<style>
  :global(:root) {
    --color-accent: v-bind(uiStore.theme.accent);
    --color-primary: v-bind(uiStore.theme.primary);
    --color-background: v-bind(uiStore.theme.background);
    --color-text: v-bind(uiStore.theme.text);
  }
</style>

<!-- Background video -->
{#if !isMobile}
  <div
    class="fixed inset-0 w-screen h-screen z-0 transition-opacity duration-1000"
    class:opacity-0={!videoLoaded}
    class:opacity-100={videoLoaded}
  >
    {#key videoKey}
      <video
        id="glitch-video"
        class="w-full h-full object-cover"
        loop
        muted
        autoplay
        playsinline
        data-video="glitch"
        disablePictureInPicture={true}
        oncanplay={function () {
          this.autoplay = true;
        }}
        onloadeddata={function () {
          this.autoplay = true;
          videoLoaded = true;
        }}
        onloadedmetadata={function () {
          this.muted = true;
        }}
      >
        <source
          src={currentVideoUrl}
          type="video/webm"
        />
      </video>
    {/key}
  </div>
{:else}
  <div class="fixed inset-0 w-screen h-screen z-0">
    <img
      src={uiStore.theme.staticImage}
      alt="Union background"
      class="w-full h-full object-cover"
    />
  </div>
{/if}

<div
  class={cn("relative min-h-[100svh] w-screen z-10")}
  bind:clientWidth={viewportWidth}
  style="--color-accent: {uiStore.theme.accent}"
>
  <aside
    class={cn(
      "fixed left-0 bottom-0 top-0 dark:bg-zinc-950 shadow overflow-auto border-r border-zinc-900 max-h-dvh",
      fullPageSidebar ? "right-0" : "w-64",
    )}
    hidden={hideSidebar}
  >
    <Sidebar />
  </aside>

  <!-- Main content area: Has margin to clear fixed sidebar -->
  <main
    class={cn(
      "fixed min-h-svh grow right-0 top-0 bottom-0",
      fullPageSidebar ? "w-0" : null,
      hideSidebar ? "left-0" : "left-64",
    )}
    hidden={fullPageSidebar}
  >
    <div class="sticky top-0">
      <Header showNavigation={isMobile} />
    </div>

    <div
      class="absolute top-16 left-0 right-0 bottom-0 flex-1 z-10 overflow-scroll"
    >
      {@render children()}
    </div>
  </main>
</div>
<Wallet />
<SettingsModal
  isOpen={uiStore.settingsModalOpen}
  onClose={() => uiStore.closeSettingsModal()}
/>
<ErrorsModal
  isOpen={uiStore.errorsModalOpen}
  onClose={() => uiStore.closeErrorsModal()}
/>

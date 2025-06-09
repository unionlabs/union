<script lang="ts">
import "../app.css"
import { page } from "$app/state"
import DevInfo from "$lib/components/DevInfo.svelte"
import ErrorsModal from "$lib/components/ErrorsModal.svelte"
import Header from "$lib/components/layout/Header/index.svelte"
import Sidebar from "$lib/components/layout/Sidebar/index.svelte"
import Seo from "$lib/components/Seo.svelte"
import SettingsModal from "$lib/components/SettingsModal.svelte"
import Wallet from "$lib/components/ui/Wallet/index.svelte"
import { ENV, MAX_MOBILE_SIZE } from "$lib/constants"
import { bannerQuery } from "$lib/queries/banner.svelte.ts"
import { chainsQuery } from "$lib/queries/chains.svelte"
import { channelsQuery } from "$lib/queries/channels.svelte.ts"
import { runFork$ } from "$lib/runtime"
import { settingsStore } from "$lib/stores/settings.svelte"
import { keyboardShortcuts } from "$lib/stores/shortcuts.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { cn } from "$lib/utils"
import { runExample } from "$lib/utils/convert-display.ts"
import { interceptLogos } from "$lib/utils/intercept-logos.ts"
import { onMount, type Snippet } from "svelte"
import type { PageData } from "./$types.ts"

interface Props {
  children: Snippet
  data: PageData
}

let { children, data }: Props = $props()

onMount(() => {
  // TODO: removal contender given static assignment in `UiStore` constructor
  settingsStore.setEditionDefaults(uiStore.edition)

  interceptLogos()
  runExample()
  runFork$(chainsQuery(ENV()))
  runFork$(channelsQuery())
  runFork$(bannerQuery())

  keyboardShortcuts.addShortcut(["ctrl", "option", "shift", "keya"], () => {
    uiStore.overrideEdition = "app"
  })

  keyboardShortcuts.addShortcut(["ctrl", "option", "shift", "keyb"], () => {
    uiStore.overrideEdition = "btc"
  })

  keyboardShortcuts.addShortcut(["ctrl", "option", "shift", "keyf"], () => {
    uiStore.filterWhitelist = !uiStore.filterWhitelist
  })
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

$effect(() => {
  document.documentElement.style.setProperty("--color-accent", uiStore.theme.accent)
  document.documentElement.style.setProperty("--color-primary", uiStore.theme.primary)
  document.documentElement.style.setProperty("--color-background", uiStore.theme.background)
})

// Update settings when edition changes
$effect(() => {
  const edition = uiStore.edition
  settingsStore.setEditionDefaults(edition)
})
</script>

<Seo />

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
        oncanplay={function() {
          this.autoplay = true
        }}
        onloadeddata={function() {
          this.autoplay = true
          videoLoaded = true
        }}
        onloadedmetadata={function() {
          this.muted = true
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
  class={cn("relative min-h-[100svh] w-screen z-0")}
  bind:clientWidth={viewportWidth}
  style="--color-accent: {uiStore.theme.accent}"
>
  <aside
    class={cn(
      "fixed left-0 bottom-0 top-0 dark:bg-zinc-950 shadow overflow-hidden border-r border-zinc-900 max-h-dvh z-0",
      fullPageSidebar ? "right-0" : "w-64",
    )}
    hidden={hideSidebar}
  >
    <Sidebar />
  </aside>

  <!-- Main content area: Has margin to clear fixed sidebar -->
  <main
    class={cn(
      "fixed min-h-svh grow right-0 top-0 bottom-0 z-0",
      fullPageSidebar ? "w-0" : null,
      hideSidebar ? "left-0" : "left-64",
    )}
    hidden={fullPageSidebar}
  >
    <div class="sticky top-0 z-20">
      <Header showNavigation={isMobile} />
    </div>

    <div class="absolute top-16 left-0 right-0 bottom-0 flex-1 z-0 overflow-scroll">
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
<DevInfo />

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
import { cn, vw } from "$lib/utils"
import { page } from "$app/state"

let { children } = $props()

/* Hack to be able to JSON.stringify BigInt */
interface BigInt {
  toJSON: () => string
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}

onMount(() => {
  runExample()
  const chainsFiber = Effect.runFork(chainsQuery(ENV()))
  const channelsFiber = Effect.runFork(channelsQuery())

  return () => {
    Effect.runPromise(Fiber.interrupt(chainsFiber))
    Effect.runPromise(Fiber.interrupt(channelsFiber))
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
</script>

<!-- Background video -->
<div class="fixed inset-0 w-screen h-screen z-0 transition-opacity duration-2000"
     class:opacity-0={!videoLoaded}
     class:opacity-100={videoLoaded}>
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
           videoLoaded = true;
      this.autoplay = true;
    }}
    onloadedmetadata={function () {
      this.muted = true;
    }}
  >
    <source
      src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/btc-union-background-3.webm"
      type="video/webm"
    />
  </video>
</div>

<div
  class={cn("relative min-h-[100svh] w-screen z-10")}
  bind:clientWidth={viewportWidth}
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

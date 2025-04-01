<script lang="ts">
import "../app.css"
import { onMount } from "svelte"
import { Effect, Fiber, Option } from "effect"
import { chainsQuery } from "$lib/queries/chains.svelte"
import Sidebar from "$lib/components/layout/Sidebar/index.svelte"
import { ENV } from "$lib/constants"
import { wallets } from "$lib/stores/wallets.svelte"
import Wallet from "$lib/components/ui/Wallet/index.svelte"
import SettingsModal from "$lib/components/SettingsModal.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import ErrorsModal from "$lib/components/ErrorsModal.svelte"
import Header from "$lib/components/layout/Header/index.svelte"
import { channelsQuery } from "$lib/queries/channels.svelte.ts"
import { runExample } from "$lib/utils/convert-display.ts"

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
</script>

<div class="grid grid-cols-[auto_1fr] min-h-[100svh] w-screen">
  <aside class="fixed top-0 left-0 bottom-0 w-64 dark:bg-zinc-950 shadow overflow-auto border-r border-zinc-900">
    <Sidebar/>
  </aside>
  
  <!-- Main content area: Has margin to clear fixed sidebar -->
  <main class="col-start-2 ml-64 max-w-[calc(100vw-calc(var(--spacing)*64))]">
    <Header />
    {@render children()}
  </main>
</div>
<Wallet/>
<SettingsModal 
  isOpen={uiStore.settingsModalOpen} 
  onClose={() => uiStore.closeSettingsModal()}
/>
<ErrorsModal 
  isOpen={uiStore.errorsModalOpen} 
  onClose={() => uiStore.closeErrorsModal()}
/>

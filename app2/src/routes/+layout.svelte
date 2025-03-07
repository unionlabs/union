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
import Button from "$lib/components/ui/Button.svelte"
import { totalErrorCount } from "$lib/stores/app-errors.svelte"
import { page } from "$app/state"
import { navigation } from "$lib/components/layout/Sidebar/navigation.ts"

let { children } = $props()

/* Hack to be able to JSON.stringify BigInt */
interface BigInt {
  toJSON: () => string
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}

onMount(() => {
  const fiber = Effect.runFork(chainsQuery(ENV()))
  return () => Effect.runPromise(Fiber.interrupt(fiber))
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

const pageName = $derived(() => {
  const sections = navigation.find(section => section.items.find(s => s.path === page.url.pathname))
  if (!sections) return null

  const item = sections.items.find(i => i.path === page.url.pathname)

  if (!item) return null

  return item.title
})
</script>

<div class="grid grid-cols-[auto_1fr] min-h-[100svh] w-screen">
  <aside class="fixed top-0 left-0 bottom-0 w-64 dark:bg-zinc-950 shadow overflow-auto border-r border-zinc-900">
    <Sidebar/>
  </aside>
  
  <!-- Main content area: Has margin to clear fixed sidebar -->
  <main class="col-start-2 ml-64 max-w-[calc(100vw-calc(var(--spacing)*64))]">
    <header class="flex justify-between items-center h-16 px-8 border-b-1 border-zinc-900">
      <h1 class="text-xl font-bold">{pageName() ? pageName() : page.url.pathname}</h1>
      {#if totalErrorCount() > 0}
        <Button variant="danger" onclick={() => uiStore.openErrorsModal()}>
          {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
        </Button>
      {/if}
    </header>
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

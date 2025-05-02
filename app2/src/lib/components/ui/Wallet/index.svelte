<script lang="ts">
import Connection from "$lib/components/ui/Wallet/connect/connection.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.js"
import { evmWalletsInformation, sepoliaStore } from "$lib/wallet/evm/index.js"
import Modal from "../Modal.svelte"

let currentWalletType = $state("all")

let evmConnected = $state(false)
let cosmosConnected = $state(false)

$effect(() => {
  evmConnected = sepoliaStore.connectionStatus === "connected"
  cosmosConnected = cosmosStore.connectionStatus === "connected"
})
</script>

<Modal
  isOpen={uiStore.walletModalOpen}
  onClose={() => uiStore.closeWalletModal()}
  class="w-md max-h-[650px] flex flex-col"
  divided
>
  <section class="p-6 pb-4 border-b border-zinc-200 dark:border-zinc-800 flex-none">
    <h2 class="text-xl font-bold mb-2">Connect wallet</h2>
    <p class="text-sm text-zinc-500 dark:text-zinc-400">Choose how you want to connect</p>

    <nav
      class="flex gap-1 mt-4 relative"
      aria-label="Tabs"
    >
      <div
        class="absolute bg-zinc-100 dark:bg-zinc-800 rounded-md transition-all duration-300"
        style:top="0"
        style:left={currentWalletType === "all"
        ? "0"
        : currentWalletType === "evm"
        ? "33.333%"
        : "66.666%"}
        style:width="33.333%"
        style:height="100%"
      />
      <button
        onclick={() => currentWalletType = "all"}
        class="
          flex-1 px-4 py-2 text-sm font-medium rounded-md transition-colors relative cursor-pointer
          {currentWalletType === 'all'
          ? 'text-zinc-900 dark:text-white'
          : 'text-zinc-500 hover:text-zinc-900 dark:hover:text-white'}
        "
      >
        All
      </button>
      <button
        onclick={() => currentWalletType = "evm"}
        class="
          flex-1 px-4 py-2 text-sm font-medium rounded-md transition-colors relative cursor-pointer
          {currentWalletType === 'evm'
          ? 'text-zinc-900 dark:text-white'
          : 'text-zinc-500 hover:text-zinc-900 dark:hover:text-white'}
        "
      >
        <div class="flex items-center justify-center gap-2">
          <span>EVM</span>
          <span
            class="
              w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {evmConnected
              ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
              : 'bg-white/10 dark:bg-white/5 backdrop-blur-sm ring-white/20'}
            "
          >
          </span>
        </div>
      </button>
      <button
        onclick={() => currentWalletType = "cosmos"}
        class="
          flex-1 px-4 py-2 text-sm font-medium rounded-md transition-colors relative cursor-pointer
          {currentWalletType === 'cosmos'
          ? 'text-zinc-900 dark:text-white'
          : 'text-zinc-500 hover:text-zinc-900 dark:hover:text-white'}
        "
      >
        <div class="flex items-center justify-center gap-2">
          <span>Cosmos</span>
          <span
            class="
              w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {cosmosConnected
              ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
              : 'bg-white/10 dark:bg-white/5 backdrop-blur-sm ring-white/20'}
            "
          >
          </span>
        </div>
      </button>
    </nav>
  </section>

  <section class="h-[500px] overflow-y-auto p-6 space-y-6">
    {#if currentWalletType === "evm"}
      <Connection
        chain="evm"
        address={sepoliaStore.address}
        chainWalletsInformation={evmWalletsInformation}
        connectStatus={sepoliaStore.connectionStatus}
        connectedWalletId={sepoliaStore.connectedWallet}
        onConnectClick={sepoliaStore.connect}
        onDisconnectClick={sepoliaStore.disconnect}
        showDivider={false}
      />
    {:else if currentWalletType === "cosmos"}
      <Connection
        chain="cosmos"
        address={cosmosStore.address}
        chainWalletsInformation={cosmosWalletsInformation}
        connectStatus={cosmosStore.connectionStatus}
        connectedWalletId={cosmosStore.connectedWallet}
        onConnectClick={cosmosStore.connect}
        onDisconnectClick={cosmosStore.disconnect}
        showDivider={false}
      />
    {:else if currentWalletType === "all"}
      <Connection
        chain="evm"
        address={sepoliaStore.address}
        chainWalletsInformation={evmWalletsInformation}
        connectStatus={sepoliaStore.connectionStatus}
        connectedWalletId={sepoliaStore.connectedWallet}
        onConnectClick={sepoliaStore.connect}
        onDisconnectClick={sepoliaStore.disconnect}
        showDivider={true}
      />
      <Connection
        chain="cosmos"
        address={cosmosStore.address}
        chainWalletsInformation={cosmosWalletsInformation}
        connectStatus={cosmosStore.connectionStatus}
        connectedWalletId={cosmosStore.connectedWallet}
        onConnectClick={cosmosStore.connect}
        onDisconnectClick={cosmosStore.disconnect}
        showDivider={true}
      />
    {/if}
  </section>
</Modal>

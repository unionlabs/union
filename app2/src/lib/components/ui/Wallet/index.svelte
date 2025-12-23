<script lang="ts">
import Connection from "$lib/components/ui/Wallet/connect/connection.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.js"
import { evmWalletsInformation, sepoliaStore } from "$lib/wallet/evm/index.js"
import { suiStore, suiWalletsInformation } from "$lib/wallet/sui"
import { Option } from "effect"
import Modal from "../Modal.svelte"

let currentWalletType = $state("all")

let evmConnected = $state(false)
let cosmosConnected = $state(false)
let suiConnected = $state(false)

$effect(() => {
  evmConnected = sepoliaStore.connectionStatus === "connected"
  cosmosConnected = cosmosStore.connectionStatus === "connected"
  suiConnected = suiStore.connectionStatus === "connected"
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
    {#if Option.isSome(dashboard.user)}
      <p class="text-sm text-zinc-500 dark:text-zinc-400">Chose how you want to connect</p>
    {:else}
      <p class="text-sm text-zinc-500 dark:text-zinc-400">
        Connect your dashboard accouunt to earn points.
      </p>
    {/if}
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
        ? "25%"
        : currentWalletType === "cosmos"
        ? "50%"
        : "75%"}
        style:width="25%"
        style:height="100%"
      >
      </div>
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
      <button
        onclick={() => currentWalletType = "sui"}
        class="
          flex-1 px-4 py-2 text-sm font-medium rounded-md transition-colors relative cursor-pointer
          {currentWalletType === 'sui'
          ? 'text-zinc-900 dark:text-white'
          : 'text-zinc-500 hover:text-zinc-900 dark:hover:text-white'}
        "
      >
        <div class="flex items-center justify-center gap-2">
          <span>Sui</span>
          <span
            class="
              w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {suiConnected
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
    {:else if currentWalletType === "sui"}
      {#if suiWalletsInformation.length > 0}
        <Connection
          chain="sui"
          address={suiStore.address}
          chainWalletsInformation={suiWalletsInformation}
          connectStatus={suiStore.connectionStatus}
          connectedWalletId={suiStore.connectedWallet}
          onConnectClick={(id: string) => suiStore.connect(id as any)}
          onDisconnectClick={suiStore.disconnect}
          showDivider={false}
          connectionError={suiStore.connectionError}
          errorWalletId={suiStore.errorWalletId}
        />
      {:else}
        <a
          href="https://sui.io/get-started"
          target="_blank"
          rel="noopener noreferrer"
          class="w-full bg-zinc-100 dark:bg-zinc-900 rounded-lg p-4 flex items-center gap-3 hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors"
        >
          <img
            src="/logos/chains/color/SUI.svg"
            alt="Sui"
            class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1"
          />
          <div class="flex flex-col">
            <span class="text-zinc-900 dark:text-zinc-50 font-medium">Get a Sui Wallet</span>
            <span class="text-zinc-500 text-sm">No Sui wallets detected</span>
          </div>
        </a>
      {/if}
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
      {#if suiWalletsInformation.length > 0}
        <Connection
          chain="sui"
          address={suiStore.address}
          chainWalletsInformation={suiWalletsInformation}
          connectStatus={suiStore.connectionStatus}
          connectedWalletId={suiStore.connectedWallet}
          onConnectClick={(id: string) => suiStore.connect(id as any)}
          onDisconnectClick={suiStore.disconnect}
          showDivider={true}
          connectionError={suiStore.connectionError}
          errorWalletId={suiStore.errorWalletId}
        />
      {:else}
        <div class="flex flex-col gap-2">
          <div class="flex items-center gap-4 mb-2">
            <div class="flex-1 h-px bg-zinc-200 dark:bg-zinc-800"></div>
            <span class="text-zinc-500 dark:text-zinc-400 text-xs uppercase">sui</span>
            <div class="flex-1 h-px bg-zinc-200 dark:bg-zinc-800"></div>
          </div>
          <a
            href="https://sui.io/get-started"
            target="_blank"
            rel="noopener noreferrer"
            class="w-full bg-zinc-100 dark:bg-zinc-900 rounded-lg p-4 flex items-center gap-3 hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors"
          >
            <img
              src="/logos/chains/color/SUI.svg"
              alt="Sui"
              class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1"
            />
            <div class="flex flex-col">
              <span class="text-zinc-900 dark:text-zinc-50 font-medium">Get a Sui Wallet</span>
              <span class="text-zinc-500 text-sm">No Sui wallets detected</span>
            </div>
          </a>
        </div>
      {/if}
    {/if}
  </section>
</Modal>

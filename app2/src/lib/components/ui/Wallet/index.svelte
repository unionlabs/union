<script>
import { sepoliaStore, evmWalletsInformation } from "$lib/wallet/evm/index.js"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.js"
import { aptosStore, aptosWalletsInformation } from "$lib/wallet/aptos/index.js"
import Connection from "$lib/components/ui/Wallet/connect/connection.svelte"

let isOpen = $state(false)
let currentWalletType = $state("all")

function openModal() {
  isOpen = true
}

function closeModal() {
  isOpen = false
}
</script>

<!-- Button to open modal when it's closed -->
{#if !isOpen}
  <button class="bg-neutral-600 hover:bg-neutral-700 text-white font-bold py-3 px-6 rounded-lg" onclick={openModal}>
    Connect Wallet
  </button>
{/if}

{#if isOpen}

  <div class="fixed inset-0 bg-black/90 flex items-center justify-center z-50">

    <div class="max-h-[600px] min-h-[375px] h-full bg-neutral-900 border border-neutral-700  text-white sm:rounded-lg w-full max-w-md relative flex flex-col">

      <!-- Left side: Wallet list -->
      <div class="w-full h-full">
        <button
                class="cursor-pointer absolute top-4 right-4 text-white text-lg"
                onclick={closeModal}
        >
          ✕
        </button>

        <div class="border-b border-neutral-700 p-6 pb-4">
          <h2 class="text-xl font-bold">Connect wallet</h2>
          <p class="text-sm mb-4">Choose how you want to connect.</p>
          <nav class="flex space-x-2" aria-label="Tabs">
            <button onclick={() => currentWalletType = "all"}
                    class:bg-neutral-200={currentWalletType === "all"}
                    class:bg-neutral-800={currentWalletType !== "all"}
                    class="cursor-pointer rounded-md px-3 py-1 text-sm font-medium text-gray-500 hover:text-gray-700">
              All
            </button>
            <button onclick={() => currentWalletType = "evm"}
                    class:bg-neutral-200={currentWalletType === "evm"}
                    class:bg-neutral-800={currentWalletType !== "evm"}
                    class="cursor-pointer rounded-md px-3 py-1 text-sm font-medium text-gray-500 hover:text-gray-700">
              Evm
            </button>
            <button onclick={() => currentWalletType = "cosmos"}
                    class:bg-neutral-200={currentWalletType === "cosmos"}
                    class:bg-neutral-800={currentWalletType !== "cosmos"}
                    class="cursor-pointer rounded-md bg-neutral-800 px-3 py-1 text-sm font-medium text-gray-500 hover:text-gray-700">
              Cosmos
            </button>
            <button onclick={() => currentWalletType = "aptos"}
                    class:bg-neutral-200={currentWalletType === "aptos"}
                    class:bg-neutral-800={currentWalletType !== "aptos"}
                    class="cursor-pointer rounded-md bg-neutral-800 px-3 py-1 text-sm font-medium text-gray-500"
                    aria-current="page">Aptos
            </button>
          </nav>
        </div>

        <div class="calch overflow-y-scroll p-6 space-y-4">

          {#if currentWalletType === "evm"}
            <Connection
                    chain="evm"
                    address={sepoliaStore.address}
                    chainWalletsInformation={evmWalletsInformation}
                    connectStatus={sepoliaStore.connectionStatus}
                    connectedWalletId={sepoliaStore.connectedWallet}
                    onConnectClick={sepoliaStore.connect}
                    onDisconnectClick={sepoliaStore.disconnect}
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
            />
          {:else if currentWalletType === "aptos"}
            <Connection
                    chain="aptos"
                    address={aptosStore.address}
                    chainWalletsInformation={aptosWalletsInformation}
                    connectStatus={aptosStore.connectionStatus}
                    connectedWalletId={aptosStore.connectedWallet}
                    onConnectClick={aptosStore.connect}
                    onDisconnectClick={aptosStore.disconnect}
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
            />
            <Connection
                    chain="cosmos"
                    address={cosmosStore.address}
                    chainWalletsInformation={cosmosWalletsInformation}
                    connectStatus={cosmosStore.connectionStatus}
                    connectedWalletId={cosmosStore.connectedWallet}
                    onConnectClick={cosmosStore.connect}
                    onDisconnectClick={cosmosStore.disconnect}
            />
            <Connection
                    chain="aptos"
                    address={aptosStore.address}
                    chainWalletsInformation={aptosWalletsInformation}
                    connectStatus={aptosStore.connectionStatus}
                    connectedWalletId={aptosStore.connectedWallet}
                    onConnectClick={aptosStore.connect}
                    onDisconnectClick={aptosStore.disconnect}
            />
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .calch {
    height: calc(100% - 133px);
  }
</style>
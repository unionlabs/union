<script lang="ts">
import { evmWalletsInformation, sepoliaStore } from "$lib/wallet/evm/index.js"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.js"
import Connection from "$lib/components/ui/Wallet/connect/connection.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Modal from "../Modal.svelte"

let currentWalletType = $state("all")
</script>

<Modal 
  isOpen={uiStore.walletModalOpen} 
  onClose={() => uiStore.closeWalletModal()}
  class="w-md"
  divided
>
      <section class="p-6 pb-4">

          <h2 class="text-xl font-bold">Connect wallet</h2>
          <p class="text-sm mb-4">Choose how you want to connect.</p>
          <nav class="flex gap-2" aria-label="Tabs">
            <button onclick={() => currentWalletType = "all"}
                    class:bg-zinc-200={currentWalletType === "all"}
                    class:bg-zinc-800={currentWalletType !== "all"}
                    class="cursor-pointer rounded-md px-3 py-1 text-sm font-medium text-zinc-500 hover:text-zinc-700">
              All
            </button>
            <button onclick={() => currentWalletType = "evm"}
                    class:bg-zinc-200={currentWalletType === "evm"}
                    class:bg-zinc-800={currentWalletType !== "evm"}
                    class="cursor-pointer rounded-md px-3 py-1 text-sm font-medium text-zinc-500 hover:text-zinc-700">
              Evm
            </button>
            <button onclick={() => currentWalletType = "cosmos"}
                    class:bg-zinc-200={currentWalletType === "cosmos"}
                    class:bg-zinc-800={currentWalletType !== "cosmos"}
                    class="cursor-pointer rounded-md bg-zinc-800 px-3 py-1 text-sm font-medium text-zinc-500 hover:text-zinc-700">
              Cosmos
            </button>
            <!--
            <button onclick={() => currentWalletType = "aptos"}
                    class:bg-zinc-200={currentWalletType === "aptos"}
                    class:bg-zinc-800={currentWalletType !== "aptos"}
                    class="cursor-pointer rounded-md bg-zinc-800 px-3 py-1 text-sm font-medium text-zinc-500"
                    aria-current="page">Aptos
            </button>
            !-->
        </section>

        <section class="calch h-[calc(100%-133px)] overflow-y-scroll p-6 space-y-4">
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
            <!--
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
            !-->
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
            <!--
            <Connection
                    chain="aptos"
                    address={aptosStore.address}
                    chainWalletsInformation={aptosWalletsInformation}
                    connectStatus={aptosStore.connectionStatus}
                    connectedWalletId={aptosStore.connectedWallet}
                    onConnectClick={aptosStore.connect}
                    onDisconnectClick={aptosStore.disconnect}
            />
            !-->
          {/if}
        </section>
</Modal>


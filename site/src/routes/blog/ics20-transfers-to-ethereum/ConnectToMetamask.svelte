<script lang="ts">
  import { browser } from '$app/environment'
  import { initClients, startBalanceWorkers } from '$lib/transferDemo'
  import { onMount } from 'svelte'
  import {
    metamaskInstalled,
    connectedToSepolia,
    connectedToUnion,
    snapInstalled,
    sepUNOAdded,
  } from '$lib/stores/wallets'
  import {
    ethersSetup,
    connectToSepolia,
    updateConnectedToSepolia,
    connectLeapSnap,
    updateSnapInstalled,
    updateConnectedToUnion,
    connectToUnion,
    addUnoErc,
  } from '$lib/ethersSetup'

  import TerminalContainer from '$lib/TerminalContainer.svelte'
  import DemoButton from '$lib/DemoButton.svelte'
  import DemoButtonA from '$lib/DemoButtonA.svelte'

  connectedToUnion.subscribe(async connected => {
    if (connected) {
      await initClients()
      startBalanceWorkers()
    }
  })

  onMount(async () => {
    if (browser) {
      const mmInstalled = window.ethereum !== undefined
      metamaskInstalled.set(mmInstalled)
      if (mmInstalled) {
        ethersSetup()
        updateConnectedToSepolia()
        updateSnapInstalled()
        updateConnectedToUnion()
      }
    }
  })
</script>

<TerminalContainer>
  {#if !$metamaskInstalled}
    <div class="my-4">
      <DemoButtonA href="https://metamask.io/download/">Install MetaMask to continue 🦊</DemoButtonA
      >
    </div>
  {:else}
    <div>✅ MetaMask is installed</div>
    {#if !$connectedToSepolia}
      <div class="mt-4">
        <DemoButton on:click={connectToSepolia}>Connect to Sepolia</DemoButton>
      </div>
    {:else}
      <div>✅ Connected to Sepolia</div>
      {#if !$snapInstalled}
        <div class="mt-4">
          <DemoButton on:click={connectLeapSnap}>Add Leap Cosmos Wallet to Metamask 🌌</DemoButton>
        </div>
      {:else}
        <div>✅ Leap Cosmos Wallet Installed</div>
        {#if !$connectedToUnion}
          <div class="mt-4">
            <DemoButton on:click={connectToUnion}>Connect to Union in Leap 🚀</DemoButton>
          </div>
        {:else}
          <div>✅ Connected to Union Testnet</div>
          {#if !$sepUNOAdded}
            <div class="mt-4 text-sm">
              Optional:
              <button
                class="px-2 py-1 border-[1px] text-sm border-accent text-accent font-jetbrains"
                on:click={addUnoErc}
              >
                Track UNO on Sepolia in Metamask 💸
              </button>
            </div>
          {:else}
            <div>✅ Tracking UNO on Sepolia</div>
          {/if}
        {/if}
      {/if}
    {/if}
  {/if}
</TerminalContainer>

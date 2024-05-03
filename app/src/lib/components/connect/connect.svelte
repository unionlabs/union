<script lang="ts">
import { navigating } from "$app/stores"
import { slide } from "svelte/transition"
import Connection from "./connection.svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Sheet from "$lib/components/ui/sheet"
import { Button } from "$lib/components/ui/button"
import * as Avatar from "$lib/components/ui/avatar"
import { Separator } from "$lib/components/ui/separator"
import * as Collapsible from "$lib/components/ui/collapsible"
import ChevronsUpDown from "virtual:icons/lucide/chevrons-up-down"
import { sepoliaStore, evmWalletsInformation } from "$lib/wallet/evm/index.ts"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.ts"

/**
 * TODO: check both chains
 */
$: buttonText = $sepoliaStore.connectionStatus === "connected" ? "Connected" : "Connect Wallet"

let sheetOpen = false
$: if ($navigating) sheetOpen = false

let collapsibleOpen = true
</script>

<Sheet.Root bind:open={sheetOpen}>
  <Sheet.Trigger asChild let:builder>
    <Button
      variant="outline"
      builders={[builder]}
      on:click={() => (sheetOpen = !sheetOpen)}
      class={cn([
        'truncate max-w-44 space-x-2 px-4 text-lg',
        {
          'border-cyan-300/50':
            $sepoliaStore.connectionStatus === 'connected' ||
            $cosmosStore.connectionStatus === 'connected',
        },
      ])}
    >
      <span class="">{buttonText}</span>
    </Button>
  </Sheet.Trigger>
  <Sheet.Content
    class="border-solid border-white/20 min-w-[95%] sm:min-w-min sm:max-w-[475px] px-2"
  >
    <Sheet.Header class="mb-4 pl-2">
      <Sheet.Title>
        <!-- Connect Wallet -->
        <Avatar.Root
          class={cn(['size-8', { hidden: $sepoliaStore.connectionStatus !== 'connected' }])}
        >
          <Avatar.Image
            alt="ethereum avatar"
            src={`https://effigy.im/a/${$sepoliaStore.address}.png`}
          />
          <Avatar.Fallback>UN</Avatar.Fallback>
        </Avatar.Root>
      </Sheet.Title>
    </Sheet.Header>
    <Collapsible.Root
      open={true}
      tabindex={-1}
      onOpenChange={() => (collapsibleOpen = !collapsibleOpen)}
      class="focus:ring-0 ring-transparent focus-visible:ring-0 mb-0 pb-0"
    >
      <Collapsible.Trigger
        tabindex={-1}
        class={cn([
          'mb-3 font-bold w-full flex justify-between items-center align-middle transition-all active:scale-98 rounded-md px-2',
          'border-solid border-[1px] border-transparent hover:bg-white/10',
          { 'border-accent': !collapsibleOpen },
        ])}
      >
        <span class="mb-0.5 text-center w-full text-lg">Connect Wallets</span>
        <Button variant="ghost" size="sm" class="w-9 p-0 my-auto h-10 hover:bg-transparent">
          <ChevronsUpDown class="w-6 h-6" />
          <span class="sr-only">Toggle</span>
        </Button>
      </Collapsible.Trigger>
      <Collapsible.Content transition={node => slide(node, { duration: 300, delay: 50 })}>
        <Connection
          chain="evm"
          address={$sepoliaStore.address}
          hoverState={$sepoliaStore.hoverState}
          onConnectClick={sepoliaStore.connect}
          onDisconnectClick={sepoliaStore.disconnect}
          connectStatus={$sepoliaStore.connectionStatus}
          chainWalletsInformation={evmWalletsInformation}
          connectedWalletId={$sepoliaStore.connectedWallet}
        />
        <Separator class={cn(['bg-[#303033] my-1.5'])} />
        <Connection
          chain="cosmos"
          address={$cosmosStore.address}
          hoverState={$cosmosStore.hoverState}
          onConnectClick={cosmosStore.connect}
          onDisconnectClick={cosmosStore.disconnect}
          connectStatus={$cosmosStore.connectionStatus}
          chainWalletsInformation={cosmosWalletsInformation}
          connectedWalletId={$cosmosStore.connectedWallet}
        />
      </Collapsible.Content>
    </Collapsible.Root>
    <Separator class="mb-3 bg-[#303033]" />
  </Sheet.Content>
</Sheet.Root>

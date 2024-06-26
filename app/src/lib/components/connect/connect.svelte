<script lang="ts">
import { navigating } from "$app/stores"
import { slide } from "svelte/transition"
import Connection from "./connection.svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Sheet from "$lib/components/ui/sheet"
import { Button } from "$lib/components/ui/button"
import * as Avatar from "$lib/components/ui/avatar"
import WalletIcon from "virtual:icons/lucide/wallet"
import { Separator } from "$lib/components/ui/separator"
import * as Collapsible from "$lib/components/ui/collapsible"
import ThemeSwitch from "$lib/components/header/theme-switch.svelte"
import ChevronsUpDownIcon from "virtual:icons/lucide/chevrons-up-down"
import { sepoliaStore, evmWalletsInformation } from "$lib/wallet/evm/index.ts"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.ts"
import { Switch } from "$lib/components/ui/switch"
import { Label } from "$lib/components/ui/label"
import { showUnsupported } from "$lib/stores/user.ts"

let buttonText: string

$: if (
  $sepoliaStore.connectionStatus === "connected" &&
  $cosmosStore.connectionStatus === "connected"
) {
  buttonText = "Connected"
} else if (
  $sepoliaStore.connectionStatus === "connected" ||
  $cosmosStore.connectionStatus === "connected"
) {
  buttonText = "Connected (1/2)"
} else {
  buttonText = "Connect Wallet"
}

let sheetOpen = false
$: if ($navigating) sheetOpen = false

let collapsibleOpen = true
</script>

<Sheet.Root bind:open={sheetOpen}>
  <Sheet.Trigger asChild let:builder class="w-full">
    <Button
      size="sm"
      builders={[builder]}
      on:click={() => (sheetOpen = !sheetOpen)}
      class={cn(
        'space-x-2 w-[189px] text-md bg-accent text-black hover:bg-cyan-300/90',
        ($sepoliaStore.connectionStatus === 'connected' &&
          $cosmosStore.connectionStatus === 'connected')
      )}
    >
      <WalletIcon class="size-4 text-black" />
      <span class="font-supermolot font-bold uppercase">{buttonText}</span>
    </Button>
  </Sheet.Trigger>
  <Sheet.Content
    class="h-full border-solid border-left min-w-[95%] max-w-[90%] sm:min-w-min sm:max-w-[475px] flex flex-col justify-start"
  >
    <Sheet.Header class="mb-4 pl-2">
      <Sheet.Title class="flex gap-4 items-center">
        <!-- Connect Wallet -->
        <Avatar.Root
          class={cn('size-10', $sepoliaStore.connectionStatus !== 'connected' && 'hidden')}
        >
          <Avatar.Image
            alt="ethereum avatar"
            src={`https://effigy.im/a/${$sepoliaStore.address || '0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd'}.png`}
          />
          <Avatar.Fallback>UN</Avatar.Fallback>
        </Avatar.Root>
        <h2 class=" text-start w-full text-2xl font-bold uppercase font-supermolot">Connect Wallets</h2>
      </Sheet.Title>
    </Sheet.Header>
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
    <Separator class={cn('px-0 bg-border my-4')} />
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
    <div class="flex items-center space-x-2 mt-auto">
      <Switch id="unsupported-assets" bind:checked={$showUnsupported}/>
      <Label for="unsupported-assets">Show unverified assets</Label>
    </div>
    <ThemeSwitch />
  </Sheet.Content>
</Sheet.Root>


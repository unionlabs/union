<script lang="ts">
import { navigating } from "$app/stores"
import Connection from "./connection.svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import * as Sheet from "$lib/components/ui/sheet"
import { Button } from "$lib/components/ui/button"
import { Switch } from "$lib/components/ui/switch"
import * as Avatar from "$lib/components/ui/avatar"
import WalletIcon from "virtual:icons/lucide/wallet"
import { showUnsupported } from "$lib/stores/user.ts"
import { Separator } from "$lib/components/ui/separator"
import ThemeSwitch from "$lib/components/header/theme-switch.svelte"
import { evmStore, evmWalletsInformation } from "$lib/wallet/evm/index.ts"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.ts"

$: buttonText =
  $evmStore.connectionStatus === "connected" || $cosmosStore.connectionStatus === "connected"
    ? "Connected"
    : "Connect Wallet"

let sheetOpen = false
$: if ($navigating) sheetOpen = false
</script>

<Sheet.Root open={true}>
  <Sheet.Trigger asChild let:builder class="w-full">
    <Button
      size="sm"
      builders={[builder]}
      on:click={() => (sheetOpen = !sheetOpen)}
      class={cn(
        'space-x-2 text-md bg-accent text-black hover:bg-cyan-300/90',
        $evmStore.connectionStatus === 'connected' &&
          $cosmosStore.connectionStatus === 'connected',
      )}
    >
      <WalletIcon class="size-4 text-black" />
      <span class="font-supermolot font-bold uppercase">{buttonText}</span>
    </Button>
  </Sheet.Trigger>
  <Sheet.Content
    class="h-full border-solid border-left min-w-[95%] max-w-[92%] sm:min-w-min sm:max-w-[485px] flex flex-col justify-start sm:px-4 px-2"
  >
    <Sheet.Header class="mb-4 pl-2">
      <Sheet.Title class="flex gap-4 items-center">
        <!-- Connect Wallet -->
        <Avatar.Root
          class={cn('size-10', $evmStore.connectionStatus !== 'connected' && 'hidden')}
        >
          <Avatar.Image
            alt="ethereum avatar"
            src={$evmStore.address
              ? `https://ensdata.net/media/avatar/${$evmStore.address}`
              : 'https://effigy.im/a/0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd.png'}
          />
          <Avatar.Fallback>UN</Avatar.Fallback>
        </Avatar.Root>
        <h2 class=" text-start w-full text-2xl font-bold uppercase font-supermolot">
          Connect Wallets
        </h2>
      </Sheet.Title>
    </Sheet.Header>
    <Connection
      chain="evm"
      address={$evmStore.address}
      hoverState={$evmStore.hoverState}
      onConnectClick={evmStore.connect}
      onDisconnectClick={evmStore.disconnect}
      connectStatus={$evmStore.connectionStatus}
      chainWalletsInformation={evmWalletsInformation}
      connectedWalletId={$evmStore.connectedWallet}
    />
    <Separator class={cn('px-0 bg-border my-3')} />
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
    <div class="flex items-center space-x-2">
      <Switch id="unsupported-assets" bind:checked={$showUnsupported} />
      <Label for="unsupported-assets">Show unsupported assets</Label>
    </div>
    <ThemeSwitch />
  </Sheet.Content>
</Sheet.Root>

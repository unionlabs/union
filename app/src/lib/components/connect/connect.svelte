<script lang="ts">
import { setMode } from "mode-watcher"
import { navigating } from "$app/stores"
import Sun from "virtual:icons/lucide/sun"
import Moon from "virtual:icons/lucide/moon"
import Connection from "./connection.svelte"
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import * as Sheet from "$lib/components/ui/sheet"
import { Switch } from "$lib/components/ui/switch"
import { Button } from "$lib/components/ui/button"
import * as Avatar from "$lib/components/ui/avatar"
import WalletIcon from "virtual:icons/lucide/wallet"
import { showUnsupported } from "$lib/stores/user.ts"
import { crtEffectEnabled } from "$lib/stores/user.ts"
import * as DropdownMenu from "$lib/components/ui/dropdown-menu"
import { sepoliaStore, evmWalletsInformation } from "$lib/wallet/evm/index.ts"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/index.ts"

let buttonText: string
let connectedWallets = 0

$: if (
  $sepoliaStore.connectionStatus === "connected" &&
  $cosmosStore.connectionStatus === "connected"
) {
  buttonText = "Connected"
  connectedWallets = 2
} else if (
  $sepoliaStore.connectionStatus === "connected" ||
  $cosmosStore.connectionStatus === "connected"
) {
  buttonText = "Connected"
  connectedWallets = 1
} else {
  buttonText = "Connect Wallet"
  connectedWallets = 0
}

let sheetOpen = false
$: if ($navigating) sheetOpen = false
</script>

<Sheet.Root bind:open={sheetOpen}>
  <Sheet.Trigger asChild class="w-full" let:builder>
    <Button
      builders={[builder]}
      class={cn(
        connectedWallets === 1 ? "w-[75px]" : "w-[50px]",
        "space-x-1.5 lg:w-[180px] text-md bg-accent text-black ml-auto",
        "hover:bg-cyan-300/90",
        $sepoliaStore.connectionStatus === "connected" &&
          $cosmosStore.connectionStatus === "connected",
      )}
      on:click={() => (sheetOpen = !sheetOpen)}
      size="sm"
    >
      <WalletIcon class="size-6 text-black" />
      <span class="font-supermolot font-bold uppercase lg:block hidden">
        {buttonText}
      </span>
      <span class={cn(connectedWallets === 1 ? "font-supermolot font-bold uppercase" : "hidden")}>
        {connectedWallets === 1 ? "1/2" : ""}
      </span>
    </Button>
  </Sheet.Trigger>
  <Sheet.Content
    class={cn(
      "h-full border-solid border-left flex flex-col justify-start",
      "min-w-[95%] max-w-[90%] sm:min-w-min sm:max-w-[500px]",
      "overflow-y-auto",
    )}
  >
    <Sheet.Header>
      <Sheet.Title class="flex gap-4 items-center">
        <!-- Connect Wallet -->
        <Avatar.Root
          class={cn("size-10", $sepoliaStore.connectionStatus !== "connected" && "hidden")}
        >
          <Avatar.Image
            alt="ethereum avatar"
            src={`https://effigy.im/a/${$sepoliaStore.address || "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd"}.png`}
          />
          <Avatar.Fallback>UN</Avatar.Fallback>
        </Avatar.Root>
        <h2 class=" text-start w-full text-2xl font-bold uppercase font-supermolot">
          Connect Wallets
        </h2>
      </Sheet.Title>
    </Sheet.Header>
    <Connection
      address={$sepoliaStore.address}
      chain="evm"
      chainWalletsInformation={evmWalletsInformation}
      connectStatus={$sepoliaStore.connectionStatus}
      connectedWalletId={$sepoliaStore.connectedWallet}
      hoverState={$sepoliaStore.hoverState}
      onConnectClick={sepoliaStore.connect}
      onDisconnectClick={sepoliaStore.disconnect}
    />
    <Connection
      address={$cosmosStore.address}
      chain="cosmos"
      chainWalletsInformation={cosmosWalletsInformation}
      connectStatus={$cosmosStore.connectionStatus}
      connectedWalletId={$cosmosStore.connectedWallet}
      hoverState={$cosmosStore.hoverState}
      onConnectClick={cosmosStore.connect}
      onDisconnectClick={cosmosStore.disconnect}
    />
    <div class="flex items-center space-x-2">
      <Switch bind:checked={$showUnsupported} id="unsupported-assets" />
      <Label for="unsupported-assets">Show unverified assets</Label>
    </div>
    <div class="mt-auto flex justify-between">
      <div class="flex items-center space-x-2">
        <Switch bind:checked={$crtEffectEnabled} id="crt-effect-enabled" />
        <Label for="unsupported-assets">CRT effect</Label>
      </div>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild let:builder>
          <Button
            builders={[builder]}
            variant="default"
            size="icon"
            class="hover:text-black hover:bg-accent"
          >
            <Sun
              class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
            />
            <Moon
              class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
            />
            <span class="sr-only">Toggle theme</span>
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content class="w-fit rounded-none bg-secondary">
          <DropdownMenu.Group>
            <DropdownMenu.Item on:click={() => setMode("system")} class="cursor-pointer">
              System
            </DropdownMenu.Item>
            <DropdownMenu.Item on:click={() => setMode("dark")} class="cursor-pointer">
              Dark
            </DropdownMenu.Item>
            <DropdownMenu.Item on:click={() => setMode("light")} class="cursor-pointer">
              Light
            </DropdownMenu.Item>
          </DropdownMenu.Group>
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    </div>
  </Sheet.Content>
</Sheet.Root>

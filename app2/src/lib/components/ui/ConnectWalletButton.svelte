<script lang="ts">
import SharpWalletIcon from "$lib/components/icons/SharpWalletIcon.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import { Option } from "effect"
import Button from "./Button.svelte"
</script>

<Button
  variant="secondary"
  class="flex items-center w-full"
  onclick={() => uiStore.openWalletModal()}
>
  <SharpWalletIcon class="size-5" />
  My wallets
  <div class="flex items-center gap-1 ml-1 -mr-1">
    <div
      class="{Option.isSome(wallets.evmAddress) ? 'pulse-1 bg-green-500 shadow-[0_0_2px_1px_rgba(34,197,94,0.6)]' : 'bg-zinc-800'} w-2 h-2 rounded-full transition-colors duration-200"
      title="EVM"
    >
    </div>
    <div
      class="{Option.isSome(wallets.cosmosAddress) ? 'pulse-2 bg-green-500 shadow-[0_0_2px_1px_rgba(34,197,94,0.6)]' : 'bg-zinc-800'} w-2 h-2 rounded-full transition-colors duration-200"
      title="Cosmos"
    >
    </div>
  </div>
</Button>

<style>
@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.7; }
  100% { opacity: 1; }
}
.pulse-1 {
  animation: pulse 2s ease-in-out infinite;
}
.pulse-2 {
  animation: pulse 2s ease-in-out infinite;
  animation-delay: 0.3s;
}
</style>

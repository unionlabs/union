<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import { createEventDispatcher } from "svelte"
import type { EnhancedWallet } from "../stores/wallets.svelte"

type WalletCategory = "evm" | "cosmos" | "other"

const { wallet, category = "other" as WalletCategory } = $props<{
  wallet: EnhancedWallet
  category?: WalletCategory
}>()
const dispatch = createEventDispatcher<{ remove: string }>()

function getChainDisplayNameForAvatar(chainId: string): string {
  if (!chainId) {
    return "U" // Unknown
  }
  return chainId.charAt(0).toUpperCase() + chainId.slice(1)
}

function getChainDisplayNameForTitle(chainId: string, cat: WalletCategory): string {
  if (cat === "evm") {
    if (!chainId) {
      return "EVM Wallet"
    }
    return chainId.charAt(0).toUpperCase() + chainId.slice(1)
  }
  return ""
}

function handleRemove() {
  dispatch("remove", wallet.address)
}

function truncateAddress(address: string): string {
  if (!address || address.length <= 11) {
    return address || ""
  }
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}
</script>

<Card class="flex flex-col gap-3 p-4 transition-all hover:bg-zinc-800/50 group">
  <div class="flex items-center justify-between gap-2">
    <div class="flex items-center gap-3 min-w-0">
      <div class="w-6 h-6 rounded-full bg-zinc-700 flex items-center justify-center text-xs font-medium text-zinc-300 shrink-0">
        {getChainDisplayNameForAvatar(wallet.chain_id).substring(0, 1).toUpperCase()}
      </div>
      {#if category === "evm"}
        <span
          class="text-sm font-medium text-zinc-200 truncate"
          title={getChainDisplayNameForTitle(wallet.chain_id, category)}
        >
          {getChainDisplayNameForTitle(wallet.chain_id, category)}
        </span>
      {/if}
      {#if wallet.hasGrouping}
        <span
          title="Part of a wallet group"
          class="text-zinc-500 text-xs whitespace-nowrap shrink-0"
        >
          (Grouped)
        </span>
      {/if}
    </div>
    <button
      onclick={handleRemove}
      class="text-xs text-zinc-500 hover:text-red-500 transition-colors opacity-0 group-hover:opacity-100 p-1 -m-1 shrink-0"
      title="Remove wallet"
    >
      Remove
    </button>
  </div>

  <div
    class="text-xs text-zinc-400 font-mono break-all"
    title={wallet.address}
  >
    {truncateAddress(wallet.address)}
  </div>

  <div class="text-xs text-zinc-500 mt-1">
    Added: {
      new Date(wallet.createdAt).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      })
    }
  </div>
</Card>

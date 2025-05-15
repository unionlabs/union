<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Effect, Option, pipe } from "effect"
import type { EnhancedWallet } from "../stores/wallets.svelte"
import WalletRowSkeleton from "./WalletRowSkeleton.svelte"

const groupColors = [
  "border-blue-500",
  "border-green-500",
  "border-yellow-500",
  "border-purple-500",
  "border-pink-500",
  "border-indigo-500",
  "border-teal-500",
]
const groupColorCache = new Map<string, string>()
let nextColorIndex = 0

function getGroupColorClass(groupingId: string | null): string {
  if (!groupingId) {
    return "border-transparent"
  }
  const cachedColor = groupColorCache.get(groupingId)
  if (cachedColor) {
    return cachedColor
  }

  const color = groupColors[nextColorIndex % groupColors.length]
  groupColorCache.set(groupingId, color)
  nextColorIndex++
  return color
}

function getWalletCategory(chainId: string): "evm" | "cosmos" | "other" {
  if (!chainId) {
    return "other"
  }
  const lowerChainId = chainId.toLowerCase()
  if (lowerChainId === "evm") {
    return "evm"
  }
  if (lowerChainId.startsWith("cosmos:")) {
    return "cosmos"
  }
  return "other"
}

function truncateAddress(address: string): string {
  if (!address || address.length <= 11) {
    return address || ""
  }
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

function truncateGroupingId(groupingId: string | null): string {
  if (!groupingId) {
    return "-" // Display dash if no group
  }
  if (groupingId.length <= 11) {
    return groupingId
  }
  return `${groupingId.slice(0, 6)}...${groupingId.slice(-4)}`
}

// Helper function to group wallets for card-in-card display
function groupWalletsForDisplay(wallets: EnhancedWallet[]): EnhancedWallet[][] {
  if (!wallets || wallets.length === 0) {
    return []
  }
  const groups: Record<string, EnhancedWallet[]> = {}
  const ungroupedStandalone: EnhancedWallet[][] = []

  wallets.forEach(wallet => {
    if (wallet.grouping) {
      if (!groups[wallet.grouping]) {
        groups[wallet.grouping] = []
      }
      groups[wallet.grouping].push(wallet)
    } else {
      ungroupedStandalone.push([wallet])
    }
  })
  // Ensure groups with multiple wallets come first, then ungrouped
  const sortedGroups = Object.values(groups).sort((a, b) => b.length - a.length)
  return [...sortedGroups, ...ungroupedStandalone]
}

let evmWalletGroups = $derived(
  Option.match(dashboard.wallets, {
    onNone: () => [],
    onSome: (walletStore) =>
      Option.match(walletStore.wallets, {
        onNone: () => [],
        onSome: (_w) =>
          groupWalletsForDisplay(
            walletStore.enhanced.filter((w: EnhancedWallet) =>
              getWalletCategory(w.chain_id) === "evm"
            ),
          ),
      }),
  }),
)

let cosmosWalletGroups = $derived(
  Option.match(dashboard.wallets, {
    onNone: () => [],
    onSome: (walletStore) =>
      Option.match(walletStore.wallets, {
        onNone: () => [],
        onSome: (_w) =>
          groupWalletsForDisplay(
            walletStore.enhanced.filter((w: EnhancedWallet) =>
              getWalletCategory(w.chain_id) === "cosmos"
            ),
          ),
      }),
  }),
)

let otherWalletGroups = $derived(
  Option.match(dashboard.wallets, {
    onNone: () => [],
    onSome: (walletStore) =>
      Option.match(walletStore.wallets, {
        onNone: () => [],
        onSome: (_w) =>
          groupWalletsForDisplay(
            walletStore.enhanced.filter((w: EnhancedWallet) =>
              getWalletCategory(w.chain_id) === "other"
            ),
          ),
      }),
  }),
)

let isLoadingStore = $derived(
  Option.match(dashboard.wallets, {
    onNone: () => true,
    onSome: (walletStore) => Option.isNone(walletStore.wallets),
  }),
)

let selectedWallets = $state(new Set<string>()) // Stores address:chainId for uniqueness
let İsRemoving = $state(false)

function toggleWalletSelection(walletAddress: string, chainId: string) {
  const key = `${walletAddress}::${chainId}` // Use :: to avoid issues if address/chainId contain :
  const newSet = new Set(selectedWallets)
  if (newSet.has(key)) {
    newSet.delete(key)
  } else {
    newSet.add(key)
  }
  selectedWallets = newSet
}

function handleRemoveSelectedWallets() {
  if (Option.isNone(dashboard.wallets) || selectedWallets.size === 0 || İsRemoving) {
    return
  }
  İsRemoving = true
  const walletStore = dashboard.wallets.value

  const removalEffects = Array.from(selectedWallets).map(key => {
    const [address /* chainId */] = key.split("::")
    return walletStore.removeWallet(address)
  })

  Effect.runPromise(
    pipe(
      Effect.all(removalEffects, { concurrency: 3, discard: true }),
      Effect.tap(() => {
        selectedWallets = new Set<string>()
      }),
      Effect.catchAll((error) => {
        console.error("Error removing selected wallets:", error)
        return Effect.void // Keep selection on error for user to retry or see
      }),
      Effect.ensuring(Effect.sync(() => {
        İsRemoving = false
      })),
    ),
  )
}
</script>

<div class="flex flex-col gap-6">
  <!-- EVM Wallets Section -->
  <Card class="flex flex-col">
    <h2 class="text-sm font-medium text-zinc-200 mb-3">EVM Wallets</h2>
    <div class="flex flex-col gap-3">
      {#if isLoadingStore}
        {#each Array(1) as _} <WalletRowSkeleton /> {/each}
      {:else if evmWalletGroups.length > 0}
        {#each evmWalletGroups as walletGroup (walletGroup[0].address + walletGroup[0].chain_id)}
          <Card
            class="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 transition-all duration-200 hover:border-zinc-600/50"
          >
            <div class="flex flex-col gap-2">
              {#each walletGroup as wallet (wallet.address + wallet.chain_id)}
                <div class="flex items-center justify-between gap-3 pt-1 first:pt-0">
                  <div class="min-w-0 flex-grow flex items-center">
                    <span
                      class="font-mono text-sm text-zinc-300 truncate block"
                      title={wallet.address}
                    >{truncateAddress(wallet.address)}</span>
                  </div>
                  <label class="flex items-center cursor-pointer shrink-0">
                    <input
                      type="checkbox"
                      class="opacity-0 absolute w-1 h-1 peer"
                      checked={selectedWallets.has(`${wallet.address}::${wallet.chain_id}`)}
                      onchange={() => toggleWalletSelection(wallet.address, wallet.chain_id)}
                      aria-label="Select wallet {truncateAddress(wallet.address)}"
                      disabled={İsRemoving}
                    />
                    <span
                      class="h-4 w-4 rounded border border-zinc-600 bg-transparent flex items-center justify-center transition-colors peer-checked:border-accent peer-focus:ring-2 peer-focus:ring-accent peer-focus:ring-offset-2 peer-focus:ring-offset-zinc-900 hover:border-zinc-500"
                    >
                      {#if selectedWallets.has(`${wallet.address}::${wallet.chain_id}`)}
                        <svg
                          class="w-2.5 h-2.5 text-accent pointer-events-none"
                          viewBox="0 0 16 16"
                          fill="currentColor"
                        >
                          <path d="M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z" />
                        </svg>
                      {/if}
                    </span>
                  </label>
                </div>
              {/each}
            </div>
          </Card>
        {/each}
      {:else}
        <div class="text-center py-8">
          <div class="text-zinc-400 mb-1">No EVM Wallets Found</div>
          <div class="text-xs text-zinc-500">Add an EVM compatible wallet to see it here.</div>
        </div>
      {/if}
    </div>
  </Card>

  <!-- Cosmos Wallets Section -->
  <Card class="flex flex-col">
    <h2 class="text-sm font-medium text-zinc-200 mb-3">Cosmos Wallets</h2>
    <div class="flex flex-col gap-3">
      {#if isLoadingStore}
        {#each Array(1) as _} <WalletRowSkeleton /> {/each}
      {:else if cosmosWalletGroups.length > 0}
        {#each cosmosWalletGroups as
          walletGroup
          (walletGroup[0].grouping || walletGroup[0].address + walletGroup[0].chain_id)
        }
          <Card
            class="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 transition-all duration-200 hover:border-zinc-600/50"
          >
            <div class="flex flex-col gap-2">
              {#each walletGroup as wallet (wallet.address + wallet.chain_id)}
                <div class="flex items-center justify-between gap-3 pt-1 first:pt-0">
                  <div class="min-w-0 flex-grow flex items-center">
                    <span
                      class="font-mono text-sm text-zinc-300 truncate block"
                      title={wallet.address}
                    >{truncateAddress(wallet.address)}</span>
                  </div>
                  <label class="flex items-center cursor-pointer shrink-0">
                    <input
                      type="checkbox"
                      class="opacity-0 absolute w-1 h-1 peer"
                      checked={selectedWallets.has(`${wallet.address}::${wallet.chain_id}`)}
                      onchange={() => toggleWalletSelection(wallet.address, wallet.chain_id)}
                      aria-label="Select wallet {truncateAddress(wallet.address)}"
                      disabled={İsRemoving}
                    />
                    <span
                      class="h-4 w-4 rounded border border-zinc-600 bg-transparent flex items-center justify-center transition-colors peer-checked:border-accent peer-focus:ring-2 peer-focus:ring-accent peer-focus:ring-offset-2 peer-focus:ring-offset-zinc-900 hover:border-zinc-500"
                    >
                      {#if selectedWallets.has(`${wallet.address}::${wallet.chain_id}`)}
                        <svg
                          class="w-2.5 h-2.5 text-accent pointer-events-none"
                          viewBox="0 0 16 16"
                          fill="currentColor"
                        >
                          <path d="M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z" />
                        </svg>
                      {/if}
                    </span>
                  </label>
                </div>
              {/each}
            </div>
          </Card>
        {/each}
      {:else}
        <div class="text-center py-8">
          <div class="text-zinc-400 mb-1">No Cosmos Wallets Found</div>
          <div class="text-xs text-zinc-500">Add a Cosmos compatible wallet.</div>
        </div>
      {/if}
    </div>
  </Card>

  <!-- Other Wallets Section (Optional) -->
  {#if !isLoadingStore && otherWalletGroups.length > 0}
    <Card class="flex flex-col">
      <h2 class="text-sm font-medium text-zinc-200 mb-3">Other Wallets</h2>
      <div class="flex flex-col gap-3">
        {#each otherWalletGroups as walletGroup (walletGroup[0].address + walletGroup[0].chain_id)}
          <Card
            class="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 transition-all duration-200 hover:border-zinc-600/50"
          >
            <div class="flex flex-col gap-2">
              {#each walletGroup as wallet (wallet.address + wallet.chain_id)}
                <div class="flex items-center justify-between gap-3 pt-1 first:pt-0">
                  <div class="min-w-0 flex-grow flex items-center">
                    <span
                      class="font-mono text-sm text-zinc-300 truncate block"
                      title={wallet.address}
                    >{truncateAddress(wallet.address)}</span>
                  </div>
                  <label class="flex items-center cursor-pointer shrink-0">
                    <input
                      type="checkbox"
                      class="opacity-0 absolute w-1 h-1 peer"
                      checked={selectedWallets.has(`${wallet.address}::${wallet.chain_id}`)}
                      onchange={() => toggleWalletSelection(wallet.address, wallet.chain_id)}
                      aria-label="Select wallet {truncateAddress(wallet.address)}"
                      disabled={İsRemoving}
                    />
                    <span
                      class="h-4 w-4 rounded border border-zinc-600 bg-transparent flex items-center justify-center transition-colors peer-checked:border-accent peer-focus:ring-2 peer-focus:ring-accent peer-focus:ring-offset-2 peer-focus:ring-offset-zinc-900 hover:border-zinc-500"
                    >
                      {#if selectedWallets.has(`${wallet.address}::${wallet.chain_id}`)}
                        <svg
                          class="w-2.5 h-2.5 text-accent pointer-events-none"
                          viewBox="0 0 16 16"
                          fill="currentColor"
                        >
                          <path d="M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z" />
                        </svg>
                      {/if}
                    </span>
                  </label>
                </div>
              {/each}
            </div>
          </Card>
        {/each}
      </div>
    </Card>
  {/if}

  {#if selectedWallets.size > 0}
    <div class="mt-6 flex justify-end">
      <Button
        variant="danger"
        onclick={handleRemoveSelectedWallets}
        disabled={İsRemoving || isLoadingStore}
      >
        {#if İsRemoving}Removing...{:else}Remove Selected ({selectedWallets.size}){/if}
      </Button>
    </div>
  {/if}
</div>

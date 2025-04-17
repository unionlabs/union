<script lang="ts">
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { Option } from "effect"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { uiStore } from "$lib/stores/ui.svelte.ts"
import { cn } from "$lib/utils"
import { onDestroy, onMount } from "svelte"
import { crossfade, fade, fly } from "svelte/transition"
import Button from "$lib/components/ui/Button.svelte"
import SharpChevronLeftIcon from "$lib/components/icons/SharpChevronLeftIcon.svelte"
import FilledBookmarkIcon from "$lib/components/icons/FilledBookmarkIcon.svelte"
import OutlinedBookmarkIcon from "$lib/components/icons/OutlinedBookmarkIcon.svelte"
import SharpCancelIcon from "$lib/components/icons/SharpCancelIcon.svelte"
import SharpListRemoveIcon from "$lib/components/icons/SharpListRemoveIcon.svelte"
import RestoreIcon from "$lib/components/icons/RestoreIcon.svelte"
import SharpWalletIcon from "$lib/components/icons/SharpWalletIcon.svelte"
import { getDerivedReceiverSafe } from "$lib/services/shared"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { clickOutside } from "$lib/utils/actions.ts"

let destinationChain = $derived(
  Option.isSome(transferData.destinationChain) ? Option.getOrNull(transferData.destinationChain) : null
)
let destinationChainId = $derived(destinationChain ? destinationChain.universal_chain_id : null)
let hasWalletAddress = $derived(
  destinationChain && Option.isSome(wallets.getAddressForChainWithOutInput(destinationChain))
)

$effect(() => {
  if (destinationChain && destinationChainId) {
    // First try to use connected wallet if available
    if (hasWalletAddress) {
      const address = wallets.getAddressForChain(destinationChain)
      const addressValue = Option.getOrNull(address)
      if (addressValue && !transferData.raw.receiver) {
        transferData.raw.updateField("receiver", addressValue)
        return
      }
    }

    if (recentAddresses[destinationChainId]?.length > 0) {
      const mostRecentAddress = recentAddresses[destinationChainId][0]
      if (mostRecentAddress && !transferData.raw.receiver) {
        transferData.raw.updateField("receiver", mostRecentAddress)
      }
    }
  }
})

let isModalOpen = $state(false)
let currentView = $state("main") // "main", "recent", "bookmarks"
let previousView = $state("main") // Track previous view for transitions
let manualAddress = $state("")
let showClearConfirm = $state(false)
let bookmarkOnAdd = $state(false)

let recentAddresses: Record<string, Array<string>> = $state({})
let bookmarkedAddresses: Record<string, Array<string>> = $state({})

// Create crossfade transition
const [send, receive] = crossfade({
  duration: 200,
  fallback(node) {
    return fly(node, { delay: 0, duration: 200, y: 20 })
  }
})

// Handle keyboard events
function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && isModalOpen) {
    closeModal()
  }
}

// Load addresses from localStorage
onMount(() => {
  try {
    const savedRecent = localStorage.getItem("recentAddresses")
    if (savedRecent) recentAddresses = JSON.parse(savedRecent)

    const savedBookmarked = localStorage.getItem("bookmarkedAddresses")
    if (savedBookmarked) bookmarkedAddresses = JSON.parse(savedBookmarked)
  } catch (e) {
    console.error("Error loading addresses from localStorage", e)
  }

  document.addEventListener("keydown", handleKeydown)
})

onDestroy(() => {
  document.removeEventListener("keydown", handleKeydown)
})

function saveAddress(address: string, isBookmarked = false) {
  if (!destinationChainId) return

  // Save to recent addresses
  if (!recentAddresses[destinationChainId]) {
    recentAddresses[destinationChainId] = []
  }

  // Remove if already exists (to move to top)
  const existingIndex = recentAddresses[destinationChainId].indexOf(address)
  if (existingIndex > -1) {
    recentAddresses[destinationChainId].splice(existingIndex, 1)
  }

  // Add to the beginning
  recentAddresses[destinationChainId].unshift(address)

  // Keep only the last 5 addresses
  if (recentAddresses[destinationChainId].length > 5) {
    recentAddresses[destinationChainId] = recentAddresses[destinationChainId].slice(0, 5)
  }

  // Save to localStorage
  localStorage.setItem("recentAddresses", JSON.stringify(recentAddresses))

  // Handle bookmarks
  if (isBookmarked) {
    if (!bookmarkedAddresses[destinationChainId]) {
      bookmarkedAddresses[destinationChainId] = []
    }

    if (!bookmarkedAddresses[destinationChainId].includes(address)) {
      bookmarkedAddresses[destinationChainId].push(address)
      localStorage.setItem("bookmarkedAddresses", JSON.stringify(bookmarkedAddresses))
    }
  }
}

function toggleBookmark(address: string) {
  if (!destinationChainId) return

  if (!bookmarkedAddresses[destinationChainId]) {
    bookmarkedAddresses[destinationChainId] = []
  }

  const index = bookmarkedAddresses[destinationChainId].indexOf(address)
  if (index > -1) {
    // Remove from bookmarks
    bookmarkedAddresses[destinationChainId].splice(index, 1)
  } else {
    // Add to bookmarks
    bookmarkedAddresses[destinationChainId].push(address)
  }

  localStorage.setItem("bookmarkedAddresses", JSON.stringify(bookmarkedAddresses))
}

function removeAddress(address: string, type: "recent" | "bookmarked") {
  if (!destinationChainId) return

  if (type === "recent" && recentAddresses[destinationChainId]) {
    const index = recentAddresses[destinationChainId].indexOf(address)
    if (index > -1) {
      recentAddresses[destinationChainId].splice(index, 1)
      localStorage.setItem("recentAddresses", JSON.stringify(recentAddresses))
    }
  } else if (type === "bookmarked" && bookmarkedAddresses[destinationChainId]) {
    const index = bookmarkedAddresses[destinationChainId].indexOf(address)
    if (index > -1) {
      bookmarkedAddresses[destinationChainId].splice(index, 1)
      localStorage.setItem("bookmarkedAddresses", JSON.stringify(bookmarkedAddresses))
    }
  }
}

function clearAddresses(type: "recent" | "bookmarked") {
  if (!destinationChainId) return

  if (type === "recent") {
    recentAddresses[destinationChainId] = []
    localStorage.setItem("recentAddresses", JSON.stringify(recentAddresses))
  } else if (type === "bookmarked") {
    bookmarkedAddresses[destinationChainId] = []
    localStorage.setItem("bookmarkedAddresses", JSON.stringify(bookmarkedAddresses))
  }

  showClearConfirm = false
}

function isBookmarked(address: string): boolean {
  if (!destinationChainId) return false
  return bookmarkedAddresses[destinationChainId]?.includes(address)
}

function useAddress(address: string, shouldBookmark = false) {
  // Update the transferData receiver
  transferData.raw.updateField("receiver", address)

  // Always save to recent addresses
  if (shouldBookmark || bookmarkOnAdd) {
    // First add to bookmarks if requested
    if (!bookmarkedAddresses[destinationChainId]) {
      bookmarkedAddresses[destinationChainId] = []
    }

    if (!bookmarkedAddresses[destinationChainId].includes(address)) {
      bookmarkedAddresses[destinationChainId].push(address)
      localStorage.setItem("bookmarkedAddresses", JSON.stringify(bookmarkedAddresses))
    }
  }

  // Then save to recent (this ensures it's in both places if needed)
  saveAddress(address)

  closeModal()
}

function useConnectedWallet() {
  if (hasWalletAddress && destinationChain) {
    const address = wallets.getAddressForChain(destinationChain)
    const addressValue = Option.getOrNull(address)
    if (addressValue) {
      useAddress(addressValue, bookmarkOnAdd)
    }
  } else {
    uiStore.walletModalOpen = true
  }
}

function submitManualAddress() {
  if (manualAddress.trim()) {
    const derivedReceiverAddr = getDerivedReceiverSafe(manualAddress)
    const derived = Option.getOrNull(derivedReceiverAddr)
    manualAddress = derived ?? manualAddress
    useAddress(manualAddress.trim(), bookmarkOnAdd)
  }
}

function toggleBookmarkOnAdd() {
  bookmarkOnAdd = !bookmarkOnAdd
}

function openModal() {
  isModalOpen = true
  currentView = "main"
  previousView = "main"
  bookmarkOnAdd = false
}

function closeModal() {
  isModalOpen = false
  manualAddress = ""
  currentView = "main"
  previousView = "main"
  showClearConfirm = false
  bookmarkOnAdd = false
}

function goBack() {
  if (currentView === "main") {
    closeModal()
  } else {
    previousView = currentView
    currentView = "main"
    showClearConfirm = false
  }
}

function showRecent() {
  previousView = currentView
  currentView = "recent"
}

function showBookmarks() {
  previousView = currentView
  currentView = "bookmarks"
}

function resetReceiver() {
  transferData.raw.updateField("receiver", "")
}

function hasRecent() {
  return destinationChainId && recentAddresses[destinationChainId]?.length > 0
}

function hasBookmarks() {
  return destinationChainId && bookmarkedAddresses[destinationChainId]?.length > 0
}
</script>

<Button class="w-fit" onclick={openModal} disabled={!destinationChainId}>
  <SharpWalletIcon class="size-5"/>
</Button>

<!-- Modal -->
{#if isModalOpen}
  <div class="absolute bg-zinc-925 inset-0 z-40"
       transition:fade={{ duration: 300 }}
       use:clickOutside
       onClickOutside={goBack}
  >
    <div class="w-full h-full max-h-full flex flex-col" transition:fly={{ y: 30, duration: 300, opacity: 0 }}>

      <div class="border-b border-zinc-800 flex justify-between items-center h-12 flex-shrink-0 p-4">
        <div class="flex items-center h-full">
          <button
                  aria-label="Back"
                  onclick={goBack}
                  class="mr-3 flex items-center text-zinc-400 hover:text-zinc-200 cursor-pointer h-full"
          >
            <SharpChevronLeftIcon class="size-6"/>
            <div class="ml-2 flex items-center">
              <div class="relative w-32 h-8 flex items-center">
                {#if currentView === "main"}
                  <span
                          class="text-lg text-zinc-100 absolute"
                          in:receive={{key: 'receiver'}}
                          out:send={{key: 'receiver'}}
                  >
                    Receiver
                  </span>
                {:else if currentView === "recent"}
                  <span
                          class="text-lg text-zinc-100 absolute"
                          in:receive={{key: 'recent'}}
                          out:send={{key: 'recent'}}
                  >
                    Recent
                  </span>
                {:else if currentView === "bookmarks"}
                  <span
                          class="text-lg text-zinc-100 absolute"
                          in:receive={{key: 'bookmarked'}}
                          out:send={{key: 'bookmarked'}}
                  >
                    Bookmarked
                  </span>
                {/if}
              </div>
            </div>
          </button>
        </div>

        {#if (currentView === "recent" && hasRecent()) || (currentView === "bookmarks" && hasBookmarks())}
          <div class="flex items-center h-full">
            <button
                    onclick={() => showClearConfirm = true}
                    class="text-zinc-400 hover:text-zinc-200 cursor-pointer p-2 rounded flex items-center h-full"
                    aria-label="Clear all"
            >
              <SharpListRemoveIcon/>
              <span class="ml-1 text-sm">Clear</span>
            </button>
          </div>
        {/if}
      </div>

      {#if showClearConfirm}
        <div class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
          <div class="bg-zinc-800 p-4 rounded-lg w-4/5 max-w-md">
            <h4 class="text-lg font-semibold mb-3">Clear all {currentView === "recent" ? "recent" : "bookmarked"}
              addresses?</h4>
            <p class="mb-4 text-zinc-300">This action cannot be undone.</p>
            <div class="flex justify-end gap-3">
              <button
                      onclick={() => showClearConfirm = false}
                      class="px-4 py-2 bg-zinc-700 text-zinc-200 rounded hover:bg-zinc-600"
              >
                Cancel
              </button>
              <button
                      onclick={() => clearAddresses(currentView === "recent" ? "recent" : "bookmarked")}
                      class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
              >
                Clear All
              </button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Content area with transitions -->
      <div class="flex-grow relative overflow-hidden">
        {#if currentView === "main"}
          <div
                  class="flex flex-col justify-between h-full p-4 absolute inset-0"
                  in:fly={{ x: previousView !== "main" ? -20 : 0, duration: 300, opacity: 0 }}
                  out:fly={{ x: -20, duration: 300, opacity: 0 }}
          >
            <!-- Manual address input -->
            <div>
              <div class="flex flex-col gap-2">
                <div class="flex flex-col gap-2 h-10">
                  <input
                          type="text"
                          bind:value={manualAddress}
                          placeholder="Enter receiver address"
                          class={cn(
                      "w-full p-2 py-5 rounded-md bg-zinc-800 text-zinc-200 h-full text-center",
                      "focus:outline-none focus:ring-1 focus:ring-babylon-orange",
                    )}
                  />
                  <div class="flex flex-1 gap-2 w-full">
                    <Button
                            class="h-10 flex-1"
                            disabled={!manualAddress.trim()} onclick={submitManualAddress}>
                      Use
                    </Button>
                    <Button
                            class="h-10 px-2"
                            disabled={!manualAddress.trim()}
                            onclick={toggleBookmarkOnAdd}
                            aria-label={bookmarkOnAdd ? "Remove bookmark on add" : "Add bookmark on add"}
                    >

                      {#if bookmarkOnAdd}
                        <FilledBookmarkIcon class="size-5"/>
                      {:else}
                        <OutlinedBookmarkIcon class="size-5"/>
                      {/if}
                    </Button>
                  </div>
                </div>
                {#if bookmarkOnAdd}
                  <div class="text-xs text-zinc-400">Address will be bookmarked when added</div>
                {/if}
              </div>
            </div>

            <!--BUTTONS-->
            <div class="flex flex-col gap-4">
              <!-- Connected wallet option -->
              <Button class="justify-between py-5" variant="secondary" onclick={useConnectedWallet}>
                <span class="flex items-center gap-2">
                  <SharpWalletIcon class="size-5"/>
                  {#if hasWalletAddress}
                  Connected Wallet
                {:else}
                  Connect a Wallet
                {/if}
                </span>
              </Button>

              <!-- Recent Addresses Button -->
              <Button class="justify-between py-5" variant="secondary" onclick={showRecent} disabled={!hasRecent()}>
                <span class="flex items-center gap-2">
                  <RestoreIcon class="size-5" />Recent
                </span>
                {#if hasRecent()}
                  <span class="px-3 py-1 -mr-1 text-xs bg-zinc-800 rounded text-white">{recentAddresses[destinationChainId].length}</span>
                {/if}
              </Button>

              <!-- Bookmarked Addresses Button -->
              <Button class="justify-between py-5" variant="secondary" onclick={showBookmarks} disabled={!hasBookmarks()}>
                <span class="flex items-center gap-2">
                  <FilledBookmarkIcon class="size-5"/> Bookmarked
                </span>

                {#if hasBookmarks()}
                  <span class="px-3 py-1 -mr-1 text-xs bg-zinc-800 rounded text-white">{bookmarkedAddresses[destinationChainId].length}</span>
                {/if}
              </Button>
            </div>
          </div>
        {:else if currentView === "recent"}
          <!-- Recent Addresses View -->
          <div
                  class="overflow-y-auto p-4 absolute inset-0 h-full"
                  in:fly={{ x: previousView === "main" ? 20 : -20, duration: 300, opacity: 0 }}
                  out:fly={{ x: previousView === "main" ? -20 : 20, duration: 300, opacity: 0 }}
          >
            {#if destinationChainId && recentAddresses[destinationChainId]?.length > 0}
              <div class="space-y-2">
                {#each recentAddresses[destinationChainId] as address}
                  <div class="flex items-center justify-between px-4 py-3 bg-zinc-900 hover:bg-zinc-800 transition-colors cursor-pointer rounded">
                    <button
                            onclick={() => useAddress(address)}
                            class="text-left flex-grow truncate text-zinc-200 hover:text-white text-sm cursor-pointer"
                    >
                  <AddressComponent
                    address={address}
                    chain={destinationChain}
                  />
                    </button>
                    <div class="flex items-center ml-2">
                      <button
                              onclick={() => toggleBookmark(address)}
                              class="cursor-pointer p-1"
                              aria-label={isBookmarked(address) ? "Remove bookmark" : "Add bookmark"}
                      >
                        {#if isBookmarked(address)}
                          <FilledBookmarkIcon/>
                        {:else}
                          <OutlinedBookmarkIcon/>
                        {/if}
                      </button>
                      <button
                              onclick={() => removeAddress(address, "recent")}
                              class="cursor-pointer p-1 text-zinc-400 hover:text-zinc-200"
                              aria-label="Remove from recent"
                      >
                        <SharpCancelIcon/>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center py-8 text-zinc-400">
                No recent addresses found
              </div>
            {/if}
          </div>
        {:else if currentView === "bookmarks"}
          <!-- Bookmarked Addresses View -->
          <div
                  class="overflow-y-auto p-4 absolute inset-0 h-full"
                  in:fly={{ x: previousView === "main" ? 20 : -20, duration: 300, opacity: 0 }}
                  out:fly={{ x: previousView === "main" ? -20 : 20, duration: 300, opacity: 0 }}
          >
            {#if destinationChainId && bookmarkedAddresses[destinationChainId]?.length > 0}
              <div class="space-y-2">
                {#each bookmarkedAddresses[destinationChainId] as address}
                  <div class="flex items-center justify-between px-4 py-3 bg-zinc-900 hover:bg-zinc-800 transition-colors rounded cursor-pointer">
                    <button
                            onclick={() => useAddress(address)}
                            class="text-left flex-grow truncate text-zinc-200 hover:text-white cursor-pointer"
                    >
                      {address}
                    </button>
                    <div class="flex items-center ml-2">
                      <button
                              onclick={() => toggleBookmark(address)}
                              class="cursor-pointer p-1"
                              aria-label="Remove bookmark"
                      >
                        <FilledBookmarkIcon/>
                      </button>
                      <button
                              onclick={() => removeAddress(address, "bookmarked")}
                              class="cursor-pointer p-1 text-zinc-400 hover:text-zinc-200"
                              aria-label="Remove from bookmarks"
                      >
                        <SharpCancelIcon/>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center py-8 text-zinc-400">
                No bookmarked addresses found
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
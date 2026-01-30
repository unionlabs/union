<script lang="ts">
import { goto } from "$app/navigation"
import SearchIcon from "@lucide/svelte/icons/search"
import WalletIcon from "@lucide/svelte/icons/wallet"
import CornerMarks from "$lib/components/corner-marks.svelte"

let address = $state("")
let error = $state("")

function handleSubmit(e: Event) {
  e.preventDefault()
  const trimmed = address.trim()

  if (!trimmed) {
    error = "Please enter an address"
    return
  }

  // Basic validation - Union addresses start with "union" or "unionvaloper"
  if (!trimmed.startsWith("union")) {
    error = "Invalid address format. Union addresses start with 'union'"
    return
  }

  error = ""
  goto(`/account/${trimmed}`)
}
</script>


<div class="max-w-2xl mx-auto py-12">
  <div class="relative border border-border">
    <CornerMarks />

    <!-- Header -->
    <div class="flex items-center gap-3 px-4 py-3 border-b border-border bg-muted/20">
      <WalletIcon class="h-4 w-4 text-muted-foreground" />
      <span class="text-xs font-medium uppercase tracking-wider">Account Lookup</span>
    </div>

    <!-- Content -->
    <div class="p-6">
      <p class="text-sm text-muted-foreground mb-6">
        Enter a Union address to view account details, balances, delegations, and transaction history.
      </p>

      <form onsubmit={handleSubmit}>
        <div class="relative mb-4">
          <input
            type="text"
            bind:value={address}
            placeholder="union1..."
            class="w-full h-12 px-4 pr-12 font-mono text-sm bg-background border border-border focus:border-foreground focus:outline-none transition-colors"
          />
          <button
            type="submit"
            class="absolute right-2 top-1/2 -translate-y-1/2 p-2 hover:bg-muted transition-colors"
          >
            <SearchIcon class="h-4 w-4" />
          </button>
        </div>

        {#if error}
          <p class="text-sm text-destructive mb-4">{error}</p>
        {/if}

        <div class="flex items-center gap-4 text-xs text-muted-foreground">
          <span>Examples:</span>
          <button
            type="button"
            onclick={() => { address = "union1jk9psyhvgkrt2cumz8eytpn4a7dcsz46zg7xj4"; error = "" }}
            class="font-mono hover:text-foreground transition-colors"
          >
            union1jk9...7xj4
          </button>
        </div>
      </form>
    </div>
  </div>

  <!-- Recent lookups could go here -->
  <div class="mt-8 text-center text-xs text-muted-foreground">
    <p>Tip: You can also access accounts directly via URL:</p>
    <code class="font-mono text-foreground/60">/account/union1...</code>
  </div>
</div>

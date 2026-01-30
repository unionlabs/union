<script lang="ts">
import { goto } from "$app/navigation"
import * as Command from "$lib/components/ui/command/index.js"
import * as Dialog from "$lib/components/ui/dialog/index.js"
import { onMount } from "svelte"
import { runPromise } from "$lib/runtime"
import { chainStore } from "$lib/stores/chain.svelte"
import { fetchBlockByHeight } from "$lib/queries/blocks"
import { fetchTransaction } from "$lib/queries/transactions"
import { fetchValidators, fetchValidator } from "$lib/queries/validators"
import { fetchAccount, fetchBalances } from "$lib/queries/accounts"
import { formatTimeAgo, truncateHash, truncateAddress, formatAmount } from "$lib/utils/format"
import { prefetchValidatorAvatars, getCachedAvatar } from "$lib/utils/validators.svelte"
import type { Block, TxResponse, Validator, Account } from "$lib/types/cosmos"
import SearchIcon from "@lucide/svelte/icons/search"
import BoxIcon from "@lucide/svelte/icons/box"
import ArrowLeftRightIcon from "@lucide/svelte/icons/arrow-left-right"
import ShieldIcon from "@lucide/svelte/icons/shield"
import WalletIcon from "@lucide/svelte/icons/wallet"
import VoteIcon from "@lucide/svelte/icons/vote"
import CheckIcon from "@lucide/svelte/icons/check"
import XIcon from "@lucide/svelte/icons/x"
import LoaderIcon from "@lucide/svelte/icons/loader"

let open = $state(false)
let searchValue = $state("")
let isSearching = $state(false)

// Search results
let blockResult = $state<Block | null>(null)
let txResult = $state<TxResponse | null>(null)
let validatorResults = $state<Validator[]>([])
let accountResult = $state<{ account: Account; balance?: string } | null>(null)

// Trigger re-render when avatars load
let avatarsVersion = $state(0)

// Debounce timer
let searchTimer: ReturnType<typeof setTimeout> | null = null

// Setup keyboard shortcut
onMount(() => {
  const handleKeydown = (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault()
      open = true
    }
  }
  document.addEventListener("keydown", handleKeydown)
  return () => document.removeEventListener("keydown", handleKeydown)
})

// Load avatars for validator results
async function loadAvatars(validators: Validator[]) {
  await prefetchValidatorAvatars(validators)
  avatarsVersion++
}

function getAvatarUrl(identity?: string): string | null {
  return getCachedAvatar(identity)
}

// Clear results
function clearResults() {
  blockResult = null
  txResult = null
  validatorResults = []
  accountResult = null
}

// Debounced search
function handleInput() {
  if (searchTimer) clearTimeout(searchTimer)
  clearResults()

  const value = searchValue.trim()
  if (!value || value.length < 2) {
    isSearching = false
    return
  }

  isSearching = true
  searchTimer = setTimeout(() => doSearch(value), 300)
}

async function doSearch(value: string) {
  // Block height (all digits)
  if (/^\d+$/.test(value)) {
    try {
      const block = await runPromise(fetchBlockByHeight(value))
      blockResult = block
    } catch { /* not found */ }
  }

  // Transaction hash (64 hex chars)
  if (/^[A-Fa-f0-9]{64}$/.test(value)) {
    try {
      const result = await runPromise(fetchTransaction(value.toUpperCase()))
      txResult = result.tx_response
    } catch { /* not found */ }
  }

  // Validator address (starts with unionvaloper)
  if (value.startsWith("unionvaloper")) {
    try {
      const result = await runPromise(fetchValidator(value))
      validatorResults = [result.validator]
      loadAvatars(validatorResults)
    } catch { /* not found */ }
  }

  // Account address (starts with union but not unionvaloper)
  if (value.startsWith("union") && !value.startsWith("unionvaloper")) {
    try {
      const [accResult, balResult] = await Promise.all([
        runPromise(fetchAccount(value)),
        runPromise(fetchBalances(value)).catch(() => null)
      ])
      const mainBalance = balResult?.balances?.find(b => b.denom === "au")
      accountResult = {
        account: accResult.account,
        balance: mainBalance?.amount
      }
    } catch { /* not found */ }
  }

  // Search validators by moniker (if not an address)
  if (!value.startsWith("union") && !/^[A-Fa-f0-9]{64}$/.test(value) && !/^\d+$/.test(value)) {
    try {
      const result = await runPromise(fetchValidators())
      const matches = result.validators.filter(v =>
        v.description.moniker.toLowerCase().includes(value.toLowerCase())
      ).slice(0, 5)
      validatorResults = matches
      loadAvatars(validatorResults)
    } catch { /* ignore */ }
  }

  isSearching = false
}

// Quick navigation items (reactive to chain changes)
const quickNav = $derived([
  { title: "Blocks", href: `/${chainStore.id}/blocks`, icon: BoxIcon },
  { title: "Transactions", href: `/${chainStore.id}/transactions`, icon: ArrowLeftRightIcon },
  { title: "Validators", href: `/${chainStore.id}/validators`, icon: ShieldIcon },
  { title: "Accounts", href: `/${chainStore.id}/account`, icon: WalletIcon },
  { title: "Governance", href: `/${chainStore.id}/governance`, icon: VoteIcon },
])

// Check if we have any results
let hasResults = $derived(
  blockResult !== null ||
  txResult !== null ||
  validatorResults.length > 0 ||
  accountResult !== null
)
</script>

<!-- Trigger button for sidebar -->
<button
  onclick={() => open = true}
  class="w-full flex items-center gap-2 px-3 py-2 text-sm text-muted-foreground hover:text-foreground hover:bg-muted transition-colors group"
>
  <SearchIcon class="h-4 w-4" />
  <span class="flex-1 text-left group-data-[collapsible=icon]:hidden">Search...</span>
  <kbd class="hidden sm:inline-flex h-5 select-none items-center gap-1 rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground group-data-[collapsible=icon]:hidden">
    <span class="text-xs">⌘</span>K
  </kbd>
</button>

<!-- Command Dialog -->
<Dialog.Root bind:open onOpenChange={(isOpen) => { if (!isOpen) { searchValue = ""; clearResults() } }}>
  <Dialog.Content class="overflow-hidden p-0 sm:max-w-[550px] min-h-[300px]">
    <Command.Root class="[&_[data-slot=command-input-wrapper]]:border-b" shouldFilter={false}>
      <Command.Input
        placeholder="Search block, tx, validator, address..."
        bind:value={searchValue}
        oninput={handleInput}
      />
      <Command.List class="max-h-[400px]">
        {#if isSearching}
          <div class="py-6 flex items-center justify-center gap-2 text-sm text-muted-foreground">
            <LoaderIcon class="h-4 w-4 animate-spin" />
            Searching...
          </div>
        {:else if searchValue.trim().length >= 2 && !hasResults}
          <Command.Empty>
            <p class="py-6 text-center text-sm text-muted-foreground">
              No results found
            </p>
          </Command.Empty>
        {:else if !searchValue.trim()}
          <!-- Quick Navigation when empty -->
          <Command.Group heading="Quick Navigation">
            {#each quickNav as item}
              <Command.Item onSelect={() => { goto(item.href); open = false }} class="cursor-pointer">
                <item.icon class="mr-2 h-4 w-4" />
                <span>{item.title}</span>
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}

        <!-- Block Result -->
        {#if blockResult}
          <Command.Group heading="Block">
            <Command.Item
              onSelect={() => { goto(`/${chainStore.id}/blocks/${blockResult.block.header.height}`); open = false }}
              class="cursor-pointer"
            >
              <div class="flex items-center gap-3 w-full">
                <BoxIcon class="h-4 w-4 shrink-0 text-muted-foreground" />
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-mono font-medium">#{blockResult.block.header.height}</span>
                    <span class="text-xs text-muted-foreground">{formatTimeAgo(blockResult.block.header.time)}</span>
                  </div>
                  <div class="text-xs text-muted-foreground font-mono truncate">
                    {truncateHash(blockResult.block_id.hash, 16)}
                  </div>
                </div>
                <span class="text-xs text-muted-foreground shrink-0">
                  {blockResult.block.data.txs?.length ?? 0} txs
                </span>
              </div>
            </Command.Item>
          </Command.Group>
        {/if}

        <!-- Transaction Result -->
        {#if txResult}
          <Command.Group heading="Transaction">
            <Command.Item
              onSelect={() => { goto(`/${chainStore.id}/transactions/${txResult.txhash}`); open = false }}
              class="cursor-pointer"
            >
              <div class="flex items-center gap-3 w-full">
                <div class="w-5 h-5 flex items-center justify-center shrink-0 {txResult.code === 0 ? 'bg-success/20' : 'bg-destructive/20'}">
                  {#if txResult.code === 0}
                    <CheckIcon class="h-3 w-3 text-success" />
                  {:else}
                    <XIcon class="h-3 w-3 text-destructive" />
                  {/if}
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-sm">{truncateHash(txResult.txhash, 12)}</span>
                    <span class="text-xs text-muted-foreground">{formatTimeAgo(txResult.timestamp)}</span>
                  </div>
                  <div class="text-xs text-muted-foreground">
                    Block #{txResult.height} · {txResult.tx.body.messages.length} message(s)
                  </div>
                </div>
              </div>
            </Command.Item>
          </Command.Group>
        {/if}

        <!-- Validator Results -->
        {#if validatorResults.length > 0}
          <Command.Group heading="Validators">
            {#each validatorResults as validator}
              {@const avatarUrl = getAvatarUrl(validator.description.identity)}
              <Command.Item
                onSelect={() => { goto(`/${chainStore.id}/validators/${validator.operator_address}`); open = false }}
                class="cursor-pointer"
              >
                <div class="flex items-center gap-3 w-full">
                  {#if avatarUrl}
                    <img src={avatarUrl} alt="" class="w-6 h-6 rounded-full shrink-0 object-cover" />
                  {:else}
                    <div class="w-6 h-6 rounded-full shrink-0 bg-muted flex items-center justify-center text-xs font-medium">
                      {validator.description.moniker.charAt(0).toUpperCase()}
                    </div>
                  {/if}
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-medium truncate">{validator.description.moniker}</span>
                      {#if validator.jailed}
                        <span class="text-[10px] font-mono px-1 py-0.5 bg-red-500/10 text-red-500">Jailed</span>
                      {/if}
                    </div>
                    <div class="text-xs text-muted-foreground font-mono truncate">
                      {truncateAddress(validator.operator_address, 12)}
                    </div>
                  </div>
                  <span class="text-xs text-muted-foreground shrink-0">
                    {(Number(validator.commission.commission_rates.rate) * 100).toFixed(0)}% commission
                  </span>
                </div>
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}

        <!-- Account Result -->
        {#if accountResult}
          <Command.Group heading="Account">
            <Command.Item
              onSelect={() => { goto(`/${chainStore.id}/account/${accountResult.account.address}`); open = false }}
              class="cursor-pointer"
            >
              <div class="flex items-center gap-3 w-full">
                <WalletIcon class="h-4 w-4 shrink-0 text-muted-foreground" />
                <div class="flex-1 min-w-0">
                  <div class="font-mono text-sm truncate">{truncateAddress(accountResult.account.address, 16)}</div>
                  <div class="text-xs text-muted-foreground">
                    {accountResult.account["@type"]?.split(".").pop() ?? "Account"}
                  </div>
                </div>
                {#if accountResult.balance}
                  <span class="text-xs font-mono text-muted-foreground shrink-0">
                    {formatAmount(accountResult.balance, 18)} U
                  </span>
                {/if}
              </div>
            </Command.Item>
          </Command.Group>
        {/if}
      </Command.List>
    </Command.Root>
  </Dialog.Content>
</Dialog.Root>

<script lang="ts">
import { matchPromiseWithCache, cache } from "$lib/cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge } from "$lib/components/ui/badge/index.js"
import * as Collapsible from "$lib/components/ui/collapsible/index.js"
import { dataRow, sectionHeader } from "$lib/components/ui/snippets.svelte"
import { formatTime, truncateHash, truncateAddress } from "$lib/utils/format"
import { copyToClipboard as copyUtil } from "$lib/utils/clipboard"
import {
  buildValidatorMap as buildMap,
  prefetchValidatorAvatars,
  getCachedAvatar,
  getAvatarCacheVersion,
  getValidatorFromBase64Address,
  base64ToHex,
  formatAddress
} from "$lib/utils/validators.svelte"
import { urls } from "$lib/utils/urls"
import type { Block, TxResponse, PaginationResponse, Validator } from "$lib/types/cosmos"
import ChevronRight from "@lucide/svelte/icons/chevron-right"
import ChevronLeft from "@lucide/svelte/icons/chevron-left"
import ChevronDown from "@lucide/svelte/icons/chevron-down"
import Copy from "@lucide/svelte/icons/copy"
import Check from "@lucide/svelte/icons/check"
import UserIcon from "@lucide/svelte/icons/user"
import BoxIcon from "@lucide/svelte/icons/box"
import ClockIcon from "@lucide/svelte/icons/clock"
import FileTextIcon from "@lucide/svelte/icons/file-text"
import UsersIcon from "@lucide/svelte/icons/users"
import CornerMarks from "$lib/components/corner-marks.svelte"
import BlockVisual from "$lib/components/block-visual.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

let copied = $state<string | null>(null)

// Mapping from consensus hex address to validator info
let validatorMap = $state<Map<string, Validator>>(new Map())
let mapBuilt = $state(false)
const avatarsVersion = $derived(getAvatarCacheVersion())

// Build mapping from consensus address to validator
async function initValidatorMap(validators: Validator[]) {
  if (mapBuilt) return
  validatorMap = await buildMap(validators)
  mapBuilt = true
  await prefetchValidatorAvatars(validators)
}

// Watch for cached validators and build map
$effect(() => {
  const cached = cache.get("validators:bonded") as { validators: Validator[] } | undefined
  if (cached?.validators && !mapBuilt) {
    initValidatorMap(cached.validators)
  }
})

// Get validator info from signature's validator_address (base64 encoded)
function getValidatorForSignature(sigValidatorAddress: string): Validator | undefined {
  if (!sigValidatorAddress) return undefined
  const hex = base64ToHex(sigValidatorAddress)
  return validatorMap.get(hex)
}

function getAvatarUrl(identity?: string): string | null {
  return getCachedAvatar(identity)
}

// Get proposer validator from base64 proposer address
function getProposerInfo(proposerAddressBase64: string): { hex: string; validator: Validator | undefined; avatar: string | null } {
  return getValidatorFromBase64Address(proposerAddressBase64, validatorMap)
}

function copyToClipboard(text: string) {
  copyUtil(text, () => {
    copied = text
    setTimeout(() => copied = null, 2000)
  })
}
</script>

{#snippet loading()}
  <div class="space-y-4">
    <Skeleton class="h-12 w-64" />
    <Skeleton class="h-64" />
    <Skeleton class="h-48" />
  </div>
{/snippet}

{#snippet blockDetails(block: Block)}
  {@const header = block.block.header}
  {@const signatures = block.block.last_commit?.signatures ?? []}
  {@const signedCount = signatures.filter(s => s.signature !== null).length}
  {@const txCount = block.block.data.txs.length}

  <div class="space-y-6">
    <!-- Hero Header -->
    <div class="relative border border-border overflow-hidden">
      <CornerMarks />

      <div class="flex items-stretch">
        <!-- Block Visual -->
        <div class="relative shrink-0 border-r border-border bg-muted/20 p-6 flex items-center justify-center">
          <BlockVisual hash={block.block_id.hash} size={140} />
        </div>

        <!-- Block Info -->
        <div class="flex-1 p-6">
          <div class="flex items-start justify-between mb-4">
            <div>
              <div class="flex items-center gap-2 mb-2">
                <BoxIcon class="h-4 w-4 text-muted-foreground" />
                <span class="text-xs font-mono text-muted-foreground uppercase tracking-wider">Block</span>
              </div>
              <h1 class="text-4xl font-mono font-bold tracking-tight">#{header.height}</h1>
            </div>

            <!-- Navigation -->
            <div class="flex items-center gap-2">
              <a
                href={urls.block(Number(header.height) - 1)}
                class="p-2.5 border border-border hover:bg-muted transition-colors"
                title="Previous block"
              >
                <ChevronLeft class="h-5 w-5" />
              </a>
              <a
                href={urls.block(Number(header.height) + 1)}
                class="p-2.5 border border-border hover:bg-muted transition-colors"
                title="Next block"
              >
                <ChevronRight class="h-5 w-5" />
              </a>
            </div>
          </div>

          <!-- Quick Stats -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
            <div class="flex items-center gap-3">
              <div class="p-2 bg-muted">
                <ClockIcon class="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">Time</div>
                <div class="text-sm font-medium">{formatTime(header.time)}</div>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <div class="p-2 bg-muted">
                <FileTextIcon class="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">Transactions</div>
                <div class="text-sm font-medium">{txCount}</div>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <div class="p-2 bg-muted">
                <UsersIcon class="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">Signatures</div>
                <div class="text-sm font-medium">{signedCount} / {signatures.length}</div>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <div class="p-2 bg-muted">
                <BoxIcon class="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <div class="text-[10px] font-mono uppercase text-muted-foreground">Chain</div>
                <div class="text-sm font-mono">{header.chain_id}</div>
              </div>
            </div>
          </div>

          <!-- Hash -->
          <div class="mt-4 pt-4 border-t border-border">
            <div class="flex items-center gap-2 group">
              <span class="text-[10px] font-mono uppercase text-muted-foreground shrink-0">Hash</span>
              <span class="font-mono text-xs text-muted-foreground truncate flex-1">{block.block_id.hash}</span>
              <button
                onclick={() => copyToClipboard(block.block_id.hash)}
                class="p-1 hover:bg-muted transition-colors opacity-0 group-hover:opacity-100"
              >
                {#if copied === block.block_id.hash}
                  <Check class="h-3 w-3 text-green-500" />
                {:else}
                  <Copy class="h-3 w-3 text-muted-foreground" />
                {/if}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Proposer -->
    {#if true}
      {@const proposer = getProposerInfo(header.proposer_address)}
      <div class="relative border border-border">
        <CornerMarks />
        {@render sectionHeader("Proposer", "01")}
        {#if proposer.validator}
          <div class="flex items-center justify-between py-2.5 px-4 border-b border-border">
            <span class="text-xs text-muted-foreground">Validator</span>
            <a href={urls.validator(proposer.validator.operator_address)} class="flex items-center gap-2 hover:text-foreground/80">
              {#if proposer.avatar}
                <img src={proposer.avatar} alt="" class="w-5 h-5 rounded-full" />
              {:else}
                <div class="w-5 h-5 bg-muted flex items-center justify-center rounded-full">
                  <span class="text-[10px] font-bold">{proposer.validator.description.moniker.charAt(0)}</span>
                </div>
              {/if}
              <span class="text-sm">{proposer.validator.description.moniker}</span>
            </a>
          </div>
        {/if}
        {@render dataRow("Address", formatAddress(proposer.hex), { mono: true, copy: true })}
      </div>
    {/if}

    <!-- Block ID -->
    <div class="relative border border-border">
      <CornerMarks />

      {@render sectionHeader("Block ID", "02")}
      {@render dataRow("Hash", block.block_id.hash, { mono: true, copy: true })}
      {@render dataRow("Part Set Hash", block.block_id.part_set_header.hash, { mono: true, copy: true })}
      {@render dataRow("Part Set Total", block.block_id.part_set_header.total)}
    </div>

    <!-- Header Hashes -->
    <Collapsible.Root>
      <div class="relative border border-border">
        <CornerMarks />

        <Collapsible.Trigger class="w-full">
          <div class="flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors">
            <div class="flex items-center gap-3">
              <span class="text-xs font-mono text-muted-foreground">03</span>
              <span class="text-xs font-medium uppercase tracking-wider">Header Hashes</span>
            </div>
            <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform [[data-state=open]_&]:rotate-180" />
          </div>
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="border-t border-border">
            {@render dataRow("App Hash", header.app_hash, { mono: true, copy: true })}
            {@render dataRow("Data Hash", header.data_hash, { mono: true, copy: true })}
            {@render dataRow("Validators Hash", header.validators_hash, { mono: true, copy: true })}
            {@render dataRow("Next Validators", header.next_validators_hash, { mono: true, copy: true })}
            {@render dataRow("Consensus Hash", header.consensus_hash, { mono: true, copy: true })}
            {@render dataRow("Last Commit Hash", header.last_commit_hash, { mono: true, copy: true })}
            {@render dataRow("Last Results Hash", header.last_results_hash, { mono: true, copy: true })}
            {@render dataRow("Evidence Hash", header.evidence_hash, { mono: true, copy: true })}
          </div>
        </Collapsible.Content>
      </div>
    </Collapsible.Root>

    <!-- Previous Block -->
    <Collapsible.Root>
      <div class="relative border border-border">
        <CornerMarks />

        <Collapsible.Trigger class="w-full">
          <div class="flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors">
            <div class="flex items-center gap-3">
              <span class="text-xs font-mono text-muted-foreground">04</span>
              <span class="text-xs font-medium uppercase tracking-wider">Previous Block</span>
            </div>
            <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform [[data-state=open]_&]:rotate-180" />
          </div>
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="border-t border-border">
            {@render dataRow("Hash", header.last_block_id.hash, { mono: true, copy: true, link: urls.block(Number(header.height) - 1) })}
            {@render dataRow("Part Set Hash", header.last_block_id.part_set_header.hash, { mono: true, copy: true })}
            {@render dataRow("Part Set Total", header.last_block_id.part_set_header.total)}
          </div>
        </Collapsible.Content>
      </div>
    </Collapsible.Root>

    <!-- Version -->
    <Collapsible.Root>
      <div class="relative border border-border">
        <CornerMarks />

        <Collapsible.Trigger class="w-full">
          <div class="flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors">
            <div class="flex items-center gap-3">
              <span class="text-xs font-mono text-muted-foreground">05</span>
              <span class="text-xs font-medium uppercase tracking-wider">Version</span>
            </div>
            <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform [[data-state=open]_&]:rotate-180" />
          </div>
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="border-t border-border">
            {@render dataRow("Block", header.version.block)}
            {@render dataRow("App", header.version.app)}
          </div>
        </Collapsible.Content>
      </div>
    </Collapsible.Root>

    <!-- Last Commit -->
    {#if block.block.last_commit}
      <Collapsible.Root>
        <div class="relative border border-border">
          <CornerMarks />

          <Collapsible.Trigger class="w-full">
            <div class="flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors">
              <div class="flex items-center gap-3">
                <span class="text-xs font-mono text-muted-foreground">06</span>
                <span class="text-xs font-medium uppercase tracking-wider">Last Commit</span>
                <span class="text-xs text-muted-foreground">({signedCount} signatures)</span>
              </div>
              <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform [[data-state=open]_&]:rotate-180" />
            </div>
          </Collapsible.Trigger>
          <Collapsible.Content>
            <div class="border-t border-border">
              {@render dataRow("Height", block.block.last_commit.height)}
              {@render dataRow("Round", block.block.last_commit.round)}
              {@render dataRow("Block ID", block.block.last_commit.block_id.hash, { mono: true, copy: true })}

              {#if signatures.length > 0}
                <div class="border-t border-border">
                  <div class="px-4 py-2 bg-muted/20">
                    <span class="text-xs font-mono uppercase tracking-wider text-muted-foreground">Signatures</span>
                  </div>
                  <div class="max-h-[400px] overflow-y-auto">
                    {#each signatures as sig, i}
                      {@const validator = getValidatorForSignature(sig.validator_address)}
                      {@const avatarUrl = validator ? getAvatarUrl(validator.description.identity) : null}
                      <div class="flex items-center gap-3 px-4 py-2.5 text-xs border-b border-border last:border-b-0 hover:bg-muted/30 {sig.signature ? '' : 'opacity-40'}">
                        <span class="font-mono text-muted-foreground w-8 shrink-0">{String(i + 1).padStart(3, '0')}</span>
                        <!-- Validator Avatar -->
                        <div class="w-6 h-6 shrink-0 bg-muted flex items-center justify-center overflow-hidden">
                          {#if avatarUrl}
                            <img src={avatarUrl} alt="" class="w-full h-full object-cover" />
                          {:else}
                            <UserIcon class="h-3 w-3 text-muted-foreground" />
                          {/if}
                        </div>
                        <!-- Validator Name/Address -->
                        <div class="flex-1 min-w-0">
                          {#if validator}
                            <a href={urls.validator(validator.operator_address)} class="font-medium text-sm hover:underline truncate block">
                              {validator.description.moniker}
                            </a>
                          {:else}
                            <span class="font-mono text-muted-foreground truncate block">{truncateAddress(formatAddress(base64ToHex(sig.validator_address)), 8)}</span>
                          {/if}
                        </div>
                        <Badge variant={sig.signature ? "success" : "secondary"}>
                          {sig.signature ? "Signed" : "Absent"}
                        </Badge>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </Collapsible.Content>
        </div>
      </Collapsible.Root>
    {/if}
  </div>
{/snippet}

{#snippet txLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Transactions", "07")}
    <div class="p-4">
      <Skeleton class="h-32" />
    </div>
  </div>
{/snippet}

{#snippet txSuccess(result: { tx_responses: TxResponse[]; pagination: PaginationResponse })}
  {@render transactionsTable(result.tx_responses ?? [])}
{/snippet}

{#snippet txError(_err: unknown)}
  {@render transactionsTable([])}
{/snippet}

{#snippet transactionsTable(txs: TxResponse[])}
  <div class="relative border border-border">
    <CornerMarks />

    {@render sectionHeader("Transactions", "07")}

    {#if txs.length === 0}
      <div class="px-4 py-12 text-center">
        <p class="text-sm text-muted-foreground">No transactions in this block</p>
      </div>
    {:else}
      <!-- Header -->
      <div class="grid grid-cols-12 gap-4 px-4 py-2 border-b border-border bg-muted/20 text-xs font-mono uppercase tracking-wider text-muted-foreground">
        <div class="col-span-3">Hash</div>
        <div class="col-span-4">Messages</div>
        <div class="col-span-2">Status</div>
        <div class="col-span-3 text-right">Gas</div>
      </div>

      <!-- Rows -->
      {#each txs as tx}
        {@const messages = tx.tx.body.messages}
        <div class="grid grid-cols-12 gap-4 px-4 py-3 border-b border-border last:border-b-0 hover:bg-muted/30 items-center">
          <div class="col-span-3">
            <a href={urls.transaction(tx.txhash)} class="font-mono text-sm hover:underline">
              {truncateHash(tx.txhash, 8)}
            </a>
          </div>
          <div class="col-span-4 flex flex-wrap gap-1">
            {#each messages.slice(0, 2) as msg}
              <Badge variant="default">
                {msg["@type"]?.split(".").pop() ?? "Unknown"}
              </Badge>
            {/each}
            {#if messages.length > 2}
              <Badge variant="secondary">
                +{messages.length - 2}
              </Badge>
            {/if}
          </div>
          <div class="col-span-2">
            <Badge variant={tx.code === 0 ? "success" : "destructive"}>
              {tx.code === 0 ? "Success" : "Failed"}
            </Badge>
          </div>
          <div class="col-span-3 text-right font-mono text-xs text-muted-foreground">
            {tx.gas_used} / {tx.gas_wanted}
          </div>
        </div>
      {/each}
    {/if}
  </div>
{/snippet}

{#snippet error(err: unknown)}
  <div class="relative border border-destructive/50">
    <CornerMarks />
    <div class="p-6">
      <p class="text-sm font-medium text-destructive mb-2">Failed to load block</p>
      <p class="text-xs text-muted-foreground font-mono">{String(err)}</p>
      <a href={urls.blocks()} class="inline-block mt-4 text-xs font-mono uppercase tracking-wider hover:underline">
        Back to blocks
      </a>
    </div>
  </div>
{/snippet}

{#snippet validatorsLoading()}{/snippet}
{#snippet validatorsSuccess(_r: { validators: Validator[]; pagination: PaginationResponse })}{/snippet}
{#snippet validatorsError(_e: unknown)}{/snippet}

<!-- Pre-load validators for signature lookup -->
{@render matchPromiseWithCache(data.validators, {
  cacheKey: `${cachePrefix}validators:bonded`,
  onLoading: validatorsLoading,
  onSuccess: validatorsSuccess,
  onError: validatorsError,
})}

<div class="space-y-6">
  {@render matchPromiseWithCache(data.block, {
    cacheKey: `${cachePrefix}block:${data.height}`,
    onLoading: loading,
    onSuccess: blockDetails,
    onError: error,
  })}

  {@render matchPromiseWithCache(data.transactions, {
    cacheKey: `${cachePrefix}txs:height:${data.height}`,
    onLoading: txLoading,
    onSuccess: txSuccess,
    onError: txError,
  })}
</div>

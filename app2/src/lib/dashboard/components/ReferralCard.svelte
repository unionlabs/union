<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import type { AppClaimedCode, AppReferralCode } from "$lib/dashboard/stores/airdrop.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"

let generateAmount = $state(0)
let claimCode = $state("")

// Button feedback states
let copyingCodeId = $state<string | null>(null)
let copyingLinkId = $state<string | null>(null)
let deletingCodeId = $state<string | null>(null)

const availableToShare = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => 0,
    onSome: (store) => store.availableToSharePercentage,
  }),
)

const totalAllocation = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => 0,
    onSome: (store) => store.totalAllocationTokens,
  }),
)

const referralCodes = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => [] as AppReferralCode[],
    onSome: (store) => Option.getOrElse(store.appReferralCodes, () => []),
  }),
)

const claimedCodes = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => [] as AppClaimedCode[],
    onSome: (store) => Option.getOrElse(store.appClaimedCodes, () => []),
  }),
)

const isGenerating = $derived(
  Option.map(dashboard.airdrop, (store) => store.isGeneratingCode).pipe(
    Option.getOrElse(() => false),
  ),
)

const isClaiming = $derived(
  Option.map(dashboard.airdrop, (store) => store.isClaimingCode).pipe(
    Option.getOrElse(() => false),
  ),
)

// Calculate actual token amounts for context
let availableTokens = $derived((availableToShare / 100) * totalAllocation)

// Validate and clamp the generate amount
function validateGenerateAmount(value: number) {
  if (value < 0.1) {
    return 0.1 // Minimum 0.1% to prevent spam codes
  }
  if (value > availableToShare) {
    return availableToShare
  }
  return Math.round(value * 10) / 10
}

// Handle input blur (when user finishes typing)
function handleGenerateAmountBlur(event: Event) {
  const target = event.target as HTMLInputElement
  const value = parseFloat(target.value) || 0
  generateAmount = validateGenerateAmount(value)
}

async function handleGenerateCode() {
  const validAmount = validateGenerateAmount(generateAmount)
  if (validAmount >= 0.1 && validAmount <= availableToShare) {
    const success = await Option.match(dashboard.airdrop, {
      onNone: () => Promise.resolve(false),
      onSome: (store) => store.generateCode(validAmount),
    })
    if (success) {
      generateAmount = 0 // Reset on success
    }
  }
}

async function handleClaimCode() {
  if (claimCode.trim()) {
    const success = await Option.match(dashboard.airdrop, {
      onNone: () => Promise.resolve(false),
      onSome: (store) => store.claimCode(claimCode.trim()),
    })
    if (success) {
      claimCode = "" // Reset on success
    }
  }
}

// Removed error clearing functions - using global error system

async function handleRemoveCode(codeId: string) {
  deletingCodeId = codeId
  try {
    await Option.match(dashboard.airdrop, {
      onNone: () => Promise.resolve(false),
      onSome: (store) => store.removeCode(codeId),
    })
  } finally {
    deletingCodeId = null
  }
}

function getStatusBadge(status: "pending" | "claimed") {
  switch (status) {
    case "pending":
      return {
        text: "PENDING",
        color: "text-yellow-400",
        bgColor: "bg-zinc-800/80",
        borderColor: "border-yellow-400/50",
      }
    case "claimed":
      return {
        text: "CLAIMED",
        color: "text-accent",
        bgColor: "bg-zinc-800/80",
        borderColor: "border-accent/50",
      }
  }
}

async function copyToClipboard(text: string, codeId: string) {
  copyingCodeId = codeId
  try {
    await navigator.clipboard.writeText(text)
    setTimeout(() => {
      copyingCodeId = null
    }, 1000)
  } catch (error) {
    copyingCodeId = null
  }
}

async function copyReferralLink(code: string, codeId: string) {
  copyingLinkId = codeId
  const baseUrl = "https://app.union.build/udrop"
  const fullUrl = `${baseUrl}?code=${code}`
  try {
    await navigator.clipboard.writeText(fullUrl)
    setTimeout(() => {
      copyingLinkId = null
    }, 1000)
  } catch (error) {
    copyingLinkId = null
  }
}
</script>

<Card>
  <div class="flex flex-col gap-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg font-semibold text-white">Pre-stake Referral System</h3>
        <p class="text-sm text-zinc-400">Generate codes or claim pre-stake allowance from others</p>
      </div>
      <div class="text-right">
        <div class="text-sm text-zinc-400">Available to Share</div>
        <div class="text-lg font-semibold text-white">
          {availableToShare.toFixed(1)}%
        </div>
      </div>
    </div>

    <!-- Claim & Generate Actions -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- Claim Code Section -->
      <Card class="bg-zinc-900/50 border-zinc-800">
        <div class="flex flex-col gap-4">
          <div>
            <h4 class="text-sm font-medium text-zinc-300">Claim Code</h4>
            <p class="text-xs text-zinc-500">
              Use codes from others to increase your pre-stake allowance
            </p>
          </div>

          <div class="flex flex-col gap-3">
            <div>
              <label
                for="claim-code"
                class="text-xs text-zinc-400 block mb-2"
              >
                Referral Code
              </label>
              <input
                id="claim-code"
                type="text"
                bind:value={claimCode}
                class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md text-white text-sm focus:outline-none focus:border-accent"
                placeholder="UNION-ABC123"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
              />
            </div>

            <div class="flex justify-start gap-3">
              <Button
                variant="primary"
                onclick={handleClaimCode}
                disabled={isClaiming || !claimCode.trim()}
              >
                {#if isClaiming}
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="animate-spin mr-2"
                  >
                    <path d="M21 12a9 9 0 11-6.219-8.56" />
                  </svg>
                  Claiming...
                {:else}
                  Claim Code
                {/if}
              </Button>

              {#if claimCode.trim()}
                <Button
                  variant="outline"
                  onclick={() => claimCode = ""}
                  disabled={isClaiming}
                >
                  Reset
                </Button>
              {/if}
            </div>
          </div>
        </div>
      </Card>

      <!-- Generate Code Section -->
      <Card class="bg-zinc-900/50 border-zinc-800">
        <div class="flex flex-col gap-4">
          <div>
            <h4 class="text-sm font-medium text-zinc-300">Generate Code</h4>
            <p class="text-xs text-zinc-500">Share a percentage of your pre-stake allowance</p>
          </div>

          {#if availableToShare > 0}
            <div class="flex flex-col gap-3">
              <div>
                <label
                  for="generate-amount"
                  class="text-xs text-zinc-400 block mb-2"
                >
                  Percentage to Share
                </label>
                <div class="relative">
                  <input
                    id="generate-amount"
                    type="number"
                    min="0.1"
                    max={availableToShare}
                    step="0.1"
                    bind:value={generateAmount}
                    onblur={handleGenerateAmountBlur}
                    class="w-full px-3 py-2 pr-8 bg-zinc-800 border border-zinc-700 rounded-md text-white text-sm focus:outline-none focus:border-accent"
                    placeholder="0.0"
                    autocomplete="off"
                    autocorrect="off"
                    autocapitalize="off"
                    spellcheck="false"
                  />
                  <span class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 text-sm"
                  >%</span>
                </div>
              </div>

              <div class="flex justify-start gap-3">
                <Button
                  variant="primary"
                  onclick={handleGenerateCode}
                  disabled={isGenerating || generateAmount < 0.1
                  || generateAmount > availableToShare}
                >
                  {#if isGenerating}
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      class="animate-spin mr-2"
                    >
                      <path d="M21 12a9 9 0 11-6.219-8.56" />
                    </svg>
                    Generating...
                  {:else}
                    Generate Code
                  {/if}
                </Button>

                {#if generateAmount > 0}
                  <Button
                    variant="outline"
                    onclick={() => generateAmount = 0}
                    disabled={isGenerating}
                  >
                    Reset
                  </Button>
                {/if}
              </div>
            </div>
          {:else}
            <div class="h-full p-4 bg-amber-500/10 border border-amber-500/30 rounded-lg flex items-center">
              <div class="flex items-start gap-3">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="text-amber-400 flex-shrink-0 mt-0.5"
                >
                  <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
                  <path d="M12 9v4" />
                  <path d="m12 17 .01 0" />
                </svg>
                <div>
                  <p class="text-sm font-medium text-amber-300 mb-1">
                    No pre-stake allowance available to share
                  </p>
                  <p class="text-xs text-amber-200">
                    Claim referral codes from others to increase your allowance.
                  </p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </Card>
    </div>

    <!-- Your Generated Codes -->
    {#if referralCodes.length > 0}
      <div class="flex flex-col gap-3">
        <h4 class="text-sm font-medium text-zinc-300">Your Codes</h4>
        <div class="flex flex-col gap-2">
          {#each referralCodes as code}
            {@const status = getStatusBadge(code.status)}

            <div class="flex items-center justify-between py-2 px-3 bg-zinc-900/30 border border-zinc-800 rounded-lg hover:border-accent/30 transition-all duration-200">
              <!-- Code Info -->
              <div class="flex items-center gap-3">
                <div class="flex flex-col gap-1">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-sm text-white">{code.code}</span>
                    <button
                      onclick={() => copyToClipboard(code.code, code.id)}
                      class="w-5 h-5 flex items-center justify-center rounded hover:bg-zinc-800 transition-colors"
                      aria-label="Copy code"
                      title="Copy code"
                      disabled={copyingCodeId === code.id}
                    >
                      {#if copyingCodeId === code.id}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          class="text-green-400"
                        >
                          <path d="M20 6 9 17l-5-5" />
                        </svg>
                      {:else}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          class="text-zinc-500"
                        >
                          <rect
                            width="14"
                            height="14"
                            x="8"
                            y="8"
                            rx="2"
                            ry="2"
                          />
                          <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
                        </svg>
                      {/if}
                    </button>
                    <button
                      onclick={() => copyReferralLink(code.code, code.id)}
                      class="w-5 h-5 flex items-center justify-center rounded hover:bg-zinc-800 transition-colors"
                      aria-label="Copy referral link"
                      title="Copy referral link"
                      disabled={copyingLinkId === code.id}
                    >
                      {#if copyingLinkId === code.id}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          class="text-green-400"
                        >
                          <path d="M20 6 9 17l-5-5" />
                        </svg>
                      {:else}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          class="text-zinc-500"
                        >
                          <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
                          <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
                        </svg>
                      {/if}
                    </button>
                    {#if code.status === "pending"}
                      <button
                        onclick={() => handleRemoveCode(code.id)}
                        class="w-5 h-5 flex items-center justify-center rounded hover:bg-red-500/10 hover:border-red-500/20 border border-transparent transition-all duration-200 group"
                        aria-label="Remove code"
                        title="Remove code"
                        disabled={deletingCodeId === code.id}
                      >
                        {#if deletingCodeId === code.id}
                          <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="12"
                            height="12"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="text-red-400 animate-spin"
                          >
                            <circle
                              cx="12"
                              cy="12"
                              r="10"
                              opacity="0.25"
                            />
                            <path d="M4 12a8 8 0 0 1 8-8V2.5" />
                          </svg>
                        {:else}
                          <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="12"
                            height="12"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="text-zinc-500 group-hover:text-red-400"
                          >
                            <path d="M3 6h18" />
                            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                          </svg>
                        {/if}
                      </button>
                    {/if}
                  </div>
                  <div class="text-xs text-zinc-500">
                    {code.createdAt.toLocaleDateString()}
                    {#if code.claimedAt}
                      • {code.claimedAt.toLocaleDateString()}
                    {/if}
                    {#if code.claimedBy}
                      • {code.claimedBy}
                    {/if}
                  </div>
                </div>
              </div>

              <!-- Percentage & Actions -->
              <div class="flex items-center gap-3">
                <div class="text-sm font-medium text-white">
                  {code.percentage.toFixed(2)}%
                </div>

                <div class="px-2 py-1 rounded {status.bgColor} {status.borderColor} border flex items-center justify-center relative">
                  <span class="text-xs font-medium {status.color}">
                    {status.text}
                  </span>
                  {#if code.status === "claimed"}
                    <div class="absolute inset-0 rounded bg-accent/20 blur-sm animate-pulse"></div>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Your Claimed Codes -->
    {#if claimedCodes.length > 0}
      <div class="flex flex-col gap-3">
        <h4 class="text-sm font-medium text-zinc-300">Your Claims</h4>
        <div class="flex flex-col gap-2">
          {#each claimedCodes as claim}
            <div class="flex items-center justify-between py-2 px-3 bg-zinc-900/30 border border-zinc-800 rounded-lg">
              <!-- Code Info -->
              <div class="flex items-center gap-3">
                <div class="flex flex-col gap-1">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-sm text-white">{claim.code}</span>
                    <button
                      onclick={() => copyToClipboard(claim.code, claim.id)}
                      class="w-5 h-5 flex items-center justify-center rounded hover:bg-zinc-800 transition-colors"
                      aria-label="Copy code"
                      title="Copy code"
                      disabled={copyingCodeId === claim.id}
                    >
                      {#if copyingCodeId === claim.id}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          class="text-green-400"
                        >
                          <path d="M20 6 9 17l-5-5" />
                        </svg>
                      {:else}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          class="text-zinc-500"
                        >
                          <rect
                            width="14"
                            height="14"
                            x="8"
                            y="8"
                            rx="2"
                            ry="2"
                          />
                          <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
                        </svg>
                      {/if}
                    </button>
                  </div>
                  <div class="text-xs text-zinc-500">
                    {claim.claimedAt.toLocaleDateString()} • From {claim.originalOwner}
                  </div>
                </div>
              </div>

              <!-- Percentage & Badge -->
              <div class="flex items-center gap-3">
                <div class="text-sm font-medium text-white">
                  +{claim.percentage.toFixed(2)}%
                </div>

                <div class="px-2 py-1 rounded bg-zinc-800/80 border-accent/50 border flex items-center justify-center relative">
                  <span class="text-xs font-medium text-accent">
                    CLAIMED
                  </span>
                  <div class="absolute inset-0 rounded bg-accent/20 blur-sm animate-pulse"></div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</Card>

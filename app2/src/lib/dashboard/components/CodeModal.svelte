<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Modal from "$lib/components/ui/Modal.svelte"
import { verifyReferralCode } from "$lib/dashboard/queries/private"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { Option } from "effect"

interface Props {
  isOpen: boolean
  onClose: () => void
  code: string
}

const { isOpen, onClose, code }: Props = $props()

let isLoading = $state(true)
let error = $state<Option.Option<string>>(Option.none())
let codeDetails = $state<
  Option.Option<{
    code: string
    percentage: number
    owner_name: string
    owner_avatar: string | null
    owner_id: string
    is_claimed: boolean
  }>
>(Option.none())
let isClaiming = $state(false)

// Get user info for eligibility
let isEligible = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) =>
      Option.match(store.allocation, {
        onNone: () => false,
        onSome: (allocation) =>
          allocation.is_eligible && !!allocation.evm_wallet && allocation.is_human,
      }),
  }),
)

async function loadCodeDetails() {
  if (!code) {
    return
  }

  isLoading = true
  error = Option.none()

  const result = await runPromise(verifyReferralCode(code.toUpperCase()))

  if (Option.isSome(result)) {
    const details = result.value as any
    const currentUserId = Option.getOrElse(dashboard.userId, () => "")

    // Check if this is user's own code
    if (currentUserId && details.owner_id === currentUserId) {
      error = Option.some("You can't claim your own code")
      isLoading = false
      return
    }

    codeDetails = Option.some(details)
  } else {
    error = Option.some("Code not found")
  }

  isLoading = false
}

async function claimCode() {
  if (Option.isNone(codeDetails) || isClaiming) {
    return
  }

  isClaiming = true

  const success = await Option.match(dashboard.airdrop, {
    onNone: () => Promise.resolve(false),
    onSome: (store) => store.claimCode(code.toUpperCase()),
  })

  if (success) {
    onClose()
  } else {
    isClaiming = false
  }
}

// Load when modal opens
$effect(() => {
  if (isOpen && code) {
    loadCodeDetails()
  }
})

// Reset when modal closes
$effect(() => {
  if (!isOpen) {
    isLoading = true
    error = Option.none()
    codeDetails = Option.none()
    isClaiming = false
  }
})
</script>

<Modal
  {isOpen}
  {onClose}
  class="w-full max-w-sm"
>
  {#if isLoading}
    <div class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4">
        </div>
        <p class="text-zinc-400">Loading code...</p>
      </div>
    </div>
  {:else if Option.isSome(error)}
    <div class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="w-12 h-12 mx-auto mb-4 rounded-full bg-red-500/20 flex items-center justify-center">
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            class="text-red-400"
          >
            <circle
              cx="12"
              cy="12"
              r="10"
            />
            <path d="m15 9-6 6" />
            <path d="m9 9 6 6" />
          </svg>
        </div>
        <h3 class="text-lg font-semibold text-white mb-2">Invalid Code</h3>
        <p class="text-sm text-zinc-400">{error.value}</p>
      </div>
    </div>
  {:else if Option.isSome(codeDetails)}
    {#if codeDetails.value.is_claimed}
      <div class="flex items-center justify-center py-12">
        <div class="text-center">
          <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-green-500/20 flex items-center justify-center">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              class="text-green-400"
            >
              <path d="M9 12l2 2 4-4" />
              <circle
                cx="12"
                cy="12"
                r="10"
              />
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-white mb-2">Already Claimed</h3>
          <p class="text-sm text-zinc-400">This referral code has been used</p>
        </div>
      </div>
    {:else}
      <div class="flex flex-col h-full min-h-[350px]">
        <!-- Title at top -->
        <h2 class="text-xl font-bold text-white mb-6">Referral Code</h2>

        <!-- Content centered in remaining space -->
        <div class="flex-grow flex items-center justify-center">
          <div class="text-center">
            <!-- Owner -->
            <div class="flex items-center justify-center gap-3 mb-6">
              {#if codeDetails.value.owner_avatar}
                <img
                  src={codeDetails.value.owner_avatar}
                  alt=""
                  class="w-12 h-12 rounded-full border border-zinc-700"
                />
              {:else}
                <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center">
                  <svg
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    class="text-zinc-400"
                  >
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                    <circle
                      cx="12"
                      cy="7"
                      r="4"
                    />
                  </svg>
                </div>
              {/if}
              <div class="text-left">
                <div class="font-semibold text-white text-base">{codeDetails.value.owner_name}</div>
                <div class="text-sm text-zinc-400">shared this code</div>
              </div>
            </div>

            <!-- Bonus -->
            <div class="inline-flex items-center gap-2 bg-accent/10 border border-accent/30 px-6 py-3 rounded-lg">
              <span class="text-xl font-bold text-accent"
              >+{Math.round(codeDetails.value.percentage * 100) / 100}%</span>
              <span class="text-sm text-zinc-300">bonus allowance</span>
            </div>
          </div>
        </div>

        <!-- Actions at bottom -->
        <div class="flex-shrink-0">
          {#if !isEligible}
            <div class="space-y-3">
              <div class="text-center py-3 bg-orange-500/10 border border-orange-500/30 rounded-lg">
                <div class="text-orange-400 font-medium text-sm">Complete verification first</div>
              </div>
              <Button
                variant="outline"
                onclick={() =>
                navigator.clipboard.writeText(
                  Option.getOrElse(codeDetails, () => ({ code: "" })).code,
                )}
                class="w-full"
              >
                Copy Code
              </Button>
            </div>
          {:else}
            <div class="space-y-3">
              <Button
                onclick={claimCode}
                variant="primary"
                class="w-full"
                disabled={isClaiming}
              >
                {#if isClaiming}
                  <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin mr-2">
                  </div>
                  Claiming...
                {:else}
                  Claim Code
                {/if}
              </Button>
              <Button
                variant="outline"
                onclick={() =>
                navigator.clipboard.writeText(
                  Option.getOrElse(codeDetails, () => ({ code: "" })).code,
                )}
                class="w-full"
                disabled={isClaiming}
              >
                Copy Code
              </Button>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</Modal>

<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { formatUnits } from "viem"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

let claim = $derived(
  Option.flatMap(dashboard.airdrop, (store) => store.claim),
)

$effect(() => {
  if (Option.isNone(claim)) {
    window.location.href = "/udrop/claim?step=1"
  }
})

let isLoadingClaim = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => true,
    onSome: (store) => store.isLoadingClaim,
  }),
)

function handleProceedToClaim() {
  onNext()
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            Your Claim Details
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Review your eU allocation and claim proof before proceeding.
          </p>
        </div>
      </div>

      <div class="space-y-4">
        {#if isLoadingClaim}
          <div class="bg-zinc-950/50 rounded-lg p-6 border border-zinc-800">
            <div class="flex items-center justify-center">
              <div class="w-6 h-6 border-2 border-accent border-t-transparent rounded-full animate-spin">
              </div>
              <span class="ml-3 text-zinc-400">Loading claim data...</span>
            </div>
          </div>
        {:else if Option.isSome(claim)}
          <!-- Claim Status Info -->
          <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
            <div class="flex items-center gap-3">
              <div class="size-8 rounded-lg bg-accent/20 border border-accent/40 flex items-center justify-center flex-shrink-0">
                <svg
                  class="w-4 h-4 text-accent"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-white">Claim Proof Ready</div>
                <div class="text-xs text-accent mt-1">
                  Your proof has been pre-generated and verified
                </div>
              </div>
            </div>
          </div>

          <div class="flex gap-3">
            <Button
              variant="primary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={handleProceedToClaim}
            >
              Proceed to Claim
            </Button>
          </div>
        {:else}
          <!-- No claim found -->
          <div class="bg-red-500/10 border border-red-500/20 rounded-lg p-6">
            <div class="text-center">
              <div class="w-12 h-12 bg-red-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg
                  class="w-6 h-6 text-red-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
                  />
                </svg>
              </div>
              <div class="text-lg font-medium text-red-400 mb-2">No Claim Found</div>
              <div class="text-sm text-zinc-400">
                You don't have any eU available to claim at this time.
              </div>
            </div>
          </div>
        {/if}

        <!-- Back button -->
        {#if onBack}
          <Button
            variant="secondary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onBack}
          >
            ← Back
          </Button>
        {/if}
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">Your Claim Details</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Review your allocation before claiming.
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col">
        {#if isLoadingClaim}
          <!-- Loading state -->
          <div
            class="w-full h-full flex items-center justify-center"
            style="background-color: #0D2024;"
          >
            <div class="text-center">
              <div class="w-16 h-16 border-4 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4">
              </div>
              <div class="text-zinc-400">Loading claim data...</div>
            </div>
          </div>
        {:else if Option.isSome(claim)}
          <!-- Claim Details -->
          <div class="flex flex-col h-full">
            <!-- Scrollable Proof Details -->
            <div class="flex-1 overflow-y-auto p-4">
              <div class="space-y-4">
                <div>
                  <div class="text-xs text-zinc-500 mb-2 uppercase tracking-wider">Beneficiary</div>
                  <div class="text-sm font-mono text-zinc-300 bg-zinc-900 p-3 rounded border border-zinc-800 break-all">
                    {claim.value.beneficiary}
                  </div>
                </div>

                <div>
                  <div class="text-xs text-zinc-500 mb-2 uppercase tracking-wider">Amount</div>
                  <div class="text-sm font-mono text-zinc-300 bg-zinc-900 p-3 rounded border border-zinc-800">
                    {
                      claim.value.amount
                      ? formatUnits(BigInt(claim.value.amount), 18)
                      : "0"
                    } eU
                  </div>
                </div>

                <div>
                  <div class="text-xs text-zinc-500 mb-2 uppercase tracking-wider">
                    Merkle Proof ({
                      Array.isArray(claim.value.proof)
                      ? claim.value.proof.length
                      : 0
                    } hashes)
                  </div>
                  <div class="bg-zinc-900 rounded border border-zinc-800">
                    {#each Array.isArray(claim.value.proof) ? claim.value.proof : [] as
                      proofHash,
                      i
                    }
                      <div class="p-3 border-b border-zinc-800 last:border-b-0">
                        <div class="text-xs text-zinc-500 mb-1">Hash {i + 1}</div>
                        <div class="text-sm font-mono text-zinc-300 break-all">{proofHash}</div>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
            </div>
          </div>
        {:else}
          <!-- No claim state -->
          <div
            class="w-full h-full flex items-center justify-center"
            style="background-color: #0D2024;"
          >
            <div class="text-center">
              <div class="w-16 h-16 bg-red-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg
                  class="w-8 h-8 text-red-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01"
                  />
                </svg>
              </div>
              <div class="text-zinc-400">No claim found</div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/snippet}
</StepLayout>

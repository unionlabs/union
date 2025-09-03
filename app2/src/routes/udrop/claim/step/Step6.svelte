<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { formatUnits } from "viem"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onRestart: () => void
}

let { onRestart }: Props = $props()

let claim = $derived(Option.flatMap(dashboard.airdrop, (store) => store.claim))

let claimAmount = $derived(
  Option.match(claim, {
    onNone: () => "0",
    onSome: (claimData) => formatUnits(BigInt(claimData.amount), 18),
  }),
)

// Get transaction hash from localStorage
let transactionHash = $derived<Option.Option<string>>(
  typeof window !== "undefined"
    ? Option.fromNullable(localStorage.getItem("lastClaimTxHash"))
    : Option.none(),
)

function handleViewTransaction() {
  Option.match(transactionHash, {
    onNone: () => {},
    onSome: (hash) => window.open(`https://etherscan.io/tx/${hash}`, "_blank"),
  })
}

function handleDone() {
  onRestart()
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            U Claimed
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Your U has been successfully claimed. Thank you for being part of the Union.
          </p>
        </div>
      </div>

      <div class="space-y-4">
        <!-- Transaction Info -->
        {#if Option.isSome(transactionHash)}
          <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm font-medium text-white mb-1">Transaction Hash</div>
                <div class="text-xs font-mono text-zinc-400">
                  {transactionHash.value.slice(0, 10)}...{transactionHash.value.slice(-8)}
                </div>
              </div>
              <Button
                variant="secondary"
                class="text-xs px-3 py-1.5"
                onclick={handleViewTransaction}
              >
                View
              </Button>
            </div>
          </div>
        {/if}

        <!-- Action Buttons -->
        <div class="flex gap-3">
          <Button
            variant="primary"
            class="flex flex-1 items-center justify-center gap-3"
            onclick={handleDone}
          >
            Done
          </Button>
        </div>
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">Thank U!</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Your tokens have been successfully claimed.
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Thank you celebration -->
        <div
          class="text-center w-full h-full flex items-center justify-center"
          style="background-color: #0D2024;"
        >
          <div>
            <div class="relative mb-6">
            </div>
            <div class="text-4xl font-bold text-accent mb-2">Thank U!</div>
          </div>
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>

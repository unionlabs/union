<script lang="ts">
import { goto } from "$app/navigation"
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import AirdropProfileCard from "$lib/dashboard/components/AirdropProfileCard.svelte"
import PreStakeCard from "$lib/dashboard/components/PreStakeCard.svelte"
import ReferralCard from "$lib/dashboard/components/ReferralCard.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"

let isLoadingAllocation = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => true,
    onSome: (store) => store.isLoadingAllocation,
  }),
)

let allocation = $derived(
  Option.flatMap(dashboard.airdrop, (store) => store.calculatedAllocation),
)

let isEligible = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.isEligible,
  }),
)

let hasEvm = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.hasEvmWallet,
  }),
)

let isHuman = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.isHuman,
  }),
)

$effect(() => {
  if (!isLoadingAllocation && (!isEligible || !hasEvm || !isHuman)) {
    goto("/udrop/check")
  }
})
</script>

<div class="flex flex-col gap-6">
  <!-- Profile and Allocation Row -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Profile Card with Verification Badge -->
    <AirdropProfileCard
      isVerified={isHuman}
      showVerificationBadge={true}
    />

    <!-- U Drop Allocation -->
    <Card class="flex flex-col flex-1 relative overflow-hidden">
      <div class="flex flex-col gap-2 h-full">
        <!-- Centered Content -->
        <div class="flex-1 flex flex-col items-center justify-center text-center gap-6">
          {#if isLoadingAllocation}
            <div class="text-2xl text-zinc-400">Loading...</div>
          {:else if isEligible && Option.isSome(allocation)}
            <!-- Token Allocation -->
            <div class="flex flex-col items-center gap-2">
              <div class="text-sm text-zinc-400 font-medium uppercase tracking-wider">
                TOKEN ALLOCATION
              </div>
              <div class="text-[clamp(2rem,6vw,3rem)] font-bold text-white leading-none">
                {
                  Option.match(allocation, {
                    onNone: () => 0,
                    onSome: (alloc) => alloc.total_tokens,
                  }).toLocaleString()
                } U
              </div>
            </div>

            <!-- Staking Allowance -->
            <div class="flex flex-col items-center gap-2">
              <div class="text-sm text-zinc-400 font-medium uppercase tracking-wider">
                STAKING ALLOWANCE
              </div>
              <div class="text-[clamp(2rem,6vw,3rem)] font-bold text-accent leading-none">
                {
                  Option.match(allocation, {
                    onNone: () => "0",
                    onSome: (alloc) => {
                      const percentage = (alloc.allocation_percentage || 0) * 100
                      return `${percentage.toFixed(0)}%`
                    },
                  })
                }
              </div>
            </div>
          {:else}
            <div class="text-2xl text-zinc-400">No allocation found</div>
            <div class="text-sm text-zinc-500">Complete the verification process</div>
          {/if}
        </div>
      </div>
    </Card>
  </div>

  {#if isEligible && Option.isSome(allocation)}
    <PreStakeCard />
    <ReferralCard />
  {:else if !isLoadingAllocation}
    <Card>
      <div class="text-center py-8">
        <div class="text-zinc-400">Complete verification to access your allocation</div>
        <div class="text-sm text-zinc-500 mt-2">
          <Button
            variant="secondary"
            onclick={() => goto("/udrop/check")}
          >
            Continue Verification
          </Button>
        </div>
      </div>
    </Card>
  {/if}
</div>

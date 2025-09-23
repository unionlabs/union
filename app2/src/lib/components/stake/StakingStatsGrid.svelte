<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StatCard from "./StatCard.svelte"
import SharpUpcomingIcon from "$lib/components/icons/SharpUpcomingIcon.svelte"
import SharpDashboardIcon from "$lib/components/icons/SharpDashboardIcon.svelte"
import SharpStakeIcon from "$lib/components/icons/SharpStakeIcon.svelte"
import RotateLeftIcon from "$lib/components/icons/RotateLeftIcon.svelte"
import RestoreIcon from "$lib/components/icons/RestoreIcon.svelte"
import { Utils } from "@unionlabs/sdk"
import { BigDecimal, Exit, pipe } from "effect"
import * as O from "effect/Option"
import type { IncentiveResult, IncentiveError } from "$lib/services/incentive"

interface Props {
  incentives: O.Option<Exit.Exit<IncentiveResult, IncentiveError>>
  exchangeRate: string
  showInverseRate: boolean
  onToggleRate: () => void
}

let { 
  incentives, 
  exchangeRate, 
  showInverseRate,
  onToggleRate
}: Props = $props()
</script>

<Card class="p-4 bg-zinc-925">
  <div class="flex flex-col gap-3">
    
    <div class="grid grid-cols-2 gap-2.5">
      <!-- Rewards Card -->
      <StatCard
        label="Rewards"
        value={O.isSome(incentives) && Exit.isSuccess(incentives.value)
          ? `${pipe(
              incentives.value.value.rates.yearly,
              BigDecimal.multiply(BigDecimal.fromBigInt(100n)),
              BigDecimal.round({ mode: "from-zero", scale: 2 }),
              Utils.formatBigDecimal,
            )}%`
          : undefined}
        loading={O.isNone(incentives) || !Exit.isSuccess(incentives.value)}
        class="text-accent"
      >
        {#snippet icon()}
          <SharpUpcomingIcon class="w-3 h-3 text-accent" />
        {/snippet}
      </StatCard>

      <!-- Total Supply Card -->
      <StatCard
        label="Supply"
        value={O.isSome(incentives) && Exit.isSuccess(incentives.value)
          ? Number(BigDecimal.unsafeToNumber(incentives.value.value.totalSupply)).toLocaleString('en-US', { maximumFractionDigits: 0 })
          : undefined}
        loading={O.isNone(incentives) || !Exit.isSuccess(incentives.value)}
      >
        {#snippet icon()}
          <SharpDashboardIcon class="w-3 h-3 text-zinc-500" />
        {/snippet}
      </StatCard>

      <!-- Staked Card -->
      <StatCard
        label="Staked"
        value={O.isSome(incentives) && Exit.isSuccess(incentives.value)
          ? Number(BigDecimal.unsafeToNumber(incentives.value.value.bondedTokens)).toLocaleString('en-US', { maximumFractionDigits: 0 })
          : undefined}
        subtitle={O.isSome(incentives) && Exit.isSuccess(incentives.value)
          ? `${(BigDecimal.unsafeToNumber(incentives.value.value.bondedRatio) * 100).toFixed(1)}%`
          : undefined}
        loading={O.isNone(incentives) || !Exit.isSuccess(incentives.value)}
      >
        {#snippet icon()}
          <SharpStakeIcon class="w-3 h-3 text-zinc-500" />
        {/snippet}
      </StatCard>

      <!-- Exchange Rate Card (Interactive) -->
      <StatCard
        label={showInverseRate ? 'eU→U' : 'U→eU'}
        value={exchangeRate}
        clickable={true}
        onclick={onToggleRate}
        class="font-mono"
      >
        {#snippet icon()}
          <RotateLeftIcon class="w-3 h-3 text-zinc-500" />
        {/snippet}
        {#snippet indicator()}
          <RestoreIcon class="w-3 h-3 text-zinc-600" />
        {/snippet}
      </StatCard>
    </div>
  </div>
</Card>

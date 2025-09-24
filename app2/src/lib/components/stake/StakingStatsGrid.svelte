<script lang="ts">
import RestoreIcon from "$lib/components/icons/RestoreIcon.svelte"
import RotateLeftIcon from "$lib/components/icons/RotateLeftIcon.svelte"
import SharpDashboardIcon from "$lib/components/icons/SharpDashboardIcon.svelte"
import SharpStakeIcon from "$lib/components/icons/SharpStakeIcon.svelte"
import SharpUpcomingIcon from "$lib/components/icons/SharpUpcomingIcon.svelte"
import Card from "$lib/components/ui/Card.svelte"
import type { IncentiveError, IncentiveResult } from "$lib/services/incentive"
import { Utils } from "@unionlabs/sdk"
import { BigDecimal, Exit, pipe } from "effect"
import * as O from "effect/Option"
import StatCard from "./StatCard.svelte"

interface Props {
  incentives: O.Option<Exit.Exit<IncentiveResult, IncentiveError>>
  exchangeRate: O.Option<string>
  showInverseRate: boolean
  onToggleRate: () => void
}

let {
  incentives,
  exchangeRate,
  showInverseRate,
  onToggleRate,
}: Props = $props()
</script>

<Card class="p-4 bg-zinc-925">
  <div class="flex flex-col gap-3">
    <div class="grid grid-cols-2 gap-2.5">
      <!-- Rewards Card -->
      <StatCard
        label="Rewards"
        value={O.isSome(incentives) && Exit.isSuccess(incentives.value)
        ? O.some(`${
          pipe(
            incentives.value.value.rates.yearly,
            BigDecimal.multiply(BigDecimal.fromBigInt(100n)),
            BigDecimal.round({ mode: "from-zero", scale: 2 }),
            Utils.formatBigDecimal,
          )
        }%`)
        : O.none()}
        subtitle={O.none()}
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
        ? O.some(
          Number(BigDecimal.unsafeToNumber(incentives.value.value.totalSupply))
            .toLocaleString("en-US", { maximumFractionDigits: 0 }),
        )
        : O.none()}
        subtitle={O.none()}
        loading={O.isNone(incentives) || !Exit.isSuccess(incentives.value)}
      >
        {#snippet icon()}
          <SharpDashboardIcon class="w-3 h-3 text-zinc-500" />
        {/snippet}
      </StatCard>

      <!-- Staked Card -->
      <StatCard
        label="Staked"
        value={pipe(
          incentives,
          O.flatMap(Exit.match({
            onFailure: () => O.none(),
            onSuccess: (value) =>
              O.some(
                Number(BigDecimal.unsafeToNumber(value.bondedTokens))
                  .toLocaleString("en-US", { maximumFractionDigits: 0 }),
              ),
          })),
        )}
        subtitle={pipe(
          incentives,
          O.flatMap(Exit.match({
            onFailure: () => O.none(),
            onSuccess: (value) =>
              O.some(
                `${(BigDecimal.unsafeToNumber(value.bondedRatio) * 100).toFixed(1)}%`,
              ),
          })),
        )}
        loading={pipe(
          incentives,
          O.match({
            onNone: () => true,
            onSome: (exit) => Exit.isFailure(exit),
          }),
        )}
      >
        {#snippet icon()}
          <SharpStakeIcon class="w-3 h-3 text-zinc-500" />
        {/snippet}
      </StatCard>

      <!-- Exchange Rate Card (Interactive) -->
      <StatCard
        label={showInverseRate ? "eU→U" : "U→eU"}
        value={exchangeRate}
        subtitle={O.none()}
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

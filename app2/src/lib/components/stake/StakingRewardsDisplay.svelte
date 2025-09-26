<script lang="ts">
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { Utils } from "@unionlabs/sdk"
import { Bond, Unbond } from "@unionlabs/sdk/schema/stake"
import { BigDecimal, pipe, Array as A } from "effect"
import * as O from "effect/Option"

interface Props {
  eUBalance: O.Option<bigint>
  redemptionRate: O.Option<BigDecimal.BigDecimal>
  stakingHistory: O.Option<A.NonEmptyReadonlyArray<Bond | Unbond>>
  proxyEuDust?: O.Option<BigDecimal.BigDecimal>
}

let {
  eUBalance,
  redemptionRate,
  stakingHistory,
  proxyEuDust = O.none(),
}: Props = $props()

// Calculate current U value from eU balance
const currentUValue = $derived<O.Option<BigDecimal.BigDecimal>>(
  O.flatMap(
    O.all([eUBalance, redemptionRate]),
    ([balance, rate]) => {
      const balanceDecimal = BigDecimal.make(balance, 18)
      const balanceNorm = BigDecimal.normalize(balanceDecimal)
      const rateNorm = BigDecimal.normalize(rate)
      const resultScaled = balanceNorm.value * rateNorm.value
      return O.some(BigDecimal.make(resultScaled, balanceNorm.scale + rateNorm.scale))
    },
  ),
)

// Include proxy dust in total value calculation
const totalUValue = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  currentUValue,
  O.map(current => {
    // Convert eU dust to U value
    const euDustInU = O.match(
      O.all([proxyEuDust, redemptionRate]),
      {
        onNone: () => BigDecimal.make(0n, 18),
        onSome: ([dust, rate]) => BigDecimal.multiply(dust, rate),
      }
    )
    // Add current value + eU dust value
    return BigDecimal.sum(current, euDustInU)
  }),
))

// Calculate total U invested from staking history (bonds only)
const totalUStaked = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  stakingHistory,
  O.map(history => {
    // Sum all bonds (U invested) - only successful ones
    const bonds = history.filter((tx): tx is Bond => {
      if (tx._tag !== "Bond") return false
      // Check if bond_success is Some(true)
      return O.isSome(tx.bond_success) && tx.bond_success.value === true
    })
    
    // Bond base_amount is in U
    const totalBonded = bonds.reduce((acc, bond) => {
      const amount = BigDecimal.make(BigInt(bond.base_amount), 18)
      return BigDecimal.sum(acc, amount)
    }, BigDecimal.fromBigInt(0n))
    
    return totalBonded
  }),
))

// Calculate rewards earned (total value including dust - total invested)
const rewardsEarned = $derived<O.Option<BigDecimal.BigDecimal>>(
  O.flatMap(
    O.all([totalUValue, totalUStaked]),
    ([totalVal, invested]) => {
      const rewards = BigDecimal.subtract(totalVal, invested)
      return O.some(rewards)
    },
  ),
)

// Calculate percentage gain
const percentageGain = $derived<O.Option<string>>(
  O.flatMap(
    O.all([rewardsEarned, totalUStaked]),
    ([rewards, invested]) => {
      // If no investment, return 0
      if (BigDecimal.isZero(invested)) return O.some("0")
      
      // Calculate percentage: (rewards / invested) * 100
      const percentage = pipe(
        rewards,
        BigDecimal.divide(invested),
        O.map(rate => BigDecimal.multiply(rate, BigDecimal.fromBigInt(100n))),
        O.map(pct => BigDecimal.round({ mode: "from-zero", scale: 2 })(pct)),
        O.map(Utils.formatBigDecimal),
        O.getOrElse(() => "0")
      )
      
      return O.some(percentage)
    },
  ),
)

// Format display values
const formatValue = (value: O.Option<BigDecimal.BigDecimal>) =>
  pipe(
    value,
    O.map(v => BigDecimal.round({ mode: "from-zero", scale: 2 })(v)),
    O.map(Utils.formatBigDecimal),
    O.getOrElse(() => "0"),
  )

const formatRewards = (rewards: O.Option<BigDecimal.BigDecimal>) => {
  return pipe(
    rewards,
    O.map(r => {
      const rounded = BigDecimal.round({ mode: "from-zero", scale: 4 })(r)
      const formatted = Utils.formatBigDecimal(rounded)
      const isPositive = BigDecimal.greaterThan(r, BigDecimal.fromBigInt(0n))
      return isPositive ? `+${formatted}` : formatted
    }),
    O.getOrElse(() => "—")
  )
}
</script>

<div class="rounded-lg border border-zinc-800/70 bg-zinc-900/40 px-3 py-3">
  <div class="flex items-baseline justify-between border-b border-zinc-800/60 pb-2">
    <span class="text-[11px] uppercase tracking-wide text-zinc-500">Total Position</span>
    {#if O.isSome(totalUValue)}
      <span class="text-sm font-semibold text-zinc-100">{formatValue(totalUValue)} U</span>
    {:else}
      <Skeleton class="w-24 h-4" />
    {/if}
  </div>

  <div class="mt-2 grid grid-cols-[minmax(0,1fr)_auto] items-center gap-y-1.5 text-[12px]">
    <span class="text-zinc-500">Total Staked</span>
    {#if O.isSome(totalUStaked)}
      <span class="text-sm font-medium text-zinc-200">{formatValue(totalUStaked)} U</span>
    {:else}
      <Skeleton class="w-20 h-3.5" />
    {/if}

    <span class="text-zinc-500">Wallet eU</span>
    {#if O.isSome(eUBalance)}
      {@const balanceDecimal = BigDecimal.make(eUBalance.value, 18)}
      <span class="text-sm font-medium text-zinc-200">
        {Utils.formatBigDecimal(BigDecimal.round({ mode: "from-zero", scale: 4 })(balanceDecimal))} eU
      </span>
    {:else}
      <Skeleton class="w-16 h-3.5" />
    {/if}

    {#if O.isSome(proxyEuDust) && BigDecimal.greaterThan(proxyEuDust.value, BigDecimal.make(0n, 18))}
      <span class="text-zinc-500">Proxy eU (dust)</span>
      <span class="text-sm font-medium text-amber-300">
        {Utils.formatBigDecimal(BigDecimal.round({ mode: "from-zero", scale: 4 })(proxyEuDust.value))} eU
      </span>
    {/if}

    <span class="text-zinc-500">Net Growth</span>
    {#if O.isSome(rewardsEarned)}
      {@const isPositive = BigDecimal.greaterThan(rewardsEarned.value, BigDecimal.fromBigInt(0n))}
      <div class="flex items-baseline gap-2">
        <span class={isPositive
          ? "text-sm font-semibold text-emerald-400"
          : "text-sm font-semibold text-red-400"
        }>
          {formatRewards(rewardsEarned)} U
        </span>
        {#if O.isSome(percentageGain)}
          <span class={isPositive
            ? "text-[11px] font-medium text-emerald-300"
            : "text-[11px] font-medium text-red-400"
          }>
            {isPositive ? '+' : ''}{percentageGain.value}%
          </span>
        {/if}
      </div>
    {:else}
      <Skeleton class="w-24 h-3.5" />
    {/if}
  </div>
</div>

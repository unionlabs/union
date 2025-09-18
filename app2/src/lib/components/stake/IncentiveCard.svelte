<script lang="ts">
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import { formatIncentive, IncentiveError, IncentiveResult } from "$lib/services/incentive"
import { matchRuntimeResult } from "$lib/utils/snippets.svelte"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import { BigDecimal, pipe } from "effect"
import type { Exit, Option } from "effect"
import * as O from "effect/Option"

interface Props {
  incentives: Option.Option<Exit.Exit<IncentiveResult, IncentiveError>>
  stakeAmount: Option.Option<bigint>
  evmChain: Option.Option<Chain>
  uOnEvmToken: Option.Option<Token>
}

let { incentives, stakeAmount, evmChain, uOnEvmToken }: Props = $props()

let selectedTab: "incentives" | "stats" = $state("incentives")

function formatPercentage(value: number): string {
  return formatIncentive(value)
}

function formatLargeNumber(value: number): string {
  if (value >= 1_000_000_000) {
    return `${(value / 1_000_000_000).toFixed(1)}B`
  } else if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`
  } else if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`
  }
  return value.toLocaleString()
}

const incentiveAmounts = $derived(pipe(
  O.Do,
  O.bind("amount", () => stakeAmount),
  O.bind("incentiveData", () =>
    pipe(
      incentives,
      O.filter(exit => exit._tag === "Success"),
      O.map(exit => exit.value),
    )),
  O.filter(({ amount }) => amount > 0n),
  O.flatMap(({ amount, incentiveData }) =>
    O.gen(function*() {
      const stakingRewards = incentiveData.rates.yearly

      const stakingRewardsBasisPoints = pipe(
        BigDecimal.multiply(stakingRewards, BigDecimal.fromBigInt(10_000n)),
        BigDecimal.floor,
      )

      const yearlyWei = yield* pipe(
        BigDecimal.multiply(BigDecimal.fromBigInt(amount), stakingRewardsBasisPoints),
        BigDecimal.divide(BigDecimal.fromBigInt(10_000n)),
      )

      const monthly = yield* BigDecimal.divide(yearlyWei, BigDecimal.fromBigInt(12n))
      const weekly = yield* BigDecimal.divide(yearlyWei, BigDecimal.fromBigInt(52n))
      const daily = yield* BigDecimal.divide(yearlyWei, BigDecimal.fromBigInt(365n))
      console.log("result", daily)
      const stakingRewardsPercentage = BigDecimal.multiply(
        stakingRewards,
        BigDecimal.fromBigInt(100n),
      )

      const result = {
        yearly: yearlyWei,
        monthly,
        weekly,
        daily,
        stakingRewardsPercentage: stakingRewardsPercentage,
      } as const

      console.log("result", { result })

      return result
    })
  ),
))

const truncateBigDecimal = (self: BigDecimal.BigDecimal) =>
  pipe(
    self,
    BigDecimal.multiply(BigDecimal.make(1n, 2)),
    BigDecimal.scale(2),
    x => x.value,
  )
</script>

{#snippet renderIncentiveData(data: any)}
  <div class="flex flex-col h-full">
    <div class="p-4 border-b border-zinc-800">
      <Tabs
        items={[
          { id: "incentives", label: "Incentives" },
          { id: "stats", label: "Stats" },
        ]}
        activeId={selectedTab}
        onTabChange={(id) => selectedTab = id as "incentives" | "stats"}
      />
    </div>

    <div class="p-4 flex flex-col flex-1">
      {#if selectedTab === "incentives"}
        <div class="relative flex-1">
          <div class="space-y-3">
            <div class="flex justify-between items-center p-3 bg-zinc-800/20 rounded-lg">
              <span class="text-sm text-zinc-400">Daily (est):</span>
              <div>
                {#if O.isSome(evmChain) && O.isSome(uOnEvmToken)}
                  <TokenComponent
                    chain={evmChain.value}
                    denom={uOnEvmToken.value.denom}
                    amount={TokenRawAmount.make(
                      O.isSome(incentiveAmounts)
                        ? truncateBigDecimal(incentiveAmounts.value.daily)
                        : 0n,
                    )}
                    showWrapping={false}
                    showSymbol={true}
                    showIcon={true}
                    maxDecimals={4}
                  />
                {:else}
                  <div class="w-20 h-4 bg-zinc-700/50 rounded animate-pulse"></div>
                {/if}
              </div>
            </div>

            <div class="flex justify-between items-center p-3 bg-zinc-800/20 rounded-lg">
              <span class="text-sm text-zinc-400">Weekly (est):</span>
              <div>
                {#if O.isSome(evmChain) && O.isSome(uOnEvmToken)}
                  <TokenComponent
                    chain={evmChain.value}
                    denom={uOnEvmToken.value.denom}
                    amount={TokenRawAmount.make(
                      O.isSome(incentiveAmounts)
                        ? truncateBigDecimal(incentiveAmounts.value.weekly)
                        : 0n,
                    )}
                    showWrapping={false}
                    showSymbol={true}
                    showIcon={true}
                    maxDecimals={4}
                  />
                {:else}
                  <div class="w-20 h-4 bg-zinc-700/50 rounded animate-pulse"></div>
                {/if}
              </div>
            </div>

            <div class="flex justify-between items-center p-3 bg-zinc-800/20 rounded-lg">
              <span class="text-sm text-zinc-400">Monthly (est):</span>
              <div>
                {#if O.isSome(evmChain) && O.isSome(uOnEvmToken)}
                  <TokenComponent
                    chain={evmChain.value}
                    denom={uOnEvmToken.value.denom}
                    amount={TokenRawAmount.make(
                      O.isSome(incentiveAmounts)
                        ? truncateBigDecimal(incentiveAmounts.value.monthly)
                        : 0n,
                    )}
                    showWrapping={false}
                    showSymbol={true}
                    showIcon={true}
                    maxDecimals={4}
                  />
                {:else}
                  <div class="w-20 h-4 bg-zinc-700/50 rounded animate-pulse"></div>
                {/if}
              </div>
            </div>

            <div class="flex justify-between items-center p-3 bg-zinc-800/20 rounded-lg">
              <span class="text-sm text-zinc-400">Yearly (est):</span>
              <div>
                {#if O.isSome(evmChain) && O.isSome(uOnEvmToken)}
                  <TokenComponent
                    chain={evmChain.value}
                    denom={uOnEvmToken.value.denom}
                    amount={TokenRawAmount.make(
                      O.isSome(incentiveAmounts) ? incentiveAmounts.value.yearly.value : 0n,
                    )}
                    showWrapping={false}
                    showSymbol={true}
                    showIcon={true}
                    maxDecimals={4}
                  />
                {:else}
                  <div class="w-20 h-4 bg-zinc-700/50 rounded animate-pulse"></div>
                {/if}
              </div>
            </div>
          </div>

          <div class="text-xs text-zinc-500 text-center mt-3">
            Incentives are estimates based on current rate
          </div>
        </div>
      {:else if selectedTab === "stats"}
        <div class="flex justify-center mb-2">
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg w-full">
            <div class="text-4xl font-bold text-accent mb-2">
              {formatPercentage(data.rates.yearly)}
            </div>
            <div class="text-sm text-zinc-400">Annual Compounded Rate</div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2 flex-1">
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-mono text-white mb-1">
              {formatLargeNumber(data.totalSupply)}
            </div>
            <div class="text-xs text-zinc-500">Total Supply</div>
          </div>
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-mono text-white mb-1">
              {formatLargeNumber(data.bondedTokens)}
            </div>
            <div class="text-xs text-zinc-500">Bonded Tokens</div>
          </div>
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-semibold text-white mb-1">
              {(data.bondedRatio * 100).toFixed(1)}%
            </div>
            <div class="text-xs text-zinc-500">Bonded Ratio</div>
          </div>
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-semibold text-white mb-1">
              {(data.inflation * 100).toFixed(1)}%
            </div>
            <div class="text-xs text-zinc-500">Inflation Rate</div>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/snippet}

{#snippet renderLoading()}
  <div class="flex flex-col h-full">
    <!-- Tabs Skeleton -->
    <div class="pt-2 px-2 pb-2 border-b border-zinc-800">
      <div class="flex gap-1">
        <div class="h-8 w-20 bg-zinc-700/50 rounded animate-pulse"></div>
        <div class="h-8 w-20 bg-zinc-700/50 rounded animate-pulse"></div>
      </div>
    </div>

    <!-- Content Skeleton -->
    <div class="px-3 pb-3 flex flex-col flex-1">
      <!-- Main Display Skeleton -->
      <div class="flex justify-center mb-4 mt-3">
        <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
          <div class="h-12 w-32 bg-zinc-700/50 rounded mb-2 animate-pulse"></div>
          <div class="h-4 w-36 bg-zinc-700/50 rounded animate-pulse"></div>
        </div>
      </div>

      <!-- 2x2 Grid Skeleton -->
      <div class="grid grid-cols-2 gap-4 flex-1">
        {#each Array(4) as _}
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="h-5 w-16 bg-zinc-700/50 rounded mb-1 animate-pulse"></div>
            <div class="h-3 w-20 bg-zinc-700/50 rounded animate-pulse"></div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/snippet}

{#snippet renderError(error: any)}
  <div>
    <div class="text-center">
      <div class="text-red-400 text-sm mb-2">Failed to load incentive data</div>
      <div class="text-xs text-zinc-500">
        {error?.message || "Unknown error occurred"}
      </div>
    </div>
  </div>
{/snippet}

<Card class="p-0">
  {@render matchRuntimeResult(incentives, {
      onSuccess: renderIncentiveData,
      onFailure: renderError,
      onNone: renderLoading,
    })}
</Card>

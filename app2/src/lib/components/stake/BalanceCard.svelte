<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import { Utils } from "@unionlabs/sdk"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import { BigDecimal, pipe } from "effect"
import * as O from "effect/Option"
import TokenBalanceRow from "./TokenBalanceRow.svelte"

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<Token>
  eUOnEvmToken: O.Option<Token>
  uOnEvmBalance: O.Option<bigint>
  eUOnEvmBalance: O.Option<bigint>
  purchaseRate: O.Option<BigDecimal.BigDecimal>
}

let {
  evmChain,
  uOnEvmToken,
  eUOnEvmToken,
  uOnEvmBalance,
  eUOnEvmBalance,
  purchaseRate,
}: Props = $props()

// Calculate eU value of U balance
const uBalanceInEU = $derived<O.Option<BigDecimal.BigDecimal>>(
  O.map(
    O.all([uOnEvmBalance, purchaseRate]),
    ([balance, rate]) => {
      // Convert balance to decimal with 18 decimals
      const balanceDecimal = BigDecimal.make(balance, 18)
      // Multiply by purchase rate
      const balanceNorm = BigDecimal.normalize(balanceDecimal)
      const rateNorm = BigDecimal.normalize(rate)
      const resultScaled = balanceNorm.value * rateNorm.value
      return BigDecimal.make(resultScaled, balanceNorm.scale + rateNorm.scale)
    },
  ),
)

// Calculate U value of eU balance
const eUBalanceInU = $derived<O.Option<BigDecimal.BigDecimal>>(
  O.flatMap(
    O.all([eUOnEvmBalance, purchaseRate]),
    ([balance, rate]) => {
      const balanceDecimal = BigDecimal.make(balance, 18)
      return BigDecimal.divide(balanceDecimal, rate)
    },
  ),
)

type BalanceTab = "balances" | "rewards"
let selectedTab = $state<BalanceTab>("balances")
</script>

<Card divided>
  <div class="px-4 py-2.5 border-b border-zinc-800">
    <Tabs
      items={[
        { id: "balances", label: "Balances" },
        // { id: "rewards", label: "Rewards" },
      ]}
      activeId={selectedTab}
      onTabChange={(id) => selectedTab = id as BalanceTab}
    />
  </div>

  <div class="p-4">
    {#if selectedTab === "balances"}
      <div class="flex flex-col gap-3">
        <!-- U Balance Card -->
        <TokenBalanceRow
          chain={evmChain}
          token={uOnEvmToken}
          balance={uOnEvmBalance}
          symbol="U"
          hoverable={true}
          title="Union Token"
          class="group"
          subtitle={pipe(
            uBalanceInEU,
            O.map((val: BigDecimal.BigDecimal) =>
              `≈ ${
                Utils.formatBigDecimal(BigDecimal.round({ mode: "from-zero", scale: 4 })(val))
              } eU`
            ),
            O.getOrElse(() =>
              O.isNone(purchaseRate) && O.isSome(uOnEvmBalance) ? "loading" : ""
            ),
          )}
        />

        <!-- eU Balance Card -->
        <TokenBalanceRow
          chain={evmChain}
          token={eUOnEvmToken}
          balance={eUOnEvmBalance}
          symbol="eU"
          hoverable={true}
          title="Staked Union Token"
          subtitle={pipe(
            eUBalanceInU,
            O.map((val: BigDecimal.BigDecimal) =>
              `≈ ${
                Utils.formatBigDecimal(BigDecimal.round({ mode: "from-zero", scale: 4 })(val))
              } U`
            ),
            O.getOrElse(() =>
              O.isNone(purchaseRate) && O.isSome(eUOnEvmBalance) ? "loading" : ""
            ),
          )}
        />

        <!-- Transfer U Button -->
        <Button
          variant="primary"
          href="/transfer?source=union-1&destination=1&asset=0x6175"
          class="w-full h-9 text-sm font-semibold mt-1"
        >
          Transfer U →
        </Button>
      </div>
    {:else}
      <div class="text-center py-8 text-zinc-500">
        Rewards coming soon...
      </div>
    {/if}
  </div>
</Card>

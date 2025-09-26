<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { DESTINATION_CHANNEL_ID } from "$lib/stake/config"
import { predictProxy } from "$lib/stake/instantiate2"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { Utils } from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { EU_LST } from "@unionlabs/sdk/Constants"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import { Array, BigDecimal, Effect, Exit, pipe } from "effect"
import * as O from "effect/Option"
import ProxyDustRecovery from "./ProxyDustRecovery.svelte"
import StakingRewardsDisplay from "./StakingRewardsDisplay.svelte"
import TokenBalanceRow from "./TokenBalanceRow.svelte"

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<Token>
  eUOnEvmToken: O.Option<Token>
  uOnEvmBalance: O.Option<bigint>
  eUOnEvmBalance: O.Option<bigint>
  purchaseRate: O.Option<BigDecimal.BigDecimal>
  redemptionRate: O.Option<BigDecimal.BigDecimal>
  stakingHistory?: O.Option<Array.NonEmptyReadonlyArray<any>>
}

let {
  evmChain,
  uOnEvmToken,
  eUOnEvmToken,
  uOnEvmBalance,
  eUOnEvmBalance,
  purchaseRate,
  redemptionRate,
  stakingHistory = O.none(),
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

type BalanceTab = "balances" | "rewards" | "dust"
let selectedTab = $state<BalanceTab>("balances")

// Query proxy balance for dust
const proxyDustData = runPromiseExit$(() => {
  void WalletStore.evmAddress

  return pipe(
    WalletStore.evmAddress,
    O.match({
      onNone: () => Effect.succeed({ euDust: BigDecimal.make(0n, 18), proxyAddress: "" }),
      onSome: (address) =>
        Effect.gen(function*() {
          const proxy = yield* predictProxy({
            path: 0n,
            channel: DESTINATION_CHANNEL_ID,
            sender: address,
          })

          // Query eU balance on proxy
          const euBalanceResponse = yield* pipe(
            Cosmos.queryContract(EU_LST, {
              balance: { address: proxy.address },
            }),
            Effect.map(resp => resp as { balance: string }),
            Effect.catchAll(() => Effect.succeed({ balance: "0" })),
          )

          // Convert the raw balance string (with 18 decimals) to BigDecimal
          const euDustRaw = BigInt(euBalanceResponse.balance)
          const euDust = BigDecimal.make(euDustRaw, 18) // 18 decimals for eU

          return { euDust, proxyAddress: proxy.address }
        }),
    }),
    Effect.provide(Cosmos.Client.Live("https://rpc.union.build")),
    Effect.catchAll(() => Effect.succeed({ euDust: BigDecimal.make(0n, 18), proxyAddress: "" })),
  )
})

const proxyEuDust = $derived(
  O.isSome(proxyDustData.current) && Exit.isSuccess(proxyDustData.current.value)
    ? O.some(proxyDustData.current.value.value.euDust)
    : O.none(),
)

const proxyAddress = $derived(
  O.isSome(proxyDustData.current) && Exit.isSuccess(proxyDustData.current.value)
    ? O.some(proxyDustData.current.value.value.proxyAddress)
    : O.none(),
)
</script>

<Card divided>
  <div class="px-4 py-2.5 border-b border-zinc-800">
    <Tabs
      items={[
        { id: "balances", label: "Balances" },
        { id: "rewards", label: "Rewards" },
        { id: "dust", label: "Dust" },
      ]}
      activeId={selectedTab}
      onTabChange={(id) => selectedTab = id as BalanceTab}
    />
  </div>

  <div class="p-4 min-h-[250px] flex flex-col">
    {#if selectedTab === "balances"}
      <div class="flex flex-col gap-3 flex-1">
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

        <div class="flex-1"></div>

        <!-- Transfer U Button -->
        <Button
          variant="primary"
          href="/transfer?source=union-1&destination=1&asset=0x6175"
          class="w-full"
        >
          Transfer U →
        </Button>
      </div>
    {:else if selectedTab === "rewards"}
      <div class="flex-1">
        <StakingRewardsDisplay
          eUBalance={eUOnEvmBalance}
          {redemptionRate}
          {stakingHistory}
          {proxyEuDust}
        />
      </div>
    {:else if selectedTab === "dust"}
      <div class="flex-1 flex min-h-0">
        <ProxyDustRecovery
          {evmChain}
          {eUOnEvmToken}
          {redemptionRate}
          {proxyEuDust}
          {proxyAddress}
        />
      </div>
    {/if}
  </div>
</Card>

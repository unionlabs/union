<script lang="ts">
import BalanceCard from "$lib/components/stake/BalanceCard.svelte"
import BondComponent from "$lib/components/stake/BondComponent.svelte"
import StakingHistoryCard from "$lib/components/stake/StakingHistoryCard.svelte"
import StakingStatsGrid from "$lib/components/stake/StakingStatsGrid.svelte"
import UnbondComponent from "$lib/components/stake/UnbondComponent.svelte"
import WithdrawalComponent from "$lib/components/stake/WithdrawalComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import * as AppRuntime from "$lib/runtime"
import { calculateIncentive } from "$lib/services/incentive"
import { ETHEREUM_CHAIN_ID } from "$lib/stake/config"
import { balancesStore as BalanceStore } from "$lib/stores/balances.svelte"
import { chains as ChainStore } from "$lib/stores/chains.svelte"
import { tokensStore as TokenStore } from "$lib/stores/tokens.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { FetchHttpClient } from "@effect/platform"
import { Staking, Ucs05, Utils } from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { EU_ERC20, EU_LST, EU_STAKING_HUB, U_ERC20 } from "@unionlabs/sdk/Constants"
import { Indexer } from "@unionlabs/sdk/Indexer"
import { UniversalChainId } from "@unionlabs/sdk/schema"
import { Bond, Unbond } from "@unionlabs/sdk/schema/stake"
import {
  BigDecimal,
  Brand,
  ConfigProvider,
  DateTime,
  Effect,
  Layer,
  Order,
  pipe,
  Schema,
} from "effect"
import * as A from "effect/Array"
import { constVoid } from "effect/Function"
import * as O from "effect/Option"
import { onMount } from "svelte"

type StakeTab = "bond" | "unbond" | "withdraw"
type TableFilter = "all" | "bond" | "unbond"

const EVM_UNIVERSAL_CHAIN_ID = ETHEREUM_CHAIN_ID

// State for rate display toggle
let showInverseRate = $state(false)

const QlpConfigProvider = pipe(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://development.graphql.union.build/v1/graphql"],
    ]),
  ),
  Layer.setConfigProvider,
)

const uOnEvmToken = $derived(pipe(
  TokenStore.getData(EVM_UNIVERSAL_CHAIN_ID),
  O.flatMap(A.findFirst(xs => Brand.unbranded(xs.denom) === U_ERC20.address.toLowerCase())),
))

const eUOnEvmToken = $derived(pipe(
  TokenStore.getData(EVM_UNIVERSAL_CHAIN_ID),
  O.flatMap(A.findFirst(xs => Brand.unbranded(xs.denom) === EU_ERC20.address.toLowerCase())),
))

let selectedTab = $state<StakeTab>("bond")
let refreshTrigger = $state<number>(0)

// Shared state for bond amount (to calculate earnings)
let bondAmount = $state<O.Option<bigint>>(O.none())

const refreshBondData = () => {
  refreshTrigger = Date.now()
}

const data = AppRuntime.runPromiseExit$(() => {
  void WalletStore.evmAddress
  void refreshTrigger

  return Effect.gen(function*() {
    const staking = yield* Staking.Staking
    const address = yield* WalletStore.evmAddress
    return yield* pipe(
      Effect.all([
        staking.getBonds(Staking.GetBonds.make({ addresses: [address] })),
        staking.getUnbonds(Staking.GetUnbonds.make({ addresses: [address] })),
      ], { concurrency: "unbounded" }),
      Effect.map(A.getSomes),
      Effect.map(A.flatten),
      Effect.map(A.sort(pipe(
        Order.mapInput<Date, Bond | Unbond>(
          Order.Date,
          (x) => DateTime.toDate(x.sortDate),
        ),
        Order.reverse,
      ))),
      Effect.map(O.liftPredicate(A.isNonEmptyReadonlyArray)),
      Effect.map(x => x as O.Option<readonly [(Bond | Unbond), ...Array<(Bond | Unbond)>]>),
      Effect.tap(result =>
        Effect.sync(() => {
          setTimeout(() => {
            refreshTrigger = Date.now()
          }, 10000)
        })
      ),
    )
  }).pipe(
    Effect.provide(Staking.Staking.DefaultWithoutDependencies),
    Effect.provide(Layer.fresh(Indexer.Default)),
    Effect.provide(QlpConfigProvider),
  )
})

const incentives = AppRuntime.runPromiseExit$(() => {
  return Effect.gen(function*() {
    const incentive = yield* calculateIncentive
    console.log("Incentive data loaded:", incentive)
    return incentive
  }).pipe(
    Effect.provide(FetchHttpClient.layer),
    Effect.catchAll((error) => {
      console.error("Failed to load incentive data:", error)
      return Effect.fail(error)
    }),
  )
})

// Fetch staking rates to get the purchase rate (eU/U exchange rate)
const stakingRates = AppRuntime.runPromiseExit$(() =>
  Effect.gen(function*() {
    return yield* pipe(
      Cosmos.queryContract(
        EU_STAKING_HUB,
        {
          accounting_state: {},
        },
      ),
      Effect.flatMap(Schema.decodeUnknown(Schema.Struct({
        total_assets: Schema.BigInt,
        total_shares: Schema.BigInt,
        total_reward_amount: Schema.BigInt,
        redemption_rate: Schema.BigDecimal,
        purchase_rate: Schema.BigDecimal,
      }))),
      Effect.provide(Cosmos.Client.Live("https://rpc.union-testnet-10.union.chain.kitchen")),
    )
  })
)

const evmChain = $derived(pipe(
  ChainStore.data,
  O.flatMap(A.findFirst(x => x.universal_chain_id === EVM_UNIVERSAL_CHAIN_ID)),
))

onMount(() => {
  BalanceStore.interruptBalanceFetching()
  TokenStore.fetchTokens(EVM_UNIVERSAL_CHAIN_ID)
})

$effect(() => {
  O.match(O.all([evmChain, WalletStore.evmAddress, uOnEvmToken]), {
    onSome: ([chain, address, { denom }]) =>
      BalanceStore.fetchBalances(
        chain,
        Ucs05.anyDisplayToCanonical(address),
        denom,
        "1 second",
      ),
    onNone: constVoid,
  })

  O.match(O.all([evmChain, WalletStore.evmAddress, eUOnEvmToken]), {
    onSome: ([chain, address, { denom }]) =>
      BalanceStore.fetchBalances(
        chain,
        Ucs05.anyDisplayToCanonical(address),
        denom,
        "1 second",
      ),
    onNone: constVoid,
  })
})

const uOnEvmBalance = $derived(pipe(
  O.all([evmChain, WalletStore.evmAddress, uOnEvmToken]),
  O.flatMap(([chain, address, { denom }]) =>
    BalanceStore.getBalance(
      chain.universal_chain_id,
      Ucs05.anyDisplayToCanonical(address),
      denom,
    )
  ),
))

const eUOnEvmBalance = $derived(pipe(
  O.all([evmChain, WalletStore.evmAddress, eUOnEvmToken]),
  O.flatMap(([chain, address, { denom }]) =>
    BalanceStore.getBalance(
      chain.universal_chain_id,
      Ucs05.anyDisplayToCanonical(address),
      denom,
    )
  ),
))

// Get the exchange rate from the staking contract
const exchangeRate = $derived.by(() => {
  if (O.isSome(stakingRates.current) && stakingRates.current.value._tag === "Success") {
    const purchaseRate = stakingRates.current.value.value.purchase_rate

    if (showInverseRate) {
      // eU/U rate: how much U you need per eU (inverse of purchase rate)
      const inverseRate = pipe(
        BigDecimal.divide(BigDecimal.fromBigInt(1n), purchaseRate),
        O.getOrElse(() => BigDecimal.fromBigInt(0n)),
        BigDecimal.round({ mode: "from-zero", scale: 4 }),
        Utils.formatBigDecimal,
      )
      return inverseRate
    } else {
      // U/eU rate: how much eU you get per U (purchase rate)
      const rateNumber = pipe(
        purchaseRate,
        BigDecimal.round({ mode: "from-zero", scale: 4 }),
        Utils.formatBigDecimal,
      )
      return rateNumber
    }
  }
  return "—"
})

$inspect(data)
</script>

<Sections>
  <!-- Mobile: Balance Card first -->
  <div class="lg:hidden">
    <BalanceCard
      {evmChain}
      {uOnEvmToken}
      {eUOnEvmToken}
      {uOnEvmBalance}
      {eUOnEvmBalance}
    />
  </div>

  <!-- Main grid with actions on left, balance+stats on right (desktop) -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Left Column: Staking Actions Card -->
    <Card divided>
      <div class="px-4 py-2.5 border-b border-zinc-800">
        <Tabs
          items={[
            { id: "bond", label: "Stake" },
            { id: "unbond", label: "Unstake" },
            { id: "withdraw", label: "Withdraw" },
          ]}
          activeId={selectedTab}
          onTabChange={(id) => selectedTab = id as StakeTab}
          class="text-xs"
        />
      </div>

      <div class="p-4">
        {#if selectedTab === "bond"}
          <BondComponent
            {evmChain}
            {uOnEvmToken}
            {eUOnEvmToken}
            {uOnEvmBalance}
            bind:bondAmount
            onBondSuccess={refreshBondData}
          />
        {:else if selectedTab === "unbond"}
          <UnbondComponent
            {evmChain}
            {uOnEvmToken}
            {eUOnEvmToken}
            {eUOnEvmBalance}
            onUnbondSuccess={refreshBondData}
          />
        {:else if selectedTab === "withdraw"}
          <WithdrawalComponent
            {evmChain}
            {uOnEvmToken}
            onWithdrawSuccess={refreshBondData}
          />
        {/if}
      </div>
    </Card>

    <!-- Right Column: Balance Card + Stats Grid (desktop only) -->
    <div class="hidden lg:block space-y-6">
      <!-- Balance Card -->
      <BalanceCard
        {evmChain}
        {uOnEvmToken}
        {eUOnEvmToken}
        {uOnEvmBalance}
        {eUOnEvmBalance}
      />

      <!-- Stats Grid -->
      <StakingStatsGrid
        incentives={incentives.current}
        {exchangeRate}
        {showInverseRate}
        onToggleRate={() => showInverseRate = !showInverseRate}
      />
    </div>
  </div>

  <!-- Mobile: Stats Grid after actions -->
  <div class="lg:hidden">
    <StakingStatsGrid
      incentives={incentives.current}
      {exchangeRate}
      {showInverseRate}
      onToggleRate={() => showInverseRate = !showInverseRate}
    />
  </div>

  <!-- Staking History (separate, below everything) -->
  <StakingHistoryCard
    data={data.current}
    walletConnected={O.isSome(WalletStore.evmAddress)}
  />
</Sections>

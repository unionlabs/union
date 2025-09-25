<script lang="ts">
import EscherTextLogo from "$lib/components/icons/EscherTextLogo.svelte"
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
import { StakingRatesSchema } from "$lib/stake/schemas"
import { balancesStore as BalanceStore } from "$lib/stores/balances.svelte"
import { chains as ChainStore } from "$lib/stores/chains.svelte"
import { tokensStore as TokenStore } from "$lib/stores/tokens.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { FetchHttpClient } from "@effect/platform"
import { Staking, Ucs05, Utils } from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { EU_ERC20, EU_LST, EU_STAKING_HUB, U_ERC20 } from "@unionlabs/sdk/Constants"
import { Indexer } from "@unionlabs/sdk/Indexer"
import { Bond, Unbond } from "@unionlabs/sdk/schema/stake"
import {
  BigDecimal,
  Brand,
  ConfigProvider,
  DateTime,
  Effect,
  Exit,
  Layer,
  Order,
  pipe,
  Schedule,
  Schema,
} from "effect"
import * as A from "effect/Array"
import { constVoid } from "effect/Function"
import * as O from "effect/Option"
import { onMount } from "svelte"

type StakeTab = "bond" | "unbond" | "withdraw"
type TableFilter = "all" | "bond" | "unbond"

const EVM_UNIVERSAL_CHAIN_ID = ETHEREUM_CHAIN_ID

let showInverseRate = $state(false)

const QlpConfigProvider = pipe(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://graphql.union.build/v1/graphql"],
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
let bondAmount = $state<O.Option<bigint>>(O.none())

const refreshStakingData = () => {
  setTimeout(() => {
    refreshTrigger = Date.now()
  }, 1000)
}

// State to hold the latest staking data
let stakingData = $state<O.Option<readonly [(Bond | Unbond), ...Array<(Bond | Unbond)>]>>(O.none())

// Start the polling effect that updates stakingData
const stakingPoll = AppRuntime.runPromiseExit$(() => {
  void WalletStore.evmAddress // React to wallet changes
  void refreshTrigger // React to refresh triggers

  return pipe(
    WalletStore.evmAddress,
    Effect.flatMap(address =>
      pipe(
        Effect.gen(function*() {
          const staking = yield* Staking.Staking
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
          )
        }).pipe(
          Effect.provide(Staking.Staking.DefaultWithoutDependencies),
          Effect.provide(Layer.fresh(Indexer.Default)),
          Effect.provide(QlpConfigProvider),
        ),
        Effect.tap(result =>
          Effect.sync(() => {
            stakingData = result
          })
        ),
        Effect.repeat(Schedule.addDelay(Schedule.repeatForever, () => "10 seconds")),
        Effect.asVoid,
      )
    ),
  )
}, { onInterrupt: "ignore" })

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

const stakingRates = AppRuntime.runPromiseExit$(() =>
  Effect.gen(function*() {
    return yield* pipe(
      Cosmos.queryContract(
        EU_STAKING_HUB,
        {
          accounting_state: {},
        },
      ),
      Effect.flatMap(Schema.decodeUnknown(StakingRatesSchema)),
      Effect.provide(Cosmos.Client.Live("https://rpc.union.build")),
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

const stakingRatesData = $derived(pipe(
  stakingRates.current,
  O.flatMap(Exit.match({
    onFailure: () => O.none(),
    onSuccess: value => O.some(value),
  })),
))

const purchaseRate = $derived(pipe(
  stakingRatesData,
  O.map(rates => rates.purchase_rate),
))

const exchangeRate = $derived(pipe(
  purchaseRate,
  O.map(rate =>
    showInverseRate
      ? pipe(
        BigDecimal.divide(BigDecimal.fromBigInt(1n), rate),
        O.getOrElse(() => BigDecimal.fromBigInt(0n)),
        BigDecimal.round({ mode: "from-zero", scale: 4 }),
        Utils.formatBigDecimal,
      )
      : pipe(
        rate,
        BigDecimal.round({ mode: "from-zero", scale: 4 }),
        Utils.formatBigDecimal,
      )
  ),
))
</script>

<Sections>
  <!-- Escher Banner Card -->
  <Card class="bg-white p-0 overflow-hidden shadow-sm border-0">
    <div class="relative px-6 py-4 md:px-8 md:py-5">
      <!-- Background gradient - subtle and refined -->
      <div class="absolute inset-0 bg-gradient-to-r from-[#0008ff]/8 via-[#0008ff]/3 to-transparent">
      </div>

      <!-- Content container -->
      <div class="relative z-10 flex flex-col md:flex-row items-center md:justify-between text-center md:text-left gap-3 md:gap-0">
        <!-- Heading with logo -->
        <div class="flex flex-col md:flex-row items-center gap-1.5 md:gap-2.5">
          <h1 class="text-base md:text-lg font-medium text-gray-800 tracking-tight">
            Powered by
          </h1>
          <EscherTextLogo class="h-6 w-auto text-gray-800" />
        </div>

        <!-- CTA Button -->
        <a
          target="_blank"
          class="flex items-center gap-1.5 bg-[#0008ff] hover:bg-[#0006dd] transition-colors duration-200 rounded-md px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:shadow-md mt-4 md:mt-0"
          href="https://www.escher.finance/"
        >
          <div class="flex items-center justify-center">
            <svg
              class="w-3.5 h-3.5 text-white"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
              />
            </svg>
          </div>
          <span>Visit Escher App</span>
        </a>
      </div>
    </div>
  </Card>

  <!-- Mobile: Balance Card first -->
  <div class="lg:hidden">
    <BalanceCard
      {evmChain}
      {uOnEvmToken}
      {eUOnEvmToken}
      {uOnEvmBalance}
      {eUOnEvmBalance}
      {purchaseRate}
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
          onTabChange={(id: string) => selectedTab = id as StakeTab}
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
            stakingRates={stakingRatesData}
            bind:bondAmount
            onBondSuccess={refreshStakingData}
          />
        {:else if selectedTab === "unbond"}
          <UnbondComponent
            {evmChain}
            {uOnEvmToken}
            {eUOnEvmToken}
            {eUOnEvmBalance}
            stakingRates={stakingRatesData}
            onUnbondSuccess={refreshStakingData}
          />
        {:else if selectedTab === "withdraw"}
          <WithdrawalComponent
            {evmChain}
            {uOnEvmToken}
            onWithdrawSuccess={refreshStakingData}
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
        {purchaseRate}
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
    data={O.some(Exit.succeed(stakingData))}
    walletConnected={O.isSome(WalletStore.evmAddress)}
  />
</Sections>

<script lang="ts">
import StakingListItemComponent from "$lib/components/model/StakingListItemComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import BondComponent from "$lib/components/stake/BondComponent.svelte"
import IncentiveCard from "$lib/components/stake/IncentiveCard.svelte"
import StakingHistoryCard from "$lib/components/stake/StakingHistoryCard.svelte"
import UnbondComponent from "$lib/components/stake/UnbondComponent.svelte"
import WithdrawalComponent from "$lib/components/stake/WithdrawalComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import * as AppRuntime from "$lib/runtime"
import { calculateIncentive } from "$lib/services/incentive"
import { balancesStore as BalanceStore } from "$lib/stores/balances.svelte"
import { chains as ChainStore } from "$lib/stores/chains.svelte"
import { tokensStore as TokenStore } from "$lib/stores/tokens.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { FetchHttpClient } from "@effect/platform"
import { Staking, Ucs05, Utils } from "@unionlabs/sdk"
import { EU_ERC20, EU_LST, U_ERC20 } from "@unionlabs/sdk/Constants"
import { Indexer } from "@unionlabs/sdk/Indexer"
import { TokenRawAmount, UniversalChainId } from "@unionlabs/sdk/schema"
import { Bond, Unbond } from "@unionlabs/sdk/schema/stake"
import { BigDecimal, Brand, ConfigProvider, DateTime, Effect, Layer, Order, pipe } from "effect"
import * as A from "effect/Array"
import { constVoid } from "effect/Function"
import * as O from "effect/Option"
import { onMount } from "svelte"

type StakeTab = "bond" | "unbond" | "withdraw"
type TableFilter = "all" | "bond" | "unbond"

const EVM_UNIVERSAL_CHAIN_ID = UniversalChainId.make("ethereum.11155111")

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

$inspect(data)
</script>

<Sections>
  <div class="hidden md:grid grid-cols-3 gap-4">
    <Card class="p-4">
      <div class="flex flex-col gap-2">
        <div class="text-sm text-zinc-400 uppercase tracking-wide">U balance</div>
        <div class="flex items-center gap-2">
          {#if O.isNone(WalletStore.evmAddress)}
            <div class="text-xl">—</div>
          {:else if O.isSome(evmChain) && O.isSome(uOnEvmToken) && O.isSome(uOnEvmBalance)}
            <div class="text-xl font-semibold">
              <TokenComponent
                chain={evmChain.value}
                denom={uOnEvmToken.value.denom}
                amount={TokenRawAmount.make(uOnEvmBalance.value)}
                showWrapping={false}
                showSymbol={true}
                showIcon={true}
              />
            </div>
          {:else}
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 bg-zinc-700/50 rounded-full animate-pulse"></div>
              <div class="w-20 h-6 bg-zinc-700/50 rounded animate-pulse"></div>
            </div>
          {/if}
        </div>
      </div>
    </Card>

    <Card class="p-4">
      <div class="flex flex-col gap-2">
        <div class="text-sm text-zinc-400 uppercase tracking-wide">eU balance</div>
        <div class="flex items-center gap-2">
          {#if O.isNone(WalletStore.evmAddress)}
            <div class="text-xl">—</div>
          {:else if O.isSome(evmChain) && O.isSome(eUOnEvmToken) && O.isSome(eUOnEvmBalance)}
            <div class="text-xl font-semibold">
              <TokenComponent
                chain={evmChain.value}
                denom={eUOnEvmToken.value.denom}
                amount={TokenRawAmount.make(eUOnEvmBalance.value)}
                showWrapping={false}
                showSymbol={true}
                showIcon={true}
              />
            </div>
          {:else}
            <div class="flex items-center gap-2">
              <div class="w-5 h-5 bg-zinc-700/50 rounded-full animate-pulse"></div>
              <div class="w-20 h-6 bg-zinc-700/50 rounded animate-pulse"></div>
            </div>
          {/if}
        </div>
      </div>
    </Card>

    <Card class="p-4">
      <div class="flex flex-col gap-2">
        <div class="text-sm text-zinc-400 uppercase tracking-wide">Staking Rewards</div>
        <div class="flex items-center gap-2">
          {#if O.isSome(incentives.current)
              && incentives.current.value._tag === "Success"}
            {@const formattedIncentives = pipe(
              incentives.current.value.value.rates.yearly,
              BigDecimal.multiply(BigDecimal.fromBigInt(100n)),
              BigDecimal.round({ mode: "from-zero", scale: 2 }),
              Utils.formatBigDecimal,
            )}
            <div class="text-xl text-accent">
              {formattedIncentives}%
            </div>
          {:else}
            <div class="w-16 h-5 bg-zinc-700/50 rounded animate-pulse"></div>
          {/if}
        </div>
      </div>
    </Card>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <Card divided>
      <div class="p-4 border-b border-zinc-800">
        <Tabs
          items={[
            { id: "bond", label: "Stake" },
            { id: "unbond", label: "Unstake" },
            { id: "withdraw", label: "Withdraw" },
          ]}
          activeId={selectedTab}
          onTabChange={(id) => selectedTab = id as StakeTab}
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

    <IncentiveCard
      incentives={incentives.current}
      stakeAmount={bondAmount}
      {evmChain}
      {uOnEvmToken}
    />
  </div>

  <StakingHistoryCard
    data={data.current}
    walletConnected={O.isSome(WalletStore.evmAddress)}
  />
</Sections>

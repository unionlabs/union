<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import StakingListItemComponent from "$lib/components/model/StakingListItemComponent.svelte"
import NoWalletConnected from "$lib/components/NoWalletConnected.svelte"
import BondComponent from "$lib/components/stake/BondComponent.svelte"
import UnbondComponent from "$lib/components/stake/UnbondComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import * as AppRuntime from "$lib/runtime"
import { balancesStore as BalanceStore } from "$lib/stores/balances.svelte"
import { chains as ChainStore } from "$lib/stores/chains.svelte"
import { tokensStore as TokenStore } from "$lib/stores/tokens.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { cn } from "$lib/utils"
import { matchOption, matchRuntimeResult } from "$lib/utils/snippets.svelte"
import { Staking, Ucs05 } from "@unionlabs/sdk"
import { EU_ERC20, EU_LST, U_ERC20 } from "@unionlabs/sdk/Constants"
import { Indexer } from "@unionlabs/sdk/Indexer"
import { Chain, TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"
import { Bond, Unbond } from "@unionlabs/sdk/schema/stake"
import * as Utils from "@unionlabs/sdk/Utils"
import { Brand, ConfigProvider, DateTime, Effect, Layer, Order, pipe } from "effect"
import * as A from "effect/Array"
import * as E from "effect/Either"
import { constVoid, flow } from "effect/Function"
import * as O from "effect/Option"
import * as S from "effect/Schema"
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
let tableFilter = $state<TableFilter>("all")
let currentPage = $state<number>(1)
let refreshTrigger = $state<number>(0)

const itemsPerPage = 10

const refreshBondData = () => {
  refreshTrigger = Date.now()
  currentPage = 1
}

$effect(() => {
  void tableFilter
  currentPage = 1
})

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

{#snippet renderChain(chain: Chain, denom: TokenRawDenom)}
  <ChainComponent
    chain={chain}
    withToken={denom}
  />
{/snippet}

{#snippet renderStatus(bond: Bond | Unbond)}
  {#if bond.status === "success"}
    <span
      class="px-1.5 py-0.5 text-xs font-mono font-medium rounded-sm bg-emerald-500/10 text-emerald-400 ring-1 ring-emerald-500/30"
    >
      success
    </span>
  {/if}
  {#if bond.status === "failure"}
    <span
      class="px-1.5 py-0.5 text-xs font-mono font-medium rounded-sm bg-rose-500/10 text-rose-400 ring-1 ring-rose-500/30"
    >
      failure
    </span>
  {/if}
  {#if bond.status === "pending"}
    <span
      class="px-1.5 py-0.5 text-xs font-mono font-medium rounded-sm bg-yellow-500/10 text-yellow-400 ring-1 ring-yellow-500/30"
    >
      pending
    </span>
  {/if}
{/snippet}

{#snippet maybeRenderBonds(maybeBonds: O.Option<A.NonEmptyReadonlyArray<Bond | Unbond>>)}
  {#snippet noBonds()}
    <div class="flex items-center justify-center rounded-lg border border-dashed border-zinc-700/80 bg-zinc-950/40 text-zinc-400 text-sm h-28">
      No {
        tableFilter === "all"
        ? "bonds"
        : tableFilter === "bond"
        ? "stake transactions"
        : "unstake transactions"
      } yet
    </div>
  {/snippet}
  {#snippet hasBonds(bonds: A.NonEmptyReadonlyArray<Bond | Unbond>)}
    {@const filteredBonds = bonds.filter(bond =>
    tableFilter === "all"
    || (tableFilter === "bond" && bond._tag === "Bond")
    || (tableFilter === "unbond" && bond._tag === "Unbond")
  )}
    {@const totalItems = filteredBonds.length}
    {@const totalPages = Math.ceil(totalItems / itemsPerPage)}
    {@const startIndex = (currentPage - 1) * itemsPerPage}
    {@const endIndex = startIndex + itemsPerPage}
    {@const paginatedBonds = filteredBonds.slice(startIndex, endIndex)}

    <!-- Table Filter Controls -->
    <div class="pt-2 px-2 pb-2 border-b border-zinc-800">
      <div class="flex items-center justify-between gap-1 sm:gap-2">
        <Tabs
          items={[
            { id: "all", label: "All" },
            { id: "bond", label: "Stakes" },
            { id: "unbond", label: "Unstakes" },
          ]}
          activeId={tableFilter}
          onTabChange={(id) => tableFilter = id as TableFilter}
        />

        <!-- Pagination Controls -->
        {#if totalPages > 1}
          <div class="flex gap-0.5 sm:gap-1">
            <button
              onclick={() => currentPage = Math.max(1, currentPage - 1)}
              disabled={currentPage <= 1}
              class="px-1 sm:px-2 py-1 text-xs sm:text-sm font-medium rounded transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed text-zinc-500 hover:text-zinc-300 w-5 sm:w-8 bg-zinc-900 hover:bg-zinc-800"
            >
              ←
            </button>
            <div class="px-1.5 sm:px-3 py-1 text-xs sm:text-sm font-medium rounded text-white bg-zinc-800 min-w-[1.5rem] sm:min-w-[3rem] text-center">
              {currentPage}/{totalPages}
            </div>
            <button
              onclick={() => currentPage = Math.min(totalPages, currentPage + 1)}
              disabled={currentPage >= totalPages}
              class="px-1 sm:px-2 py-1 text-xs sm:text-sm font-medium rounded transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed text-zinc-500 hover:text-zinc-300 w-5 sm:w-8 bg-zinc-900 hover:bg-zinc-800"
            >
              →
            </button>
          </div>
        {/if}
      </div>
    </div>

    <div class="overflow-auto">
      {#each paginatedBonds as item}
        <StakingListItemComponent {item} />
      {:else}
        <div class="p-4 text-center text-zinc-500">No transactions found</div>
      {/each}
    </div>
  {/snippet}

  {@render matchOption(maybeBonds, hasBonds, noBonds)}
{/snippet}

{#snippet renderSkeleton()}
  <div class="relative overflow-auto max-h-72 rounded-lg ring-1 ring-zinc-800/80 animate-pulse">
    <table class="w-full text-sm">
      <thead class="sticky top-0 z-10 bg-zinc-950/90">
        <tr class="text-zinc-400 border-b border-zinc-800/80">
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">arrow</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">Type</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">Chain</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">
            Timestamp
          </th>
          <th class="px-3 py-2 text-right font-semibold tracking-wide text-xs uppercase">Amount</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">Status</th>
        </tr>
      </thead>
      <tbody>
        {#each Array(10) as _}
          <tr class="even:bg-zinc-900/30 odd:bg-zinc-900/10">
            <td class="px-3 py-2"><div class="h-4 w-24 bg-zinc-700/50 rounded"></div></td>
            <td class="px-3 py-2"><div class="h-4 w-32 bg-zinc-700/50 rounded"></div></td>
            <td class="px-3 py-2 text-right">
              <div class="h-4 w-16 bg-zinc-700/50 rounded ml-auto"></div>
            </td>
            <td class="px-3 py-2"><div class="h-4 w-14 bg-zinc-700/50 rounded"></div></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/snippet}

{#snippet renderError(error: any)}
  <pre class="text-red-500 overflow-auto">{JSON.stringify(error, null, 2)}</pre>
{/snippet}

{#snippet whenWallet()}
  <div class="flex flex-col gap-6">
    <!-- Bond / Unbond Tables -->
    <div class="grid grid-cols-1 gap-6">
      <section class="flex flex-col">
        {@render matchRuntimeResult(data.current, {
          onSuccess: maybeRenderBonds,
          onFailure: renderError,
          onNone: renderSkeleton,
        })}
      </section>
    </div>
  </div>
{/snippet}

{#snippet noWallet()}
  <NoWalletConnected title="No EVM Wallet Connected" />
{/snippet}

<Sections>
  <Card
    class="p-0 font-mono"
    divided
  >
    <!-- Controls -->
    <div class="pt-2 px-2 pb-2 border-b border-zinc-800">
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

    <!-- Content -->
    <div class="px-3 pb-3">
      {#if selectedTab === "bond"}
        <BondComponent
          {evmChain}
          {uOnEvmToken}
          {uOnEvmBalance}
          onBondSuccess={refreshBondData}
        />
      {:else if selectedTab === "unbond"}
        <UnbondComponent
          {evmChain}
          {uOnEvmToken}
          {eUOnEvmBalance}
          onUnbondSuccess={refreshBondData}
        />
      {:else if selectedTab === "withdraw"}
        <div class="flex flex-col gap-4 text-center py-8">
          <div class="text-zinc-400 text-sm">Withdrawal functionality</div>
          <div class="text-zinc-500 text-xs">
            Query withdrawable balance and implement withdrawal logic
          </div>
        </div>
      {/if}
    </div>
  </Card>

  <!-- Staking History Card -->
  <Card
    class="p-0 font-mono"
    divided
  >
    {#if O.isSome(WalletStore.evmAddress)}
      {@render whenWallet()}
    {:else}
      <div class="flex items-center justify-center text-zinc-400 text-xs font-mono h-32">
        wallet disconnected - connect to view history
      </div>
    {/if}
  </Card>
</Sections>

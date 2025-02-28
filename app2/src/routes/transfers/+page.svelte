<script lang="ts">
import {
  transferListLatestAddressQuery,
  transferListPageGtAddressQuery,
  transferListPageLtAddressQuery
} from "$lib/queries/transfer-list-address.svelte"
import { DateTime, Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"
import { transferListAddress } from "$lib/stores/transfers.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import NoWalletConnected from "$lib/components/NoWalletConnected.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import TransferListItemComponent from "$lib/components/model/TransferListItemComponent.svelte"
import TransferListItemComponentSkeleton from "$lib/components/model/TransferListItemComponentSkeleton.svelte"

let fiber: Fiber.Fiber<any, any>
let fiberLock = false

$effect(() => {
  if (Option.isSome(wallets.evmAddress)) {
    console.log("will fetch")
    fetchLive()
  }
})

const fetchLive = async () => {
  if (fiberLock) return
  fiberLock = true
  if (fiber) {
    await Effect.runPromise(Fiber.interrupt(fiber))
  }
  if (Option.isSome(wallets.evmAddress)) {
    fiber = Effect.runFork(transferListLatestAddressQuery(wallets.evmAddress.value, settingsStore.pageLimit))
  }
  fiberLock = false
}

onMount(() => {
  return () => Effect.runPromise(Fiber.interrupt(fiber))
})

const onLive = async () => {
  if (Option.isSome(transferListAddress.data) && Option.isSome(wallets.evmAddress)) {
    transferListAddress.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListLatestAddressQuery(wallets.evmAddress.value, settingsStore.pageLimit))
  }
}

const onPrevPage = async () => {
  if (Option.isSome(transferListAddress.data) && Option.isSome(wallets.evmAddress)) {
    let firstSortOrder = transferListAddress.data.value.at(0)?.sort_order
    if (!firstSortOrder) return
    transferListAddress.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(
      transferListPageGtAddressQuery(firstSortOrder, wallets.evmAddress.value, settingsStore.pageLimit)
    )
  }
}

const onNextPage = async () => {
  if (Option.isSome(transferListAddress.data) && Option.isSome(wallets.evmAddress)) {
    let lastSortOrder = transferListAddress.data.value.at(-1)?.sort_order
    if (!lastSortOrder) return
    transferListAddress.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(
      transferListPageLtAddressQuery(lastSortOrder, wallets.evmAddress.value, settingsStore.pageLimit)
    )
  }
}
</script>

<Sections>
  <section>
  <h1 class="font-bold text-4xl">Your Transfers</h1>
  <p>These are the transfers from your connected wallets</p>
  </section>
  <Card class="overflow-auto" divided>
    {#if Option.isNone(wallets.evmAddress)}
      <NoWalletConnected/>
    {:else if Option.isSome(transferListAddress.data) && Option.isSome(chains.data)}
      {@const chainss = chains.data.value}
      {#if Option.isSome(transferListAddress.error)}
        <ErrorComponent error={transferListAddress.error.value}/>
      {/if}
      {#each transferListAddress.data.value as transfer(transfer.sort_order)}
        <TransferListItemComponent {transfer} />
      {/each}
    {:else}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <TransferListItemComponentSkeleton />
      {/each}
      {#if Option.isSome(transferListAddress.error)}
        <ErrorComponent error={transferListAddress.error.value}/>
      {/if}
    {/if}
  </Card>
  <div class="flex gap-6">
    <button onclick={onLive} class="cursor-pointer border-rounded dark:bg-sky-600 border-sky-500 border h-10 w-20 rounded font-bold">
      LIVE
    </button>
    <div class="rounded shadow flex">
      <button onclick={onPrevPage} class="cursor-pointer border-l border-t border-b bg-zinc-700 border-zinc-600 h-10 w-10 rounded-tl rounded-bl">
        ←
      </button>
      <div class="bg-zinc-900 border-t border-b border-zinc-800 flex items-center justify-center px-4 min-w-[250px]">
        {#if Option.isSome(transferListAddress.data) && transferListAddress.data.value.length > 0}
          {DateTime.formatIso(transferListAddress.data.value[0].packet_send_timestamp)}
        {/if}
      </div>
      <button onclick={onNextPage} class="cursor-pointer border-r border-t border-b bg-zinc-700 border-zinc-600 h-10 w-10 rounded-tr rounded-br">
        →
      </button>
    </div>
  </div>
</Sections>

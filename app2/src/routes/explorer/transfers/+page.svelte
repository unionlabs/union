<script lang="ts">
import {
  transferListLatestQuery,
  transferListPageGtQuery,
  transferListPageLtQuery
} from "$lib/queries/transfer-list.svelte"
import { DateTime, Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"
import { transferList } from "$lib/stores/transfers.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { flip } from "svelte/animate"
import { fly } from "svelte/transition"
import Button from "$lib/components/ui/Button.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

import { settingsStore } from "$lib/stores/settings.svelte"
import TransferListItemComponent from "$lib/components/model/TransferListItemComponent.svelte"
import TransferListItemComponentSkeleton from "$lib/components/model/TransferListItemComponentSkeleton.svelte"

let fiber: Fiber.Fiber<any, any>

onMount(() => {
  fiber = Effect.runFork(transferListLatestQuery(settingsStore.pageLimit))
  return () => Effect.runPromise(Fiber.interrupt(fiber))
})

const onLive = async () => {
  if (Option.isSome(transferList.data)) {
    transferList.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListLatestQuery(settingsStore.pageLimit))
  }
}

const onPrevPage = async () => {
  if (Option.isSome(transferList.data)) {
    let firstSortOrder = transferList.data.value.at(0)?.sort_order
    if (!firstSortOrder) return
    transferList.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListPageGtQuery(firstSortOrder, settingsStore.pageLimit))
  }
}

const onNextPage = async () => {
  if (Option.isSome(transferList.data)) {
    let lastSortOrder = transferList.data.value.at(-1)?.sort_order
    if (!lastSortOrder) return
    transferList.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListPageLtQuery(lastSortOrder, settingsStore.pageLimit))
  }
}
</script>

<Sections>
  <h1 class="font-bold text-4xl">Transfers</h1>
  <Card class="overflow-auto" divided>
    {#if Option.isSome(transferList.data) && Option.isSome(chains.data)}
      {@const chainss = chains.data.value}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
      {#each transferList.data.value as transfer(transfer.sort_order)}
        <TransferListItemComponent {transfer} />
      {/each}
    {:else}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <TransferListItemComponentSkeleton />
      {/each}
    {/if}
  </Card>
  <div class="flex gap-6">
    <Button class="bg-red-500" onclick={onLive}>
      LIVE
    </Button>
    <div class="rounded shadow flex">
      <button onclick={onPrevPage} class="cursor-pointer border-l border-t border-b bg-zinc-700 border-zinc-600 h-10 w-10 rounded-tl rounded-bl">
        ←
      </button>
      <div class="bg-zinc-900 border-t border-b border-zinc-800 flex items-center justify-center px-4 min-w-[250px]">
        {#if Option.isSome(transferList.data) && transferList.data.value.length > 0}
          {DateTime.formatIso(transferList.data.value[0].packet_send_timestamp)}
        {/if}
      </div>
      <button onclick={onNextPage} class="cursor-pointer border-r border-t border-b bg-zinc-700 border-zinc-600 h-10 w-10 rounded-tr rounded-br">
        →
      </button>
    </div>
  </div>
</Sections>

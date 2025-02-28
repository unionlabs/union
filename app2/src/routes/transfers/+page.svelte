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
import { wallets } from "$lib/stores/wallets.svelte"
import NoWalletConnected from "$lib/components/NoWalletConnected.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import TransferListItemComponent from "$lib/components/model/TransferListItemComponent.svelte"
import TransferListItemComponentSkeleton from "$lib/components/model/TransferListItemComponentSkeleton.svelte"
import TransferListPagination from "$lib/components/ui/TransferListPagination.svelte"

let fiber: Fiber.Fiber<any, any>
let fiberLock = false

$effect(() => {
  if (wallets.hasAnyWallet()) {
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
  const addresses = wallets.getCanonicalByteAddressList()
  if (addresses.length > 0) {
    fiber = Effect.runFork(transferListLatestAddressQuery(addresses, settingsStore.pageLimit))
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
    const addresses = wallets.getCanonicalByteAddressList()
    fiber = Effect.runFork(transferListLatestAddressQuery(addresses, settingsStore.pageLimit))
  }
}

const onPrevPage = async () => {
  if (Option.isSome(transferListAddress.data)) {
    let firstSortOrder = transferListAddress.data.value.at(0)?.sort_order
    if (!firstSortOrder) return
    const addresses = wallets.getCanonicalByteAddressList()
    if (addresses.length === 0) return
    transferListAddress.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(
      transferListPageGtAddressQuery(firstSortOrder, addresses, settingsStore.pageLimit)
    )
  }
}

const onNextPage = async () => {
  if (Option.isSome(transferListAddress.data)) {
    let lastSortOrder = transferListAddress.data.value.at(-1)?.sort_order
    if (!lastSortOrder) return
    const addresses = wallets.getCanonicalByteAddressList()
    if (addresses.length === 0) return
    transferListAddress.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(
      transferListPageLtAddressQuery(lastSortOrder, addresses, settingsStore.pageLimit)
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
    {#if Option.isSome(transferListAddress.error)}
      <ErrorComponent error={transferListAddress.error.value}/>
    {/if}
    {#if wallets.getCanonicalByteAddressList().length === 0}
      <NoWalletConnected/>
    {:else if Option.isSome(transferListAddress.data) && Option.isSome(chains.data)}
      {#each transferListAddress.data.value as transfer(transfer.sort_order)}
        <TransferListItemComponent {transfer} />
      {/each}
    {:else}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <TransferListItemComponentSkeleton />
      {/each}
    {/if}
  </Card>
  <TransferListPagination 
    data={transferListAddress.data}
    {onLive}
    {onPrevPage}
    {onNextPage}
  />
</Sections>

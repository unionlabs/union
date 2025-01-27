<script lang="ts">
import DebugBox from "$lib/components/TransferFrom/components/DebugBox/index.svelte"
import { TRANSFER_DEBUG } from "$lib/components/TransferFrom/transfer/config.ts"
import { createTransferStore } from "$lib/components/TransferFrom/transfer"
import Intent from "$lib/components/TransferFrom/components/Cube/faces/Intent.svelte"
import Chains from "$lib/components/TransferFrom/components/Cube/faces/Chains.svelte"
import Assets from "$lib/components/TransferFrom/components/Cube/faces/Assets.svelte"
import Transfer from "$lib/components/TransferFrom/components/Cube/faces/Transfer.svelte"
import Cube from "$lib/components/TransferFrom/components/Cube/index.svelte"
import type { Chain, Ucs03Channel } from "$lib/types.ts"
import { userBalancesQuery } from "$lib/queries/balance"
import { userAddress, balanceStore } from "$lib/components/TransferFrom/transfer/balances.ts"
import { createRawIntentsStore } from "./transfer/raw-intents.ts"
import { derived, writable, type Writable } from "svelte/store"
import {
  bech32AddressToHex,
  getChannelInfo,
  getQuoteToken,
  isValidBech32Address
} from "@unionlabs/client"
import { fromHex, isHex, toHex } from "viem"
import { subscribe } from "graphql"

export let chains: Array<Chain>
export let ucs03channels: Array<Ucs03Channel>

let balances = userBalancesQuery({ chains, userAddr: $userAddress })
const rawIntents = createRawIntentsStore()
const stores = createTransferStore(chains, balances)

let channel = derived(rawIntents, $rawIntents => {
  if (!($rawIntents.source && $rawIntents.destination)) return null
  return getChannelInfo($rawIntents.source, $rawIntents.destination, ucs03channels)
})

let transferArgs: Writable<
  | {
      baseToken: string
      baseAmount: bigint
      quoteToken: string
      quoteAmount: bigint
      receiver: string
      sourceChannelId: number
      ucs03address: string
    }
  | "NO_QUOTE_AVAILABLE"
  | null
> = writable(null)

rawIntents.subscribe(async () => {
  transferArgs.set(null)
  if ($channel === null || $rawIntents.asset === null) return null
  const chain = chains.find(c => c.chain_id === $rawIntents.source)
  if (!chain) return null

  const destChain = chains.find(c => c.chain_id === $rawIntents.destination)
  if (!destChain) return null

  // decode from hex if cosmos to assert proper quote token prediction.
  let baseToken =
    chain.rpc_type === "cosmos" && isHex($rawIntents.asset)
      ? fromHex($rawIntents.asset, "string")
      : $rawIntents.asset

  console.log({ baseToken })

  const quoteToken = await getQuoteToken($rawIntents.source, baseToken, $channel)

  if (quoteToken.isErr()) {
    transferArgs.set(null)
    return
  }

  if (quoteToken.value.type === "NO_QUOTE_AVAILABLE") {
    transferArgs.set("NO_QUOTE_AVAILABLE")
    return
  }

  const receiver =
    destChain.rpc_type === "cosmos" && isValidBech32Address($rawIntents.receiver)
      ? bech32AddressToHex({ address: $rawIntents.receiver })
      : $rawIntents.receiver

  let ucs03address =
    chain.rpc_type === "cosmos"
      ? fromHex(`0x${$channel.source_port_id}`, "string")
      : `0x${$channel.source_port_id}`

  transferArgs.set({
    baseToken,
    baseAmount: BigInt($rawIntents.amount),
    quoteToken: quoteToken.value.quote_token,
    quoteAmount: BigInt($rawIntents.amount),
    receiver,
    sourceChannelId: $channel.source_channel_id,
    ucs03address
  })
})
</script>

<Cube>
  <div slot="intent" let:rotateTo class="w-full h-full">
    <Intent {chains} {channel} transferArgs={$transferArgs} {stores} {rotateTo}/>
  </div>

  <div slot="source" let:rotateTo class="w-full h-full">
    <Chains {stores} {rotateTo} selected="source"/>
  </div>

  <div slot="destination" let:rotateTo class="w-full h-full">
    <Chains {stores} {rotateTo} selected="destination"/>
  </div>

  <div slot="assets" let:rotateTo class="w-full h-full">
    <Assets {stores} {chains} {rotateTo}/>
  </div>

  <div slot="transfer" let:rotateTo class="w-full h-full">
    {#if $transferArgs && $channel}
      <Transfer channel={$channel} transferArgs={$transferArgs} {chains}/>
    {/if}
  </div>
</Cube>

<div class="absolute bottom-0 inset-x-0 text-center py-2">
  {#if TRANSFER_DEBUG}
    <DebugBox {stores}/>
  {/if}
</div>

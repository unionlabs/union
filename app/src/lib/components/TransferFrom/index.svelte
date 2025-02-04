<script lang="ts">
import DebugBox from "$lib/components/TransferFrom/components/DebugBox/index.svelte"
import { TRANSFER_DEBUG } from "$lib/components/TransferFrom/transfer/config.ts"
import { createTransferStores } from "$lib/components/TransferFrom/transfer"
import Intent from "$lib/components/TransferFrom/components/Cube/faces/Intent.svelte"
import Chains from "$lib/components/TransferFrom/components/Cube/faces/Chains.svelte"
import Assets from "$lib/components/TransferFrom/components/Cube/faces/Assets.svelte"
import Transfer from "$lib/components/TransferFrom/components/Cube/faces/Transfer.svelte"
import Cube from "$lib/components/TransferFrom/components/Cube/index.svelte"
import type { Chain, Ucs03Channel } from "$lib/types.ts"
import { userBalancesQuery } from "$lib/queries/balance"
import { balanceStore, userAddress } from "$lib/components/TransferFrom/transfer/balances.ts"
import { writable, type Writable } from "svelte/store"
import { getQuoteToken } from "@unionlabs/client"
import type { TransferArgs, TransferContext } from "$lib/components/TransferFrom/transfer/types.ts"
import { debouncePromise } from "$lib/utilities"

export let chains: Array<Chain>
export let ucs03channels: Array<Ucs03Channel>

// This is kinda ugly, but necessary to get tanstack reactivity in our .ts intent/validation funnel.
// This keeps the query reactive in svelte land. We then update and pass a writable into createTransferStores.
$: {
  const query = userBalancesQuery({ chains, userAddr: $userAddress })
  query.subscribe($balances => {
    balanceStore.set($balances)
  })
}
const stores = createTransferStores(chains, userAddress, balanceStore, ucs03channels)

const { rawIntents, validation } = stores

const transferArgs: Writable<TransferArgs | null> = writable(null)
const transferContext: Writable<TransferContext | null> = writable(null)

const debouncedGetQuoteToken = debouncePromise(getQuoteToken, 500)

validation.subscribe(async data => {
  if (
    !(
      data.transfer?.destinationChain &&
      data.transfer?.sourceChain &&
      data.transfer?.baseToken &&
      data.transfer?.channel
    )
  ) {
    transferArgs.set(null)
    transferContext.set(null)
    return
  }

  transferContext.set({
    channel: data.transfer.channel,
    sourceChain: data.transfer.sourceChain,
    destinationChain: data.transfer.destinationChain
  })

  const quoteToken = await debouncedGetQuoteToken(
    data.transfer.sourceChain.chain_id,
    data.transfer.baseToken.denom,
    data.transfer.channel
  )

  if (quoteToken.isErr()) {
    transferArgs.set(null)
    return
  }

  if (quoteToken.value.type === "NO_QUOTE_AVAILABLE") {
    transferArgs.set("NO_QUOTE_AVAILABLE")
    return
  }

  transferArgs.set({
    baseToken: data.transfer.baseToken.denom,
    baseAmount: data.transfer.parsedAmount,
    quoteToken: quoteToken.value.quote_token,
    quoteAmount: data.transfer.parsedAmount,
    receiver: data.transfer.receiver,
    sourceChannelId: data.transfer.channel.source_channel_id,
    ucs03address: data.transfer.ucs03address
  })
})
</script>

<Cube>
  <div slot="intent" let:rotateTo class="w-full h-full">
    <Intent transferArgs={$transferArgs} {stores} {rotateTo}/>
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
    {#if $transferArgs && $transferContext}
      <Transfer transferArgs={$transferArgs} transferContext={$transferContext} {chains}
      />
    {/if}
  </div>
</Cube>

<div class="absolute bottom-0 inset-x-0 text-center py-2">
  {#if TRANSFER_DEBUG}
    <DebugBox {stores}/>
  {/if}
</div>

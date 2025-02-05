<script lang="ts">
import Intent from "$lib/components/TransferFrom/components/Cube/faces/Intent.svelte"
import Chains from "$lib/components/TransferFrom/components/Cube/faces/Chains.svelte"
import Assets from "$lib/components/TransferFrom/components/Cube/faces/Assets.svelte"
import Transfer from "$lib/components/TransferFrom/components/Cube/faces/Transfer.svelte"
import Cube from "$lib/components/TransferFrom/components/Cube/index.svelte"
import type { Chain, Ucs03Channel } from "$lib/types.ts"
import { derived } from "svelte/store"
import { getChannelInfo, getQuoteToken } from "@unionlabs/client"
import { createRawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { checkValidation } from "$lib/components/TransferFrom/transfer/validation.ts"
import { createIntents } from "$lib/components/TransferFrom/transfer/intents.ts"
import { balances } from "$lib/stores/balances.ts"
import { tokenInfos } from "$lib/stores/tokens.ts"

export let chains: Array<Chain>
export let ucs03channels: Array<Ucs03Channel>

const rawIntents = createRawIntentsStore()

const userAddress = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$userAddrCosmos, $userAddrEvm, $userAddressAptos]) => ({
    evm: $userAddrEvm,
    cosmos: $userAddrCosmos,
    aptos: $userAddressAptos
  })
)

const quoteToken = derived(
  [
    derived(rawIntents, $r => $r.source),
    derived(rawIntents, $r => $r.asset),
    derived(rawIntents, $r => $r.destination)
  ],
  ([$source, $asset, $destination], set) => {
    set(null)

    if (!($source && $asset && ucs03channels)) {
      return
    }

    const channel = getChannelInfo($source, $destination, ucs03channels)
    if (!channel) {
      return
    }

    getQuoteToken($source, $asset, channel).then(quote => set(quote))
  },
  null
)

const transfer = derived(
  [rawIntents, balances, userAddress, tokenInfos, quoteToken],
  ([$rawIntents, $balances, $userAddress, $tokenInfos, $quoteToken]) => {
    const intents = createIntents(
      $rawIntents,
      $balances,
      $userAddress,
      chains,
      ucs03channels,
      $tokenInfos,
      $quoteToken
    )
    const validation = checkValidation($rawIntents, intents, $balances, $userAddress)
    return { intents, validation }
  }
)

$: console.log($tokenInfos)
</script>

<Cube>
  <div slot="intent" let:rotateTo class="w-full h-full">
    <Intent
            rawIntents={rawIntents}
            intents={$transfer.intents}
            validation={$transfer.validation}
            chains={chains}
            {rotateTo}
    />
  </div>

  <div slot="source" let:rotateTo class="w-full h-full">
    <Chains {rawIntents} {chains} {rotateTo} selected="source"/>
  </div>

  <div slot="destination" let:rotateTo class="w-full h-full">
    <Chains {rawIntents} {chains} {rotateTo} selected="destination"/>
  </div>

  <div slot="assets" let:rotateTo class="w-full h-full">
    <Assets {rawIntents} intents={$transfer.intents} {chains} {rotateTo}/>
  </div>

  <div slot="transfer" let:rotateTo class="w-full h-full">
    {#if $transfer.validation.args && $transfer.validation.context}
      <Transfer
              transferArgs={$transfer.validation.args}
              transferContext={$transfer.validation.context}
              {chains}
      />
    {/if}
  </div>
</Cube>

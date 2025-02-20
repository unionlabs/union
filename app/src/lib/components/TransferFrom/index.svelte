<script lang="ts">
import Intent from "$lib/components/TransferFrom/components/Cube/faces/Intent.svelte"
import Chains from "$lib/components/TransferFrom/components/Cube/faces/Chains.svelte"
import Assets from "$lib/components/TransferFrom/components/Cube/faces/Assets.svelte"
import Transfer from "$lib/components/TransferFrom/components/Cube/faces/Transfer.svelte"
import Cube from "$lib/components/TransferFrom/components/Cube/index.svelte"
import type { Chain, Ucs03Channel } from "$lib/types.ts"
import { derived, get, type Readable } from "svelte/store"
import { getChannelInfo, getQuoteToken } from "@unionlabs/client"
import { createRawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { checkValidation } from "$lib/components/TransferFrom/transfer/validation.ts"
import { createIntents } from "$lib/components/TransferFrom/transfer/intents.ts"
import { balances } from "$lib/stores/balances.ts"
import { tokenInfos } from "$lib/stores/tokens.ts"
import { onMount } from "svelte"
import { queryBalances } from "$lib/stores/balances.ts"
import type {
  DerivedSource,
  Nullable,
  QuoteData
} from "$lib/components/TransferFrom/transfer/types.ts"
import { persisted } from "svelte-persisted-store"

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

const quoteResults = persisted<Record<string, QuoteData | null>>("quote-results", {})
const getCacheKey = ($source: string, $asset: string, $destination: string) =>
  `${$source}-${$asset}-${$destination}`

const quoteToken: Readable<Nullable<QuoteData>> = derived<
  [DerivedSource, DerivedSource, DerivedSource],
  Nullable<QuoteData>
>(
  [
    derived(rawIntents, $r => $r.source),
    derived(rawIntents, $r => $r.asset),
    derived(rawIntents, $r => $r.destination)
  ],
  ([$source, $asset, $destination], set) => {
    set(null)
    
    if (!($source && $asset && $destination && ucs03channels)) return

    const channel = getChannelInfo($source, $destination, ucs03channels)
    if (!channel) return

    const sourceChain = chains.find(c => c.chain_id === $source)
    if (!sourceChain) {
      console.error("Invalid source chain in quote token calculation")
      return
    }

    const cacheKey = `${$source}-${$asset}-${$destination}`
    const cachedResult = get(quoteResults)[cacheKey] ?? null

    if (cachedResult) {
      set(cachedResult)
      return
    }

    getQuoteToken($source, $asset, channel)
      .then(result => {
        if (result.isOk()) {
          const quoteData = result.value
          quoteResults.update(store => ({ ...store, [cacheKey]: quoteData }))
          set(quoteData)
        } else {
          console.error("Error fetching quote token:", result.error)
          set(null)
        }
      })
      .catch(err => {
        console.error("Unexpected error in getQuoteToken:", err)
        set(null)
      })
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

// fetch balances for chain, useraddr pair on change
let previousSourceChain = ""
let previousUserAddr = ""
onMount(() => {
  const unsubscribe = transfer.subscribe(trans => {
    const chain = trans.intents.sourceChain
    if (chain) {
      const userAddr =
        chain.rpc_type === "evm"
          ? $userAddress.evm?.canonical.toLowerCase()
          : $userAddrCosmos?.canonical
      if (userAddr && (previousSourceChain !== chain.chain_id || previousUserAddr !== userAddr)) {
        previousUserAddr = userAddr
        previousSourceChain = chain.chain_id
        console.log(
          `[UserBalances] detected new pair ${chain.chain_id} ${userAddr}, fetching balances.`
        )
        queryBalances(chain, userAddr)
      }
    }
  })
  return unsubscribe
})
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

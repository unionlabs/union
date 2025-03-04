<script lang="ts">
import Intent from "$lib/components/TransferCube/components/Cube/faces/Intent.svelte"
import Chains from "$lib/components/TransferCube/components/Cube/faces/Chains.svelte"
import Assets from "$lib/components/TransferCube/components/Cube/faces/Assets.svelte"
import Transfer from "$lib/components/TransferCube/components/Cube/faces/Transfer.svelte"
import Cube from "$lib/components/TransferCube/components/Cube/index.svelte"
import type { Chain, Ucs03Channel } from "$lib/types.ts"
import { derived, get, type Readable } from "svelte/store"
import {
  type EvmChainId,
  getChannelInfo,
  getQuoteToken,
  getWethQuoteToken
} from "@unionlabs/client"
import { createRawIntentsStore } from "$lib/components/TransferCube/transfer/raw-intents.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { checkValidation } from "$lib/components/TransferCube/transfer/validation.ts"
import { createIntents } from "$lib/components/TransferCube/transfer/intents.ts"
import { balances } from "$lib/stores/balances.ts"
import { tokenInfos } from "$lib/stores/tokens.ts"
import { onMount } from "svelte"
import { queryBalances } from "$lib/stores/balances.ts"
import type {
  DerivedSource,
  Nullable,
  QuoteData
} from "$lib/components/TransferCube/transfer/types.ts"
import { persisted } from "svelte-persisted-store"
import { fromHex, type Hex } from "viem"

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
    console.log("roll")
    set(null)

    if (!($source && $asset && $destination && ucs03channels)) return

    const channel = getChannelInfo($source, $destination, ucs03channels)
    if (!channel) return

    const sourceChain = chains.find(c => c.chain_id === $source)
    if (!sourceChain) {
      console.error("Invalid source chain in quote token calculation")
      return
    }

    const cacheKey = `${$source}-${JSON.stringify(channel)}-${$asset}-v1`
    const cachedResult = get(quoteResults)[cacheKey] ?? null

    if (cachedResult) {
      set(cachedResult)
      return
    }

    set({ type: "QUOTE_LOADING" })

    //@ts-ignore
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

const wethQuoteToken: Readable<Nullable<{ wethAddress: Hex }>> = derived<
  [DerivedSource, DerivedSource],
  Nullable<{ wethAddress: Hex }>
>(
  [derived(rawIntents, $r => $r.source), derived(rawIntents, $r => $r.destination)],
  ([$source, $destination], set) => {
    set(null)

    if (!($source && $destination && ucs03channels)) return

    const channel = getChannelInfo($source, $destination, ucs03channels)
    if (!channel) return

    const sourceChain = chains.find(c => c.chain_id === $source)
    if (!sourceChain) {
      console.error("Invalid source chain in WETH quote token calculation")
      return
    }

    if (sourceChain.rpc_type !== "evm") {
      set(null)
      return
    }

    const ucs03address: `0x${string}` | null = channel.source_port_id
      ? `0x${channel.source_port_id}`
      : null

    if (!ucs03address) {
      console.error("UCS03 address not found for chain:", sourceChain.chain_id)
      return
    }

    getWethQuoteToken(sourceChain.chain_id, ucs03address, channel)
      .then(result => {
        if (result.isOk()) {
          const response = result.value
          if ("wethQuoteToken" in response) {
            set({ wethAddress: response.wethQuoteToken as Hex })
          } else {
            set(null)
          }
        } else {
          console.error("Error fetching WETH address:", result.error)
          set(null)
        }
      })
      .catch(err => {
        console.error("Unexpected error in getWethAddress:", err)
        set(null)
      })
  },
  null
)

const transfer = derived(
  [rawIntents, balances, userAddress, tokenInfos, quoteToken, wethQuoteToken],
  ([$rawIntents, $balances, $userAddress, $tokenInfos, $quoteToken, $wethQuoteToken]) => {
    const intents = createIntents(
      $rawIntents,
      $balances,
      $userAddress,
      chains,
      ucs03channels,
      $tokenInfos,
      $quoteToken,
      $wethQuoteToken
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
        let userAddr;
      if (chain.rpc_type === "evm") {
          userAddr = $userAddress.evm?.canonical.toLowerCase();
        } else if (
          chain.rpc_type === "aptos" ||
          chain.chain_id === "movement"
        ) {
          // Use the aptos field for Movement
          userAddr = $userAddress.aptos?.canonical.toLowerCase();
        } else {
          // Fallback to cosmos conversion if needed
          userAddr = $userAddrCosmos?.canonical;
        }
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

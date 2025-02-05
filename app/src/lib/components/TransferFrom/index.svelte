<script lang="ts">
  import DebugBox from "$lib/components/TransferFrom/components/DebugBox/index.svelte"
  import {TRANSFER_DEBUG} from "$lib/components/TransferFrom/transfer/config.ts"
  import Intent from "$lib/components/TransferFrom/components/Cube/faces/Intent.svelte"
  import Chains from "$lib/components/TransferFrom/components/Cube/faces/Chains.svelte"
  import Assets from "$lib/components/TransferFrom/components/Cube/faces/Assets.svelte"
  import Transfer from "$lib/components/TransferFrom/components/Cube/faces/Transfer.svelte"
  import Cube from "$lib/components/TransferFrom/components/Cube/index.svelte"
  import type {Chain, Ucs03Channel} from "$lib/types.ts"
  import {userBalancesQuery} from "$lib/queries/balance"
  import {derived, writable, type Writable} from "svelte/store"
  import {getQuoteToken} from "@unionlabs/client"
  import { Result } from "neverthrow";
  import type { BaseToken, QuoteResponse,} from "$lib/components/TransferFrom/transfer/types.ts"
  import {createRawIntentsStore} from "$lib/components/TransferFrom/transfer/raw-intents.ts"
  import {userAddrCosmos} from "$lib/wallet/cosmos"
  import {userAddrEvm} from "$lib/wallet/evm"
  import {userAddressAptos} from "$lib/wallet/aptos"
  import {checkValidation} from "$lib/components/TransferFrom/transfer/validation.ts";
  import {createIntents} from "$lib/components/TransferFrom/transfer/intents.ts";

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

  $: balances = userBalancesQuery({chains, userAddr: $userAddress})

  const intents = derived([rawIntents, balances, userAddress], ([$rawIntents, $balances, $userAddress]) => {
    return createIntents($rawIntents, $balances, $userAddress, chains, ucs03channels)
  })

  const baseToken: Writable<BaseToken | null> = writable(null)
  const quoteToken = derived<typeof intents, Result<QuoteResponse, Error> | null>(intents, ($intents, set) => {
    if (!($intents.sourceChain && $intents.baseToken && $intents.channel)) {
      set(null)
      return
    }

    getQuoteToken(
      $intents.sourceChain.chain_id,
      $intents.baseToken.denom,
      $intents.channel
    ).then(quote => set(quote))
  }, null)

  const validation = derived([intents, balances, userAddress, quoteToken], ([$intents, $balances, $quoteToken]) => {
    return checkValidation($rawIntents, $intents, $balances, $userAddress, $baseToken, $quoteToken)
  })

</script>

<Cube>
  <div slot="intent" let:rotateTo class="w-full h-full">
    <Intent transferArgs={$validation.args} {rawIntents} {intents} {validation} {chains} {rotateTo}/>
  </div>

  <div slot="source" let:rotateTo class="w-full h-full">
    <Chains {rawIntents} {chains} {rotateTo} selected="source"/>
  </div>

  <div slot="destination" let:rotateTo class="w-full h-full">
    <Chains {rawIntents} {chains}  {rotateTo} selected="destination"/>
  </div>

  <div slot="assets" let:rotateTo class="w-full h-full">
    <Assets {rawIntents} {chains} {rotateTo}/>
  </div>

  <div slot="transfer" let:rotateTo class="w-full h-full">
    {#if $validation.args && $validation.context}
      <Transfer transferArgs={$validation.args} transferContext={$validation.context} {chains}
      />
    {/if}
  </div>
</Cube>

<!--<div class="absolute bottom-0 inset-x-0 text-center py-2">-->
<!--  {#if TRANSFER_DEBUG}-->
<!--    <DebugBox {stores}/>-->
<!--  {/if}-->
<!--</div>-->
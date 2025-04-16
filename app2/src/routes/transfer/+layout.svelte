<script lang="ts">
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option } from "effect"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { balancesStore } from "$lib/stores/balances.svelte.ts"

let { children } = $props()

$effect(() => {
  if (Option.isSome(transfer.sourceChain)) {
    tokensStore.fetchTokens(transfer.sourceChain.value.universal_chain_id)
  }
})

let lastFetchKey = $state("")


//0x50A22f95bcB21E7bFb63c7A8544AC0683dCeA302
//0xe2f184241cddd9f2235d861eff25c37b7529746e

$effect(() => {
  if (Option.isNone(transfer.sourceChain)) return

  const sourceChain = transfer.sourceChain.value

  const addressOption = transfer.derivedSender
  if (Option.isNone(addressOption)) return

  const address = addressOption.value

  const tokensOption = tokensStore.data.get(sourceChain.universal_chain_id) ?? Option.none()
  if (Option.isNone(tokensOption)) return

  const fetchKey = `${sourceChain.universal_chain_id}:${address}`

  if (fetchKey !== lastFetchKey) {
    const tokens = tokensOption.value
    const denoms = tokens.map(token => token.denom)

    balancesStore.fetchBalances(sourceChain, address, denoms)
    lastFetchKey = fetchKey
  }
})

let showDetails = $state(false)
</script>

{@render children()}

<!--{#if Object.entries(transfer.validation.fieldErrors).length > 0}-->
<!--  <div class="w-full p-4">-->
<!--    <div class="p-4 rounded bg-red-500 overflow-hidden flex flex-col">-->
<!--      <div class="flex justify-between items-center gap-2">-->
<!--        <h3 class="text-xl font-bold">Transfer Validation Errors</h3>-->

<!--        <Button variant="secondary" class="self-start mt-2" onclick={() => showDetails = !showDetails}>-->
<!--          {showDetails ? "Hide Details ↑" : "Show Details ↓"}-->
<!--        </Button>-->

<!--      </div>-->

<!--      {#if showDetails}-->
<!--        <div class="mt-4">-->
<!--          {#each Object.entries(transfer.validation.fieldErrors) as [field, errors]}-->
<!--            <section class="mb-4">-->
<!--              <h3 class="text-lg font-bold">{field}</h3>-->
<!--              <ul class="text-sm">-->
<!--                {#each errors as errMsg}-->
<!--                  <li>{errMsg}</li>-->
<!--                {/each}-->
<!--              </ul>-->
<!--            </section>-->
<!--          {/each}-->
<!--        </div>-->
<!--      {/if}-->
<!--    </div>-->
<!--  </div>-->
<!--{/if}-->






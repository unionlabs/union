<script lang="ts">
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option } from "effect"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { balancesStore } from "$lib/stores/balances.svelte.ts"

let { children } = $props()

$effect(() => {
  transfer.getQuoteToken()
  transfer.getWethQuoteToken()
})

$effect(() => {
  if (Option.isSome(transfer.sourceChain)) {
    tokensStore.fetchTokens(transfer.sourceChain.value.universal_chain_id)
  }
})

let lastFetchKey = $state("")

$effect(() => {
  if (Option.isNone(transfer.sourceChain)) return

  const sourceChain = transfer.sourceChain.value

  const addressOption = wallets.getAddressForChain(sourceChain)
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
</script>

{@render children()}

{#if transfer.validation.isValid}
  <p class="text-sm">Everything looks good!</p>
{:else}
  <p>Transfer validation errors:</p>
  <ul class="text-xs">
    {#each transfer.validation.messages ?? [] as msg}
      <li>{msg}</li>
    {/each}
  </ul>
{/if}





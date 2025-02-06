<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { truncate } from "$lib/utilities/format"
import ExplorerPrecise from "$lib/components/explorer-precise.svelte"
import Token from "$lib/components/token.svelte"
import type { Chain } from "$lib/types"
import { showTokenDetails } from "$lib/stores/user"

export let chains: Array<Chain>
export let chainId: string
export let token: {
  base: {
    token: string
    amount: string
    chainId: string
  }
  quote: {
    token: string
    amount: string
    chainId: string
  }
}
</script>

<div class="flex flex-col items-start">
  <Token {chains} showWrapping={$showTokenDetails} amount={token.base.amount} denom={token.base.token} chainId={token.base.chainId}/>
  {#if $showTokenDetails}
  <Token {chains}  amount={token.quote.amount} denom={token.quote.token} chainId={token.quote.chainId}/>  {/if}
</div>

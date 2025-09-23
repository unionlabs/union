<script lang="ts">
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import * as O from "effect/Option"

interface Props {
  chain: O.Option<Chain>
  token: O.Option<Token>
  balance: O.Option<bigint>
  symbol: string
  showSymbol?: boolean
  hoverable?: boolean
  title?: string
  class?: string
}

let {
  chain,
  token,
  balance,
  symbol,
  showSymbol = true,
  hoverable = false,
  title = "",
  class: className = "",
}: Props = $props()

const isWalletConnected = $derived(O.isSome(WalletStore.evmAddress))
</script>

<div 
  class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3.5 {hoverable ? 'transition-all hover:border-zinc-700 cursor-pointer' : ''} {className}"
  {title}
>
  <div class="flex justify-between items-center">
    <!-- Token Icon -->
    <div class="flex items-center gap-2">
      {#if O.isSome(chain) && O.isSome(token)}
        <div class="w-8 h-8 [&_img]:!w-8 [&_img]:!h-8 [&>div>div]:!w-8 [&>div>div]:!h-8 [&_span]:hidden">
          <TokenComponent
            chain={chain.value}
            denom={token.value.denom}
            amount={TokenRawAmount.make(0n)}
            showWrapping={false}
            showSymbol={false}
            showIcon={true}
          />
        </div>
        {#if showSymbol}
          <span class="text-sm text-zinc-500 {hoverable ? 'group-hover:text-zinc-400 transition-colors' : ''}">
            {symbol}
          </span>
        {/if}
      {:else}
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 bg-zinc-800 rounded-full animate-pulse"></div>
          {#if showSymbol}
            <div class="w-8 h-4 bg-zinc-800 rounded animate-pulse"></div>
          {/if}
        </div>
      {/if}
    </div>
    
    <!-- Balance -->
    <div class="text-right">
      {#if !isWalletConnected}
        <div class="text-lg font-semibold text-zinc-400">—</div>
      {:else if O.isSome(chain) && O.isSome(token) && O.isSome(balance)}
        <TokenComponent
          chain={chain.value}
          denom={token.value.denom}
          amount={TokenRawAmount.make(balance.value)}
          showWrapping={false}
          showSymbol={true}
          showIcon={false}
          maxDecimals={4}
        />
      {:else}
        <div class="w-20 h-6 bg-zinc-800 rounded animate-pulse ml-auto"></div>
      {/if}
    </div>
  </div>
</div>

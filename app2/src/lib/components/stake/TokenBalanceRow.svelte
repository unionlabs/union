<script lang="ts">
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { chainLogoMap } from "$lib/constants/chain-logos"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import { Array as Arr, Option, pipe } from "effect"
import * as O from "effect/Option"

interface Props {
  chain: O.Option<Chain>
  token: O.Option<Token>
  balance: O.Option<bigint>
  symbol: string
  showSymbol?: boolean
  hoverable?: boolean
  title?: string
  subtitle?: string
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
  subtitle = "",
  class: className = "",
}: Props = $props()

const isWalletConnected = $derived(O.isSome(WalletStore.evmAddress))

const tokenLogo = $derived(
  pipe(
    token,
    Option.map(x => x.representations),
    Option.flatMap(Arr.head),
    Option.flatMap(x => Option.all({ alt: Option.some(x.name), uri: x.logo_uri })),
  ),
)
</script>

<div
  class="rounded-lg bg-zinc-900 border border-zinc-800/50 p-3.5 {hoverable ? 'transition-all hover:border-zinc-700 cursor-pointer' : ''} {className}"
  {title}
>
  <div class="flex justify-between items-center">
    <!-- Chain + Token Logo Combo (scaled 2x from ChainComponent) -->
    <div class="flex items-center gap-2">
      {#if O.isSome(chain) && O.isSome(token)}
        {@const chainLogo = chainLogoMap.get(chain.value.universal_chain_id)}
        {#if chainLogo?.color}
          <div class="flex items-center">
            <div class="relative flex items-center justify-center overflow-visible">
              <img
                src={chainLogo.color}
                class="w-8 h-8 {O.isSome(tokenLogo) ? 'asset-mask' : ''}"
                alt=""
              />
              {#if O.isSome(tokenLogo)}
                {@const alt = tokenLogo.value.alt}
                {@const src = tokenLogo.value.uri}
                <img
                  class="absolute left-5 top-1/2 -translate-y-1/2 w-8 h-8 rounded-full"
                  {src}
                  {alt}
                />
              {/if}
            </div>
          </div>
        {:else}
          <!-- Fallback to just token icon -->
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
        <div class="flex flex-col items-end">
          <TokenComponent
            chain={chain.value}
            denom={token.value.denom}
            amount={TokenRawAmount.make(balance.value)}
            showWrapping={false}
            showSymbol={true}
            showIcon={false}
            maxDecimals={4}
          />
          {#if subtitle === "loading"}
            <div class="text-xs text-zinc-500 mt-0.5">
              <div class="w-20 h-3.5 bg-zinc-800 rounded animate-pulse"></div>
            </div>
          {:else if subtitle}
            <div class="text-xs text-zinc-500 mt-0.5">{subtitle}</div>
          {/if}
        </div>
      {:else}
        <div class="flex flex-col items-end">
          <div class="w-24 h-5 bg-zinc-800 rounded animate-pulse"></div>
          {#if subtitle === "loading"}
            <div class="text-xs text-zinc-500 mt-0.5">
              <div class="w-20 h-3.5 bg-zinc-800 rounded animate-pulse"></div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
.asset-mask {
  mask-image: radial-gradient(
    circle 17px at 108% 50%,
    transparent 0%,
    transparent 99%,
    white 99%,
    white 100%
  );
  mask-composite: exclude;
  -webkit-mask-composite: destination-out;
}
</style>

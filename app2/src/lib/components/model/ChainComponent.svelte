<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { chainLogoMap } from "$lib/constants/chain-logos.ts"
import { Chain, Token, TokenRawDenom } from "@unionlabs/sdk/schema"
import { cn } from "$lib/utils"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import A from "../ui/A.svelte"
import Label from "../ui/Label.svelte"
import LongMonoWord from "../ui/LongMonoWord.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import { Array as Arr, Option, pipe } from "effect"
import { tokensStore } from "$lib/stores/tokens.svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  chain: Chain
  class?: string
  withToken?: TokenRawDenom | undefined
  disableTooltip?: boolean
}

const {
  chain,
  class: className = "",
  disableTooltip = false,
  withToken: denom,
  ...rest
}: Props = $props()

const classes = cn("text-md font-semibold", className)

// Start the query when the component mounts
$effect(() => {
  console.log("FETCHI NGTKENS")
  tokensStore.fetchTokens(chain.universal_chain_id)
})

const token = $derived(
  pipe(
    tokensStore.getData(chain.universal_chain_id),
    Option.flatMap(tokens => Option.fromNullable(tokens.find(t => t.denom === denom)))
  )
)

const tokenLogo = $derived(
  pipe(
    token,
    Option.map(x => x.representations),
    Option.flatMap(Arr.head),
    Option.flatMap(x => Option.all({ alt: Option.some(x.name), uri: x.logo_uri }))
  )
)

$effect(() => {})
</script>

{#if disableTooltip}
  <div class={classes} {...rest}>
    {chain.display_name}
  </div>
{:else}
  <Tooltip>
    {#snippet trigger()}
      {@const chainLogo = chainLogoMap.get(chain.universal_chain_id)}
      <div class="flex gap-1 items-center">
      <div>
        {#if chainLogo?.color}
          <div class="flex items-center">
            <div class="flex items-center justify-center overflow-hidden">
              <img class="size-4" src={chainLogo.color} alt="">
              {#if Option.isSome(tokenLogo)}
                {@const alt = tokenLogo.value.alt}
                {@const src = tokenLogo.value.uri}
                <img class="size-4" {src} {alt} />
              {/if}
            </div>
          </div>
        {/if}
      </div>
      <div class={classes} {...rest}>
        {chain.display_name}
      </div>
      </div>
    {/snippet}

    {#snippet content()}
    {@const chainLogo = chainLogoMap.get(chain.universal_chain_id)}
      <section>
        <div class="flex gap-1 items-center text-lg text-white font-bold">
          <div>
            {#if chainLogo?.color}
              <div class="flex items-center">
                <div class="size-5 flex items-center justify-center overflow-hidden">
                  <img src={chainLogo.color} alt="">
                </div>
              </div>
            {/if}
          </div>
          <div>{chain.display_name}</div>
        </div>
      </section>
      <section>
        <Label>Universal Chain ID</Label>
        <LongMonoWord>{chain.universal_chain_id}</LongMonoWord>
      </section>
      <section>
        <Label>Network Type</Label>
        <div>{chain.rpc_type} {chain.testnet ? 'testnet' : 'mainnet'}</div>
      </section>
      <section>
        <Label>Address Prefix</Label>
        <div>{chain.addr_prefix}</div>
      </section>

      {#if chain.explorers.length > 0}
        <section>
          <Label>Explorers</Label>
          <div class="flex flex-col gap-1">
          {#each chain.explorers as explorer}
            <A href={explorer.home_url}>
              {explorer.display_name}
            </A>
          {/each}
          </div>
        </section>
      {/if}

      {#if settingsStore.showDeveloperChainDetails}
        <section>
          <Label>RPC Endpoints</Label>
          {#each chain.rpcs as rpc}
            <div class="text-white mt-2"><span class="uppercase">{rpc.type}</span>
              <A href={rpc.url}>
                {rpc.url}
              </A>
            </div>
          {/each}
        </section>
        <section>
          <Label>Features</Label>
          <div class="grid grid-cols-2 gap-x-4 gap-y-2 mt-2">
            {#each Object.entries(chain.features[0] || {}) as [key, enabled]}
              <div class="flex items-center gap-2">
                <div class={cn(
                  "w-2 h-2 rounded-full",
                  enabled ? "bg-green-500" : "bg-red-500"
                )} ></div>
                <span class="capitalize">{key.replace(/_/g, ' ')}</span>
              </div>
            {/each}
          </div>
        </section>
      {/if}

    {/snippet}
  </Tooltip>
{/if}

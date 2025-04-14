<script lang="ts">
  import {chains} from "$lib/stores/chains.svelte"
  import {tokensStore} from "$lib/stores/tokens.svelte"
  import {Option} from "effect"
  import TokenComponent from "$lib/components/model/TokenComponent.svelte"
  import Card from "$lib/components/ui/Card.svelte"
  import Sections from "$lib/components/ui/Sections.svelte"
  import ChainComponent from "$lib/components/model/ChainComponent.svelte"

  $effect(() => {
  if (Option.isSome(chains.data)) {
    chains.data.value.forEach(c => tokensStore.fetchTokens(c.universal_chain_id))
  }
})
</script>

<Sections>
{#if Option.isSome(chains.data)} 
  {#each chains.data.value as chain}
    <Card class="overflow-none">
      <ChainComponent {chain}/>
      
      {#if tokensStore.data.has(chain.universal_chain_id)}
        {@const chainTokens = tokensStore.getData(chain.universal_chain_id)}
        {#if Option.isSome(chainTokens)}
          <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
            {#each chainTokens.value as token}
              <TokenComponent {chain} denom={token.denom} />
            {/each}
          </div>
        {:else}
          <p class="text-zinc-500">No tokens found</p>
        {/if}
      {:else}
        <p class="text-zinc-500">Loading tokens...</p>
      {/if}
    </Card>
  {/each}
{:else}
  <p class="text-zinc-500">Loading chains...</p>
{/if}
</Sections>

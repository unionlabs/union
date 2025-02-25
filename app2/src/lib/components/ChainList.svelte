<script lang="ts">
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
</script>

{#if Option.isSome(chains.data)}
  <ul>
  {#each chains.data.value as chain}
    <li><ChainComponent {chain}/></li>
  {/each}
  </ul>
{:else}
  Loading...
{/if}

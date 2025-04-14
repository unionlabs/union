<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte"
  import Card from "$lib/components/ui/Card.svelte"
  import SharpWalletIcon from "$lib/components/icons/SharpWalletIcon.svelte"
  import AddressComponent from "$lib/components/model/AddressComponent.svelte"
  import {Option} from "effect"

  import {wallets} from "$lib/stores/wallets.svelte.ts"
  import {chains} from "$lib/stores/chains.svelte.ts"
  import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
</script>

<Card divided class="self-center">
  <div class="p-4 flex gap-1 ">
    <h2>UNO Faucet</h2>
  </div>
  {#if Option.isSome(chains.data)}
  {@const unionTestnet10 = Option.fromNullable(chains.data.value.find(c => c.universal_chain_id === "union.union-testnet-10"))} <div class="flex flex-col gap-4 p-4">
      <div>
        <p>Official faucet for the UNO testnet token.</p>
        <p>This faucet is protected by CloudFlare Turnstile.</p>
        <p>You can use this faucet once a day.</p>
      </div>
      <div>
       <div class="flex items-center mr-5 text-zinc-400 justify-self-end">
        {#if Option.isSome(wallets.cosmosAddress) && Option.isSome(unionTestnet10)}
          <p class="text-xs mb-2">
            <AddressComponent truncate address={wallets.cosmosAddress.value} chain={unionTestnet10.value}/>
          </p>
        {:else}
          <p class="text-xs mb-2"> No receiver</p>
        {/if}
        <AngleArrowIcon class="rotate-270"/>
      </div>
      <div class="flex gap-4">
        <Button class="flex-1">Claim</Button>
        <Button><SharpWalletIcon class="size-5"/></Button>
      </div>
      </div>
    </div>
  {/if}
</Card>

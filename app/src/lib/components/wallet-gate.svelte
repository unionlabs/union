<script lang="ts">
import { rawToHex } from "$lib/utilities/address";
import { cosmosStore } from "$lib/wallet/cosmos";
import { sepoliaStore } from "$lib/wallet/evm";
import { derived, type Readable } from "svelte/store";

type UserAddresses = {
    cosmos: {
      canonical: string,
      normalized: string,
      bytes: Uint8Array
    },
    evm: {
      canonical: string,
      normalized: string,
    }
  }; 

let userAddr: Readable<UserAddresses | null> = derived([cosmosStore, sepoliaStore], ([$cosmosStore, $sepoliaStore]) => {
  if (!($cosmosStore?.rawAddress && $cosmosStore?.address && $sepoliaStore?.address  )) return null;
  return {
    cosmos: {
      canonical: $cosmosStore.address,
      normalized: rawToHex($cosmosStore.rawAddress),
      bytes: $cosmosStore.rawAddress
    },
    evm: {
      canonical: $sepoliaStore.address,
      normalized: $sepoliaStore.address.slice(2).toLowerCase(),
    }
  }
});

let confirmedUserAddr: Readable<UserAddresses> = derived(userAddr, ($userAddr) => {
  if ($userAddr === null) {
    // this will never happen, but is needed to satisy svelte's prop type checker
    return {
      cosmos: { canonical: "never", normalized: "never", bytes: new Uint8Array([]) },
      evm: { canonical: "never", normalized: "never", }
    } 
  } 
  return $userAddr;
});
</script>


{#if !$userAddr }
  <div>Connect your wallets to continue.</div>
{:else if $userAddr}
  <slot userAddr={$confirmedUserAddr} />
{/if}


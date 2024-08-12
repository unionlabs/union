<script lang="ts">
import { rawToHex } from "$lib/utilities/address"
import { derived, writable, type Readable } from "svelte/store"
import type { UserAddressCosmos } from "$lib/types"
import type { Address } from "viem"
import { cosmosStore } from "$lib/wallet/cosmos"
import { onMount } from "svelte"
import { sleep } from "$lib/utilities"

let userAddr: Readable<UserAddressCosmos | null> = derived([cosmosStore], ([$cosmosStore]) => {
  if ($cosmosStore?.rawAddress && $cosmosStore?.address) {
    const cosmos_normalized = rawToHex($cosmosStore.rawAddress)
    return {
      canonical: $cosmosStore.address,
      normalized: cosmos_normalized,
      bytes: $cosmosStore.rawAddress,
      normalized_prefixed: `0x${cosmos_normalized}` as Address
    }
  }

  return null
})

let confirmedUserAddr: Readable<UserAddressCosmos> = derived(userAddr, $userAddr => {
  return (
    $userAddr ?? {
      canonical: "never",
      normalized: "never",
      bytes: new Uint8Array([]),
      normalized_prefixed: "0x0" as Address
    }
  )
})
</script>

{#if $userAddr}
  <slot name="connected" userAddrCosmos={$confirmedUserAddr} />
{:else}
  <slot name="disconnected">
    <span>Connect your Cosmos wallet to continue</span>
  </slot>
{/if}



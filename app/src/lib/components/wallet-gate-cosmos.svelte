<script lang="ts">
import { derived, type Readable } from "svelte/store"
import type { UserAddressCosmos } from "$lib/types"
import type { Address } from "viem"
import { userAddrCosmos } from "$lib/wallet/cosmos"

let confirmedUserAddr: Readable<UserAddressCosmos> = derived(userAddrCosmos, $userAddr => {
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

{#if $userAddrCosmos}
  <slot name="connected" userAddrCosmos={$confirmedUserAddr} />
{:else}
  <slot name="disconnected">
    <span>Connect your Cosmos wallet to continue</span>
  </slot>
{/if}



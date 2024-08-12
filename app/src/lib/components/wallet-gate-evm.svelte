<script lang="ts">
import { derived, type Readable } from "svelte/store"
import type { UserAddressEvm } from "$lib/types"
import type { Address } from "viem"
import { userAddrEvm } from "$lib/wallet/evm"

let confirmedUserAddr: Readable<UserAddressEvm> = derived(userAddrEvm, $userAddr => {
  return (
    $userAddr ?? {
      canonical: "0xnever" as Address,
      normalized: "never",
      normalized_prefixed: "0x0" as Address
    }
  )
})
</script>

{#if $userAddrEvm}
  <slot name="connected" userAddrEvm={$confirmedUserAddr} />
{:else}
  <slot name="disconnected">
    <span>Connect your EVM wallet to continue</span>
  </slot>
{/if}



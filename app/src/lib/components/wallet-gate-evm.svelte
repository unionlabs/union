<script lang="ts">
import { rawToHex } from "$lib/utilities/address"
import { derived, writable, type Readable } from "svelte/store"
import type { UserAddressCosmos, UserAddresses, UserAddressEvm } from "$lib/types"
import type { Address } from "viem"
import { onMount } from "svelte"
import { sleep } from "$lib/utilities"
import { sepoliaStore } from "$lib/wallet/evm"

let userAddr: Readable<UserAddressEvm | null> = derived([sepoliaStore], ([$sepoliaStore]) => {
  if ($sepoliaStore?.address) {
    const evm_normalized = $sepoliaStore.address.slice(2).toLowerCase()
    return {
      canonical: $sepoliaStore.address as Address,
      normalized: $sepoliaStore.address.slice(2).toLowerCase(),
      normalized_prefixed: `0x${evm_normalized}` as Address
    }
  }

  return null
})

let confirmedUserAddr: Readable<UserAddressEvm> = derived(userAddr, $userAddr => {
  return (
    $userAddr ?? {
      canonical: "0xnever" as Address,
      normalized: "never",
      normalized_prefixed: "0x0" as Address
    }
  )
})
</script>

{#if $userAddr}
  <slot name="connected" userAddrEvm={$confirmedUserAddr} />
{:else}
  <slot name="disconnected">
    <span>Connect your EVM wallet to continue</span>
  </slot>
{/if}



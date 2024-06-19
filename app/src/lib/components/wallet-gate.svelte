<script lang="ts">
import { rawToHex } from "$lib/utilities/address"
import { cosmosStore } from "$lib/wallet/cosmos"
import { sepoliaStore } from "$lib/wallet/evm"
import { derived, type Readable } from "svelte/store"
import type { UserAddresses } from "$lib/types"
import type { Address } from "viem"

let userAddr: Readable<UserAddresses | null> = derived(
  [cosmosStore, sepoliaStore],
  ([$cosmosStore, $sepoliaStore]) => {
    if (!($cosmosStore?.rawAddress && $cosmosStore?.address && $sepoliaStore?.address)) return null

    // sometimes rawAddress is truthy but does not yield a raw hex addr
    const cosmos_normalized = rawToHex($cosmosStore.rawAddress)
    if (!cosmos_normalized) return null

    const evm_normalized = $sepoliaStore.address.slice(2).toLowerCase()

    return {
      cosmos: {
        canonical: $cosmosStore.address,
        normalized: cosmos_normalized,
        bytes: $cosmosStore.rawAddress,
        normalized_prefixed: `0x${cosmos_normalized}` as Address
      },
      evm: {
        canonical: $sepoliaStore.address,
        normalized: evm_normalized,
        normalized_prefixed: `0x${evm_normalized}` as Address
      }
    }
  }
)

let confirmedUserAddr: Readable<UserAddresses> = derived(userAddr, $userAddr => {
  if ($userAddr === null) {
    // this will never happen, but is needed to satisfy svelte's prop type checker
    return {
      cosmos: {
        canonical: "never",
        normalized: "never",
        bytes: new Uint8Array([]),
        normalized_prefixed: "0x0" as Address
      },
      evm: {
        canonical: "0xnever" as Address,
        normalized: "never",
        normalized_prefixed: "0x0" as Address
      }
    }
  }
  return $userAddr
})
</script>

{#if $userAddr}
  <slot userAddr={$confirmedUserAddr} />
{:else}
  <slot name="disconnected">
    <span>Connect your wallets to continue</span>
  </slot>
{/if}




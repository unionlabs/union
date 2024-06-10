<script lang="ts">
import { rawToHex } from "$lib/utilities/address"
import { cosmosStore } from "$lib/wallet/cosmos"
import { sepoliaStore } from "$lib/wallet/evm"
import { derived, type Readable } from "svelte/store"
import type { UserAddresses } from "$lib/types"

let userAddr: Readable<UserAddresses | null> = derived(
  [cosmosStore, sepoliaStore],
  ([$cosmosStore, $sepoliaStore]) => {
    if (!($cosmosStore?.rawAddress && $cosmosStore?.address && $sepoliaStore?.address)) return null

    // sometimes rawAddress is truthy but does not yield a raw hex addr
    const cosmos_normalized = rawToHex($cosmosStore.rawAddress)
    if (!cosmos_normalized) return null

    return {
      cosmos: {
        canonical: $cosmosStore.address,
        normalized: cosmos_normalized,
        bytes: $cosmosStore.rawAddress
      },
      evm: {
        canonical: $sepoliaStore.address,
        normalized: $sepoliaStore.address.slice(2).toLowerCase()
      }
    }
  }
)

let confirmedUserAddr: Readable<UserAddresses> = derived(userAddr, $userAddr => {
  if ($userAddr === null) {
    // this will never happen, but is needed to satisfy svelte's prop type checker
    return {
      cosmos: { canonical: "never", normalized: "never", bytes: new Uint8Array([]) },
      evm: { canonical: "never", normalized: "never" }
    }
  }
  return $userAddr
})
</script>


{#if !$userAddr }
  <div>Connect your wallets to continue.</div>
{:else if $userAddr}
  <slot userAddr={$confirmedUserAddr} />
{/if}


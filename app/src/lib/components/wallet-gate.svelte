<script lang="ts">
import { rawToHex } from "$lib/utilities/address"
import { derived, writable, type Readable } from "svelte/store"
import type { UserAddressCosmos, UserAddresses, UserAddressEvm } from "$lib/types"
import type { Address } from "viem"
import { onMount } from "svelte"
import { sleep } from "$lib/utilities"
import { cosmosStore } from "$lib/wallet/cosmos"
import { sepoliaStore } from "$lib/wallet/evm"

let loading = writable(true)

let userAddr: Readable<UserAddresses | null> = derived(
  [cosmosStore, sepoliaStore],
  ([$cosmosStore, $sepoliaStore]) => {
    let cosmosData: UserAddressCosmos | null = null
    let evmData: UserAddressEvm | null = null

    if ($cosmosStore?.rawAddress && $cosmosStore?.address) {
      const cosmos_normalized = rawToHex($cosmosStore.rawAddress)
      cosmosData = cosmos_normalized
        ? {
            canonical: $cosmosStore.address,
            normalized: cosmos_normalized,
            bytes: $cosmosStore.rawAddress,
            normalized_prefixed: `0x${cosmos_normalized}` as Address
          }
        : null
    }

    if ($sepoliaStore?.address) {
      const evm_normalized = $sepoliaStore.address.slice(2).toLowerCase()
      evmData = {
        canonical: $sepoliaStore.address as Address,
        normalized: evm_normalized,
        normalized_prefixed: `0x${evm_normalized}` as Address
      }
    }

    return cosmosData || evmData ? { cosmos: cosmosData, evm: evmData } : null
  }
)

let confirmedUserAddr: Readable<UserAddresses> = derived(userAddr, $userAddr => {
  return (
    $userAddr ?? {
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
  )
})

$: console.info($userAddr)

onMount(async () => {
  await sleep(100)
  loading.set(false)
})
</script>

<slot
  userAddr={$confirmedUserAddr}
  connected={!!$userAddr}
  evmConnected={$userAddr?.evm}
  cosmosConnected={$userAddr?.cosmos}
/>



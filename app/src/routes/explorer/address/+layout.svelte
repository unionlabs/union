<script lang="ts">
import { setContext } from "svelte"
import { derived } from "svelte/store"
import DevTools from "$lib/components/dev-tools.svelte"
import { sepoliaStore } from "$lib/wallet/evm/config.ts"
import { cosmosStore } from "$lib/wallet/cosmos/config.ts"

let addressArray = derived([sepoliaStore, cosmosStore], ([$sepoliaStore, $cosmosStore]) =>
  [$sepoliaStore.address, $cosmosStore.address].filter(Boolean)
)

setContext("addressArray", addressArray)
</script>

<DevTools></DevTools>

<slot />

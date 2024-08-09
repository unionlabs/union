<script lang="ts">
import { page } from "$app/stores"
import { setContext } from "svelte"
import { derived } from "svelte/store"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { isValidEvmAddress, bech32AddressToHex, isValidBech32Address } from "@union/client"

/**
 * TODO: instead of displaying data here, go to error page and display a proper error message
 */

let addresses = derived(page, $page => {
  const slug = $page.params.slug
  if (!slug) return []

  const addresses = slug.indexOf("-") === -1 ? [slug] : slug.split("-")

  return addresses.map(address => {
    let normalizedAddress = address
    if (isValidEvmAddress(address)) {
      normalizedAddress = address.slice(2).toLowerCase()
    }
    if (isValidBech32Address(address)) {
      normalizedAddress = bech32AddressToHex({ address }).slice(2).toLowerCase()
    }
    return {
      address,
      normalizedAddress
    }
  })
})

setContext<typeof addresses>("addresses", addresses)
</script>

<div>
  <ChainsGate let:chains>
    {#if $addresses.find( address => chains.find( chain => address.address.startsWith(chain.addr_prefix) ) )}
      <slot />
    {:else}
      <p>Invalid address(es)</p>
    {/if}
  </ChainsGate>
</div>

<script lang="ts">
import { page } from "$app/stores"
import { setContext } from "svelte"
import { derived, type Readable } from "svelte/store"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { address as addressUtilities } from "@union/client"

/**
 * TODO: instead of displaying data here, go to error page and display a proper error message
 */

let addressArray = derived(page, $page => {
  const slug = $page.params.slug
  const addresses = slug.indexOf("-") === -1 ? [slug] : slug.split("-")

  return {
    nonNormalized: addresses,
    normalized: addresses.map(address => {
      if (addressUtilities.isValidEvmAddress(address)) {
        return address.slice(2).toLowerCase()
      }
      if (addressUtilities.isValidBech32Address(address)) {
        return addressUtilities.bech32AddressToHex({ address }).slice(2).toLowerCase()
      }
      return address
    })
  }
})

setContext<typeof addressArray>("addressArray", addressArray)
</script>

<ChainsGate let:chains>
  {@const slug = $page.params.slug}
  {@const addressArray = slug.indexOf("-") === -1 ? [slug] : slug.split("-")}
  <!--
    this is not optimal so we should definitely improve this
  -->
  {#if addressArray.find( (address) => chains.find( (chain) => address.startsWith(chain.addr_prefix) ) )}
    <slot />
  {:else}
    <p>Invalid address</p>
  {/if}
</ChainsGate>

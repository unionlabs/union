<script lang="ts">
  import {
    isValidEvmAddress,
    bech32AddressToHex,
    isValidBech32Address
  } from "@union/client"
  import { page } from "$app/stores"
  import { setContext } from "svelte"
  import { derived } from "svelte/store"
  import ChainsGate from "$lib/components/chains-gate.svelte"

  /**
   * TODO: instead of displaying data here, go to error page and display a proper error message
   */

  let addressArray = derived(page, ($page) => {
    const slug = $page.params.slug
    if (!slug) return { nonNormalized: [], normalized: [] }
    const addresses = slug.indexOf("-") === -1 ? [slug] : slug.split("-")

    const normalizedAddresses = addresses.map((address) => {
      if (isValidEvmAddress(address)) {
        return address.slice(2).toLowerCase()
      }
      if (isValidBech32Address(address)) {
        return bech32AddressToHex({ address }).slice(2).toLowerCase()
      }
      return address
    })
    return {
      nonNormalized: [...new Set(addresses)],
      normalized: [...new Set(normalizedAddresses)]
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

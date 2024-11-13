<script lang="ts">
import { derived, type Readable } from "svelte/store"
import type { UserAddressCosmos } from "$lib/types"
import type { Address } from "viem"
import { userAddrCosmos } from "$lib/wallet/cosmos"
interface Props {
  connected?: import("svelte").Snippet<[any]>
  disconnected?: import("svelte").Snippet
}

let { connected, disconnected }: Props = $props()

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
  {@render connected?.({ userAddrCosmos: $confirmedUserAddr, })}
{:else}
  {#if disconnected}{@render disconnected()}{:else}
    <span>Connect your Cosmos wallet to continue</span>
  {/if}
{/if}



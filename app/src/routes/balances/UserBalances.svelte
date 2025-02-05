<script lang="ts">
import type { Chain, UserAddresses } from "$lib/types"
import { balances, deleteBalancesForRpcType, queryBalances } from "$lib/stores/balances.ts"
import { onMount } from "svelte"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { derived, type Readable } from "svelte/store"

export let chains: Array<Chain>

let userAddress: Readable<UserAddresses> = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$userAddrCosmos, $userAddrEvm, $userAddressAptos]) => ({
    evm: $userAddrEvm,
    cosmos: $userAddrCosmos,
    aptos: $userAddressAptos
  })
)

onMount(() => {
  const unsubscribe = userAddress.subscribe(addr => {
    console.log("new address", addr)

    // wipe balances on wallet disconnect
    if (!addr.evm) deleteBalancesForRpcType(chains, "evm")
    if (!addr.cosmos) deleteBalancesForRpcType(chains, "evm")
    if (!addr.aptos) deleteBalancesForRpcType(chains, "aptos")

    // foreach chain with a connected wallet, fetch balances.
    chains
      .filter(chain => addr[chain.rpc_type])
      .forEach(chain => queryBalances(chain, addr[chain.rpc_type]?.canonical as string))
  })
  return unsubscribe
})
</script>

<div class="space-y-6">
  <pre>{JSON.stringify($userAddress, null, 2)}</pre>
  <pre>{JSON.stringify($balances, null, 2)}</pre>
</div>

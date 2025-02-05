<script lang="ts">
import type { Chain } from "$lib/types"
import { deleteBalancesForRpcType, queryBalances } from "$lib/stores/balances.ts"
import { onMount } from "svelte"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"

export let chains: Array<Chain>

onMount(() => {
  const unsubscribe = userAddrEvm.subscribe(evmAddr => {
    if (!evmAddr) {
      deleteBalancesForRpcType(chains, "evm")
      return
    }
    chains
      .filter(chain => chain.rpc_type === "evm")
      .forEach(chain => queryBalances(chain, evmAddr.canonical))
  })
  return unsubscribe
})

onMount(() => {
  const unsubscribe = userAddrCosmos.subscribe(cosmosAddr => {
    if (!cosmosAddr) {
      deleteBalancesForRpcType(chains, "cosmos")
      return
    }
    chains
      .filter(chain => chain.rpc_type === "cosmos")
      .forEach(chain => queryBalances(chain, cosmosAddr.canonical))
  })
  return unsubscribe
})

onMount(() => {
  const unsubscribe = userAddressAptos.subscribe(aptosAddr => {
    if (!aptosAddr) {
      deleteBalancesForRpcType(chains, "aptos")
      return
    }
    chains
      .filter(chain => chain.rpc_type === "aptos")
      .forEach(chain => queryBalances(chain, aptosAddr.canonical))
  })
  return unsubscribe
})
</script>

<slot/>

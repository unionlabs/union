<script lang="ts">
import type { Chain } from "$lib/types"
import { deleteBalancesForRpcType, queryBalances } from "$lib/stores/balances.ts"
import { onMount } from "svelte"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"

export let chains: Array<Chain>

;[
  { userAddr: userAddrCosmos, rpcType: "cosmos" },
  { userAddr: userAddrEvm, rpcType: "evm" },
  { userAddr: userAddressAptos, rpcType: "aptos" }
].forEach(({ userAddr, rpcType }) => {
  onMount(() => {
    const unsubscribe = userAddr.subscribe(addr => {
      if (!addr) {
        deleteBalancesForRpcType(chains, rpcType)
        return
      }
      chains
        .filter(chain => chain.rpc_type === rpcType)
        .forEach(chain => queryBalances(chain, addr.canonical))
    })
    return unsubscribe
  })
})
</script>

<slot/>

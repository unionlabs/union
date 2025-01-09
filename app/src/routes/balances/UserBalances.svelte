<script lang="ts">
    import { userBalancesQuery } from "$lib/queries/balance";


import type { Chain, UserAddresses } from "$lib/types.ts"
    import { userAddressAptos } from "$lib/wallet/aptos";
    import { userAddrCosmos } from "$lib/wallet/cosmos";
    import { userAddrEvm } from "$lib/wallet/evm";
    import { derived } from "svelte/store";
export let chains: Array<Chain>

  
let userAddress: Readable<UserAddresses> = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$userAddrCosmos, $userAddrEvm, $userAddressAptos]) => ({
    evm: $userAddrEvm,
    cosmos: $userAddrCosmos,
    aptos: $userAddressAptos
  })
)

$: userBalances = userBalancesQuery({ chains, userAddr: $userAddress, connected: true })
$: console.log({$userBalances})

</script>

<p>hi</p>


<div>
{#each $userBalances as balance}
  {JSON.stringify(balance.data, 2)}
{/each}

<h2>chains</h2>
{JSON.stringify(chains, null)}
</div>


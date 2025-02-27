<script lang="ts">
import { Chain } from "$lib/schema/chain"
interface Props {
  chain: Chain
}

let { chain }: Props = $props()
</script>

<div>
	<p class="text-md font-bold">{chain.display_name}</p>
	<!--
	<div>{chain.chain_id}</div>
	<div>{chain.testnet}</div>
	<div>{chain.rpc_type}</div>
	<div>{JSON.stringify(chain.features)}</div>
	!-->
</div>

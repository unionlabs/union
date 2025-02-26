<script lang="ts">
import { Chain } from "$lib/schema/chain"
interface Props {
  chain: Chain
}

let { chain }: Props = $props()
</script>

<div class="p-4">
	<h2 class="text-lg font-bold">{chain.display_name}</h2>
	<div>{chain.chain_id}</div>
	<div>{chain.testnet}</div>
	<div>{JSON.stringify(chain.features)}</div>
</div>

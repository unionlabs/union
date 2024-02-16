<script lang="ts">
  import { getUnoFromFaucet } from '$/lib/faucet.ts'
  import { unionAddress } from '$/lib/wallet/config'
</script>

<div>
  <button on:click={() => getUnoFromFaucet({ address: $unionAddress })}>
    get UNO from faucet
  </button>
</div>

<script lang="ts">
import genesis from '../../../networks/genesis/union-testnet-5/genesis.json'

/**
 * do some logic 
 */
</script>

<button>
  <a href={"data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(genesis,undefined,2))} download="genesis.json">
    Download Genesis
  </a>
</button>


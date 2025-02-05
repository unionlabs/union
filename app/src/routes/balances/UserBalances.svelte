<script lang="ts">
import { balances } from "$lib/stores/balances.ts"
</script>

<div>
  <pre>{JSON.stringify($balances, null, 2)}</pre>
</div>

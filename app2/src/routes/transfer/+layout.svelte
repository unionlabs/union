<script lang="ts">
  import {createTransfer} from "$lib/components/Transfer/transfer.svelte.ts";

  let { children } = $props()

  createTransfer()
</script>

{@render children()}
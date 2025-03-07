<script lang="ts">
  import {createTransfer} from "./transfer.svelte.ts";

  let { children } = $props()

  createTransfer()
</script>

{@render children()}
<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>

<ChainsGate let:chains>
  <div class="w-full flex flex-col items-center">
    <slot {chains} />
  </div>
</ChainsGate>

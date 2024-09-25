<script lang="ts">

import Print from "$lib/components/TerminalApp/Print.svelte";

type Props = {
  stupid?: boolean
}

let { stupid = true } = $props()
</script>

<div class="text-center font-bold text-lg">
  {#if stupid}
    <Print>Your MPC Client is connected.</Print>
    <Print>Do not close this tab or your Terminal.</Print>
  {/if}
  <Print></Print>
  <Print>Ensure you have a reliable internet connection <br> and that your computer <a
          class="underline underline-offset-4 decoration-union-accent-500"
          href="https://support.apple.com/guide/mac-help/set-sleep-and-wake-settings-mchle41a6ccd/mac" target="_blank">does
    not go to sleep</a>.
  </Print>
</div>